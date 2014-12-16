use std::collections::{HashMap,HashSet};
use std::sync::{Arc,Mutex};
use std::fmt;

use parser::Node;


type EvalBox<T> = Arc<Mutex<T>>;
pub type BoxedValue = EvalBox<Value>;
pub type BoxedScope = EvalBox<Scope>;
pub type EvalResult = Result<BoxedValue, String>;


#[deriving(Clone)]
pub enum Value {
	Nil,
	List(Vec<BoxedValue>),
	Bool(bool),
	Int(i64),
	Float(f64),
	String(String),
	Atom(String),
	Function {
		code: Node,
		locals: BoxedScope,
	},
	BuiltinFunc(String),
	SpecialForm(String)
}

impl fmt::Show for Mutex<Value> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self.try_lock() {
			Some(x) => write!(f, "{}", x.deref()),
			None => write!(f, "<locked>"),
		}
	}
}

impl fmt::Show for Value {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			&Value::Nil => write!(f, "Nil"),
			&Value::List(ref x) => write!(f, "List({})", x),
			&Value::Bool(ref b) => write!(f, "Bool({})", b),
			&Value::Int(ref i) => write!(f, "Int({})", i),
			&Value::BuiltinFunc(ref func) => write!(f, "BuiltinFunc({})", func),
			&Value::SpecialForm(ref sf) => write!(f, "SpecialForm({})", sf),
			_ => write!(f, "UnknownValue")
		}
	}
}

impl Value {
	pub fn unwrap_int(&self) -> i64 {
		match self {
			&Value::Int(i) => i,
			_ => panic!("not an int")
		}
	}
}


#[deriving(Clone)]
pub struct Scope {
	vars: HashMap<String, BoxedValue>,
	parent: Option<BoxedScope>
}


impl Scope {
	pub fn new() -> Scope {
		Scope {
			vars: HashMap::new(),
			parent: None
		}
	}

	pub fn with_parent(parent: BoxedScope) -> Scope {
		Scope {
			vars: HashMap::new(),
			parent: Some(parent)
		}
	}

	pub fn get(&self, key: &str) -> Option<BoxedValue> {
		match self.vars.get(key.as_slice()) {
			Some(val) => Some(val.clone()),
			None => match self.parent {
				Some(ref p) => p.lock().get(key),
				None => None
			}
		}
	}

	pub fn insert(&mut self, key: String, val: BoxedValue) -> Option<BoxedValue> {
		self.vars.insert(key, val)
	}
}

pub struct Interpreter {
	globals: BoxedScope,
	atoms: HashSet<String>
}

impl Interpreter {
	pub fn new() -> Interpreter {
		let mut globals = Scope::new();
		globals.vars.insert(String::from_str("+"), Arc::new(Mutex::new(Value::BuiltinFunc(String::from_str("+")))));
		globals.vars.insert(String::from_str("+."), Arc::new(Mutex::new(Value::BuiltinFunc(String::from_str("+.")))));
		globals.vars.insert(String::from_str("defun"), Arc::new(Mutex::new(Value::SpecialForm(String::from_str("defun")))));
		globals.vars.insert(String::from_str("let"), Arc::new(Mutex::new(Value::SpecialForm(String::from_str("let")))));
		globals.vars.insert(String::from_str("def"), Arc::new(Mutex::new(Value::SpecialForm(String::from_str("def")))));
		let mut intp = Interpreter {
			globals: Arc::new(Mutex::new(globals)),
			atoms: HashSet::new()
		};
		intp
	}


	pub fn eval(&mut self, ast: &Node, scope: BoxedScope) -> EvalResult {
		match ast {
			&Node::IntLiteral(i) => Ok(Arc::new(Mutex::new(Value::Int(i)))),
			&Node::FloatLiteral(f) => Ok(Arc::new(Mutex::new(Value::Float(f)))),
			&Node::Symbol(ref s) => match scope.lock().get(s.as_slice()) {
				None => Err(format!("name '{}' not found in scope", s)),
				Some(val) => Ok(val.clone())
			},
			&Node::List(ref vec) => self.eval_list(ast, &scope, vec.as_slice()),
			_ => Err(format!("{} not implemeted", ast)),
		}
	}

	pub fn eval_globals(&mut self, ast: &Node) -> EvalResult {
		let globals = self.globals.clone();
		self.eval(ast, globals)
	}

	fn eval_list(&mut self, ast: &Node, scope: &BoxedScope, argv: &[Node]) -> EvalResult {
		if let &Node::Symbol(ref sym) = &argv[0] {
			if let Some(v) = scope.lock().get(sym.as_slice()) {
				if let &Value::SpecialForm(ref name) = v.lock().deref() {
					return self.eval_special(ast, name.as_slice(), scope, argv)
				}
			}
		}
		let mut unpacked: Vec<BoxedValue> = vec!();
		for val in argv.iter() {
			let evaled = try!(self.eval(val, scope.clone()));
			unpacked.push(evaled);
		}
		self.funcall(scope, unpacked)
	}

	fn eval_special(&mut self, ast: &Node, name: &str, scope: &BoxedScope, argv: &[Node]) -> EvalResult {
		let argvit = argv.iter();
		let args = argvit.slice_from_or_fail(&1);
		match name {
			"def" => self.special_def(scope, args),
			_ => Err(format!("special form {} not implemented", name))
		}
	}

	fn funcall(&mut self, scope: &BoxedScope, argv: Vec<BoxedValue>) -> EvalResult {
		let argvit = argv.iter();
		let args = argvit.slice_from_or_fail(&1);
		let funmut = argv[0].lock();
		let fun = funmut.deref();

		match fun {
			&Value::BuiltinFunc(ref name) => self.call_builtin(scope, name.as_slice(), args),
			_ => Err(format!("calling {} not implemented", fun))
		}
	}

	fn call_builtin(&mut self, scope: &BoxedScope, fun: &str, args: &[BoxedValue]) -> EvalResult {
		match fun {
			"+" => self.builtin_addi(scope, args),
			_ => Err(format!("{} not implemented", fun))
		}
	}

	///////////
	// builtins

	fn builtin_addi(&self, scope: &BoxedScope, args: &[BoxedValue]) -> EvalResult {
		let mut r = 0i64;
		for v in args.iter() {
			match v.lock().deref() {
				&Value::Int(i) => r += i,
				_ => return Err(format!("{} is not an int", v))
			}
		}
		Ok(Arc::new(Mutex::new(Value::Int(r))))
	}

	////////////////
	// special forms
	
	fn special_def(&mut self, scope: &BoxedScope, args: &[Node]) -> EvalResult {
		if args.len() != 2 {
			return Err(format!("def expects 2 arguments, got {}", args.len()))
		}

		let sym = &args[0];
		let val = &args[1];

		let name = match sym {
			&Node::Symbol(ref name) => name,
			_ => return Err(format!("{} is not a symbol", sym))
		};

		let evaled = try!(self.eval(val, scope.clone()));
		let eval2 = evaled.clone();
		// need RefCell here I guess
		scope.lock().insert(name.clone(), evaled);

		Ok(eval2)
	}
}
