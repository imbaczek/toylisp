use std::io;

#[deriving(Show,Clone,PartialEq)]
pub enum Token {
	Nil, // to avoid Result<Option<Token>, ...>
	OpenParen,
	CloseParen,
	Symbol(String),
	Number(String),
	String(String)
}

enum State {
	Normal,
	Symbol,
	String,
	Escape,
	Number,
	Comment
}

pub struct Lexer<'a> {
	input: &'a str,
	word: String,
	tokens: Vec<Token>,
	state: State,
	statelen: u32
}

// horrible, horrible code below
impl<'a> Lexer<'a> {
	pub fn new(s: &str) -> Lexer {
		Lexer {
			input: s,
			word: String::with_capacity(s.len()),
			tokens: vec!(),
			state: State::Normal,
			statelen: 0
		}
	}

	pub fn scan(&'a mut self) -> Result<&'a [Token], String> {
		for c in self.input.chars() {
			let r = match self.state {
				State::Normal => self.scan_normal(c),
				State::Symbol => self.scan_symbol(c),
				State::String => self.scan_string(c),
				State::Escape => self.scan_escape(c),
				State::Number => self.scan_number(c),
				State::Comment => self.scan_comment(c)
			};
			match r {
				Ok(Token::Nil) => {},
				Ok(x) => self.tokens.push(x),
				Err(x) => return Err(x)
			}

		}
		Ok(self.tokens.as_slice())
	}

	fn scan_normal(&mut self, c: char) -> Result<Token, String> {
		use self::Token::Nil;
		match c {
			';' => { self.state = State::Comment; Ok(Nil) },
			'"' => { self.state = State::String; Ok(Nil) },
			'(' => Ok(Token::OpenParen),
			')' => Ok(Token::CloseParen),
			'[' | ']' | '.' |
			'+' | '-' | '*' | '/' | '&' | '|' | '^' | '$' | '!' |
			':' | '?' | '\'' |
			'a'...'z' | 'A'...'Z' => {
				self.word.push(c);
				self.state = State::Symbol;
				Ok(Nil)
			},
			'0' ... '9' => {
				self.word.push(c);
				self.state = State::Number;
				Ok(Nil)
			},
			' ' | '\t' | '\r' | '\n' | '\x1f' => Ok(Nil),  // whitespace
			_ => Err(format!("invalid input in normal state: {} ({:x})", c, c as int))
		}
	}

	fn scan_symbol(&mut self, c: char) -> Result<Token, String> {
		use self::Token::Nil;
		match c {
			';' => { self.push_token(); self.state = State::Comment; Ok(Nil) },
			'"' => { self.push_token(); self.state = State::String; Ok(Nil) },
			'(' => { self.push_token(); Ok(Token::OpenParen) },
			')' => { self.push_token(); Ok(Token::CloseParen) },
			'[' | ']' | '.' |
			'+' | '-' | '*' | '/' | '&' | '|' | '^' | '$' | 
			'!' | ':' | '?' |
			'a'...'z' | 'A'...'Z' | '0' ... '9' => {
				self.word.push(c);
				Ok(Nil)
			},  // symbols and numbers
			' ' | '\t' | '\r' | '\n' | '\x1f' => {
				self.push_token();
				self.state = State::Normal;
				Ok(Nil)
			},  // whitespace
			_ => Err(format!("invalid input in symbol state: {} ({:x})", c, c as int))
		}
	}

	fn scan_string(&mut self, c: char) -> Result<Token, String> {
		Err(String::from_str("scan_string not implemented"))
	}

	fn scan_escape(&mut self, c: char) -> Result<Token, String> {
		Err(String::from_str("scan_escape not implemented"))
	}

	fn scan_number(&mut self, c: char) -> Result<Token, String> {
		use self::Token::Nil;
		match c {
			';' => { self.push_token(); self.state = State::Comment; Ok(Nil) },
			'"' => { self.push_token(); self.state = State::String; Ok(Nil) },
			'(' => { self.push_token(); Ok(Token::OpenParen) },
			')' => { self.push_token(); Ok(Token::CloseParen) },
			'0' ... '9' | '+' | '-' | 'e' | 'E' | 'u' | 'i' | 'f' | '.' => {
				self.word.push(c);
				Ok(Nil)
			},
			' ' | '\t' | '\r' | '\n' | '\x1f' => {
				self.push_token();
				self.state = State::Normal;
				Ok(Nil)
			},  // whitespace
			_ => Err(format!("invalid input in number state: {} ({:x})", c, c as int))
		}
	}

	fn scan_comment(&mut self, c: char) -> Result<Token, String> {
		use self::Token::Nil;
		match c {
			'\n' => self.state = State::Normal,
			_ => {}
		}
		Ok(Nil)
	}

	fn push_token(&mut self) {
		use self::State::{Normal,Symbol,Number,String};
		if self.word.len() == 0 {
			return
		}
		match self.state {
			Normal => panic!("can't push Normal state"),
			Symbol => self.tokens.push(Token::Symbol(self.word.clone())),
			String => self.tokens.push(Token::String(self.word.clone())),
			Number => self.tokens.push(Token::Number(self.word.clone())),
			_ => panic!("invalid push state")
		}
		self.word.clear();
	}
}
