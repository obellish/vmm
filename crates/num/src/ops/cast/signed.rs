use super::CastTo;

pub trait CastToI8: CastTo<i8> {
	fn cast_to_i8(self) -> i8
	where
		Self: Sized,
	{
		self.cast_to()
	}
}

impl<T> CastToI8 for T where T: CastTo<i8> {}
