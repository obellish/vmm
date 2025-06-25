use crate::Offset;

pub trait GetOffset {
	fn offset_of(&self) -> Option<Offset>;
}
