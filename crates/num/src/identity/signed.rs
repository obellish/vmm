use super::Unsigned;

pub trait Signed: Sized {
	type Unsigned: Unsigned;

	fn to_unsigned(self) -> Self::Unsigned;

	fn from_unsigned(unsigned: Self::Unsigned) -> Self;

	fn try_into_unsigned(self) -> Option<Self::Unsigned>
	where
		Self::Unsigned: TryFrom<Self>,
	{
		self.try_into().ok()
	}

	fn try_from_unsigned(unsigned: Self::Unsigned) -> Option<Self>
	where
		Self: TryFrom<Self::Unsigned>,
	{
		unsigned.try_into().ok()
	}

	fn to_i8(self) -> i8
	where
		Self: Into<i8>
	{
		self.into()
	}
}

impl Signed for i8 {
	type Unsigned = u8;

    fn to_unsigned(self) -> Self::Unsigned {
        self as _
    }

    fn from_unsigned(unsigned: Self::Unsigned) -> Self {
        unsigned as _
    }
}
