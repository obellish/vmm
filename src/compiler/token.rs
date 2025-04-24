use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	num::ParseFloatError,
};

use logos::Logos;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Logos)]
#[logos(error = ParseTokenError)]
#[logos(skip r"\s+")]
pub enum Token {
	#[token("(")]
	LeftParen,
	#[token(")")]
	RightParen,
	#[token("[")]
	LeftBrace,
	#[token("]")]
	RightBrace,
	#[token(",")]
	Comma,
	#[token(".")]
	Dot,
	#[token("-")]
	Minus,
	#[token("+")]
	Plus,
	#[token(";")]
	Semicolon,
	#[token("/")]
	Slash,
	#[token("*")]
	Star,
	#[token("!")]
	Bang,
	#[token("!=")]
	BangEqual,
	#[token("=")]
	Equal,
	#[token("==")]
	EqualEqual,
	#[token(">")]
	Greater,
	#[token(">=")]
	GreaterEqual,
	#[token("<")]
	Less,
	#[token("<=")]
	LessEqual,
	#[regex("[A-Za-z_]+", |lex| lex.slice().to_owned())]
	Identifier(String),
    #[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#, |lex| lex.slice().to_owned())]
	String(String),
	#[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>())]
	Number(f64),
	#[token("and")]
	And,
	#[token("class")]
	Class,
	#[token("else")]
	Else,
	#[token("false")]
	False,
	#[token("for")]
	For,
	#[token("fun")]
	Fun,
	#[token("if")]
	If,
	#[token("nil")]
	Nil,
	#[token("or")]
	Or,
	#[token("print")]
	Print,
	#[token("return")]
	Return,
	#[token("super")]
	Super,
	#[token("this")]
	This,
	#[token("true")]
	True,
	#[token("var")]
	Var,
	#[token("while")]
	While,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ParseTokenError {
	#[default]
	InvalidInput,
	ParseFloat(ParseFloatError),
}

impl Display for ParseTokenError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::InvalidInput => f.write_str("invalid input"),
			Self::ParseFloat(e) => Display::fmt(&e, f),
		}
	}
}

impl StdError for ParseTokenError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Self::ParseFloat(e) => Some(e),
			Self::InvalidInput => None,
		}
	}
}

impl From<ParseFloatError> for ParseTokenError {
	fn from(value: ParseFloatError) -> Self {
		Self::ParseFloat(value)
	}
}
