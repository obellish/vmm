use super::Signed;

pub trait Unsigned: Sized {
	type Signed: Signed;

	fn to_signed(self) -> Self::Signed;

	fn from_signed(signed: Self::Signed) -> Self;

	fn try_into_signed(self) -> Option<Self::Signed>;

	fn try_from_signed(signed: Self::Signed) -> Option<Self>;

	fn to_u8(self) -> u8;

	fn to_u16(self) -> u16;

	fn to_u32(self) -> u32;

	fn to_u64(self) -> u64;

	fn to_u128(self) -> u128;

	fn to_usize(self) -> usize;
}

impl Unsigned for u8 {
	type Signed = i8;

	fn to_signed(self) -> Self::Signed {
		self as _
	}

	fn from_signed(signed: Self::Signed) -> Self {
		signed as _
	}

	fn try_from_signed(signed: Self::Signed) -> Option<Self> {
		signed.try_into().ok()
	}

	fn try_into_signed(self) -> Option<Self::Signed> {
		self.try_into().ok()
	}

	fn to_u8(self) -> u8 {
		self
	}

	fn to_u16(self) -> u16 {
		self.into()
	}

	fn to_u32(self) -> u32 {
		self.into()
	}

	fn to_u64(self) -> u64 {
		self.into()
	}

	fn to_u128(self) -> u128 {
		self.into()
	}

	fn to_usize(self) -> usize {
		self.into()
	}
}

impl Unsigned for u16 {
	type Signed = i16;

	fn to_signed(self) -> Self::Signed {
		self as _
	}

	fn from_signed(signed: Self::Signed) -> Self {
		signed as _
	}

	fn try_from_signed(signed: Self::Signed) -> Option<Self> {
		signed.try_into().ok()
	}

	fn try_into_signed(self) -> Option<Self::Signed> {
		self.try_into().ok()
	}

	fn to_u8(self) -> u8 {
		self as _
	}

	fn to_u16(self) -> u16 {
		self
	}

	fn to_u32(self) -> u32 {
		self.into()
	}

	fn to_u64(self) -> u64 {
		self.into()
	}

	fn to_u128(self) -> u128 {
		self.into()
	}

	fn to_usize(self) -> usize {
		self.into()
	}
}

impl Unsigned for u32 {
	type Signed = i32;

	fn to_signed(self) -> Self::Signed {
		self as _
	}

	fn from_signed(signed: Self::Signed) -> Self {
		signed as _
	}

	fn try_from_signed(signed: Self::Signed) -> Option<Self> {
		signed.try_into().ok()
	}

	fn try_into_signed(self) -> Option<Self::Signed> {
		self.try_into().ok()
	}

	fn to_u8(self) -> u8 {
		self as _
	}

	fn to_u16(self) -> u16 {
		self as _
	}

	fn to_u32(self) -> u32 {
		self
	}

	fn to_u64(self) -> u64 {
		self.into()
	}

	fn to_u128(self) -> u128 {
		self.into()
	}

	fn to_usize(self) -> usize {
		self as _
	}
}

impl Unsigned for u64 {
	type Signed = i64;

	fn to_signed(self) -> Self::Signed {
		self as _
	}

	fn from_signed(signed: Self::Signed) -> Self {
		signed as _
	}

	fn try_from_signed(signed: Self::Signed) -> Option<Self> {
		signed.try_into().ok()
	}

	fn try_into_signed(self) -> Option<Self::Signed> {
		self.try_into().ok()
	}

	fn to_u8(self) -> u8 {
		self as _
	}

	fn to_u16(self) -> u16 {
		self as _
	}

	fn to_u32(self) -> u32 {
		self as _
	}

	fn to_u64(self) -> u64 {
		self
	}

	fn to_u128(self) -> u128 {
		self.into()
	}

	fn to_usize(self) -> usize {
		self as _
	}
}

impl Unsigned for u128 {
	type Signed = i128;

	fn to_signed(self) -> Self::Signed {
		self as _
	}

	fn from_signed(signed: Self::Signed) -> Self {
		signed as _
	}

	fn try_from_signed(signed: Self::Signed) -> Option<Self> {
		signed.try_into().ok()
	}

	fn try_into_signed(self) -> Option<Self::Signed> {
		self.try_into().ok()
	}

	fn to_u8(self) -> u8 {
		self as _
	}

	fn to_u16(self) -> u16 {
		self as _
	}

	fn to_u32(self) -> u32 {
		self as _
	}

	fn to_u64(self) -> u64 {
		self as _
	}

	fn to_u128(self) -> u128 {
		self
	}

	fn to_usize(self) -> usize {
		self as _
	}
}

impl Unsigned for usize {
	type Signed = isize;

	fn to_signed(self) -> Self::Signed {
		self as _
	}

	fn from_signed(signed: Self::Signed) -> Self {
		signed as _
	}

	fn try_from_signed(signed: Self::Signed) -> Option<Self> {
		signed.try_into().ok()
	}

	fn try_into_signed(self) -> Option<Self::Signed> {
		self.try_into().ok()
	}

	fn to_u8(self) -> u8 {
		self as _
	}

	fn to_u16(self) -> u16 {
		self as _
	}

	fn to_u32(self) -> u32 {
		self as _
	}

	fn to_u64(self) -> u64 {
		self as _
	}

	fn to_u128(self) -> u128 {
		self as _
	}

	fn to_usize(self) -> usize {
		self
	}
}
