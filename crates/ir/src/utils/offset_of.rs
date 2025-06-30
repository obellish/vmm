use crate::Offset;

pub trait GetOffset: super::sealed::Sealed {
	fn offset_of(&self) -> Option<Offset>;
}
