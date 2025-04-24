use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
	pub kind: TokenType,
	pub lexeme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenType {}
