use logos::Logos;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Logos)]
pub enum OpCode {
	#[token("<")]
	MoveLeft,
	#[token(">")]
	MoveRight,
	#[token("+")]
	Increment,
	#[token("-")]
	Decrement,
	#[token(",")]
	Input,
	#[token(".")]
	Output,
	#[token("]")]
	JumpLeft,
	#[token("[")]
	JumpRight,
}
