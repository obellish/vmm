use super::Signed;

pub trait Unsigned: Sized {
	type Signed: Signed;

	fn to_signed(self) -> Self::Signed;

	fn from_signed(signed: Self::Signed) -> Self;

	fn try_into_signed(self) -> Option<Self::Signed>
	where
		Self: TryInto<Self::Signed>,
	{
		self.try_into().ok()
	}

	fn try_from_signed(signed: Self::Signed) -> Option<Self>
	where
		Self::Signed: TryInto<Self>,
	{
		signed.try_into().ok()
	}

	fn to_u8(self) -> u8
	where
		Self: Into<u8>,
	{
		self.into()
	}

	fn to_u16(self) -> u16
	where
		Self: Into<u16>,
	{
		self.into()
	}

	fn to_u32(self) -> u32
	where
		Self: Into<u32>,
	{
		self.into()
	}

	fn to_u64(self) -> u64
	where
		Self: Into<u64>,
	{
		self.into()
	}

	fn to_u128(self) -> u128
	where
		Self: Into<u128>,
	{
		self.into()
	}

	fn to_usize(self) -> usize
	where
		Self: Into<usize>,
	{
		self.into()
	}
}

impl Unsigned for u8 {
	type Signed = i8;

	fn to_signed(self) -> Self::Signed {
		self as _
	}

	fn from_signed(signed: Self::Signed) -> Self {
		signed as _
	}
}
