use std::io;

mod lexer;
mod parser;
mod eval;

fn rep(interp: &mut eval::Interpreter, s: &str) -> eval::EvalResult {
	let mut lexer = lexer::Lexer::new(s.as_slice());
	let r = lexer.scan();
	println!("lex:   {}", r);
	let rr = try!(r);
	let parsed = parser::parse(rr);
	println!("parse: {}", parsed);
	let ast = try!(parsed);
	interp.eval_globals(&ast)
}

fn main() {
	let mut interp = eval::Interpreter::new();
	print!(">>> ");
	for line in io::stdin().lock().lines() {
		let s = line.unwrap();
		let val = rep(&mut interp, s.as_slice());
		println!("result: {}", val);
		print!(">>> ");
	}
}

#[test]
fn test() {
	let mut interp = eval::Interpreter::new();
	let r1 = rep(&mut interp, "(+ 1 2)").unwrap().lock().unwrap_int();
	assert_eq!(r1, 3);

	let r2 = rep(&mut interp, "(def x x)").unwrap_err();
	assert!(r2.len() > 0);

	let r3 = rep(&mut interp, "(def a (+ (def x 1) (def y 2) x y))").unwrap().lock().unwrap_int();
	assert_eq!(r3, 6);
}
