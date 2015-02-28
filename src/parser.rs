use std::io;
use std::slice::Iter;

use lexer::Token;


#[derive(Debug,Clone,PartialEq)]
pub enum Node {
	Nothing,
	List(Vec<Node>),
	BoolLiteral(bool),
	IntLiteral(i64),
	FloatLiteral(f64),
	StringLiteral(String),
	Symbol(String),
	Funcall(Vec<Node>)
}

// is this a recursive descent parser?
pub fn parse(tokens: &[Token]) -> Result<Node, String> {
	let mut it = tokens.iter();
	let result = try!(_parse(&mut it));
	Ok(result)
}


fn next(it: &mut Iter<Token>) -> Result<Token, String> {
	let tok = it.next();
	println!("next: {:?}", tok);
	match tok {
		Some(x) => Ok(x.clone()), // FIXME
		None => Err(format!("unexpected end of input"))
	}
}

fn peek(it: &mut Iter<Token>, idx: usize) -> Result<Token, String> {
	let tok = it.idx(idx);
	println!("peek: {:?}", tok);
	match tok {
		Some(x) => Ok(x.clone()),
		None => Err(format!("unexpected end of input"))
	}
}

fn _parse(it: &mut Iter<Token>) -> Result<Node, String> {
	let x = { try!(next(it)) };
	println!("_parse got {:?}", x);
	match x {
		Token::OpenParen => { Ok(try!(parse_list(it))) },
		Token::Symbol(s) => Ok(Node::Symbol(s)),
		Token::Number(n) => Ok(try!(parse_number(n.as_slice()))),
		_ => Ok(Node::Nothing)
	}
}

fn parse_list(it: &mut Iter<Token>) -> Result<Node, String> {
	let mut list: Vec<Node> = vec!();

	loop {
		// do not consume yet, leave that to _parse()
		let x = { try!(peek(it, 0)) };
		println!("parse_list peeked {:?}", x);
		match x {
			Token::Nil => { try!(next(it)); },
			Token::CloseParen => {
				// consume the paren
				try!(next(it));
				return Ok(Node::List(list));
			},
			_ => {
				let node = try!(_parse(it));
				println!("parse_list received from _parse: {:?}", node);
				list.push(node);
			}
		}
	}
}

fn parse_number(num: &str) -> Result<Node, String> {
	let i:Option<i64> = num.parse();
	if i.is_some() {
		return Ok(Node::IntLiteral(i.unwrap()));
	}
	let f:Option<f64> = num.parse();
	if f.is_some() {
		return Ok(Node::FloatLiteral(f.unwrap()));
	}
	Err(format!("couldn't parse number: {}", num))
}
