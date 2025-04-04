use super::{Decode, Encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum Difficulty {
	Peaceful,
	Easy,
	Normal,
	Hard,
}
