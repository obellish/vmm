use super::Unsigned;

pub trait Signed: Sized {
	type Unsigned: Unsigned;

	fn to_unsigned(self) -> Self::Unsigned;

	fn from_unsigned(unsigned: Self::Unsigned) -> Self;

	fn try_into_unsigned(self) -> Option<Self::Unsigned>;

	fn try_from_unsigned(unsigned: Self::Unsigned) -> Option<Self>;

	fn to_i8(self) -> i8;

	fn to_i16(self) -> i16;

	fn to_i32(self) -> i32;

	fn to_i64(self) -> i64;

	fn to_i128(self) -> i128;

	fn to_isize(self) -> isize;
}

impl Signed for i8 {
	type Unsigned = u8;

	fn to_unsigned(self) -> Self::Unsigned {
		self as _
	}

	fn from_unsigned(unsigned: Self::Unsigned) -> Self {
		unsigned as _
	}

	fn try_from_unsigned(unsigned: Self::Unsigned) -> Option<Self> {
		unsigned.try_into().ok()
	}

	fn try_into_unsigned(self) -> Option<Self::Unsigned> {
		self.try_into().ok()
	}

	fn to_i8(self) -> i8 {
		self
	}

	fn to_i16(self) -> i16 {
		self.into()
	}

	fn to_i32(self) -> i32 {
		self.into()
	}

	fn to_i64(self) -> i64 {
		self.into()
	}

	fn to_i128(self) -> i128 {
		self.into()
	}

	fn to_isize(self) -> isize {
		self.into()
	}
}

impl Signed for i16 {
	type Unsigned = u16;

	fn to_unsigned(self) -> Self::Unsigned {
		self as _
	}

	fn from_unsigned(unsigned: Self::Unsigned) -> Self {
		unsigned as _
	}

	fn try_from_unsigned(unsigned: Self::Unsigned) -> Option<Self> {
		unsigned.try_into().ok()
	}

	fn try_into_unsigned(self) -> Option<Self::Unsigned> {
		self.try_into().ok()
	}

	fn to_i8(self) -> i8 {
		self as _
	}

	fn to_i16(self) -> i16 {
		self
	}

	fn to_i32(self) -> i32 {
		self.into()
	}

	fn to_i64(self) -> i64 {
		self.into()
	}

	fn to_i128(self) -> i128 {
		self.into()
	}

	fn to_isize(self) -> isize {
		self.into()
	}
}

impl Signed for i32 {
	type Unsigned = u32;

	fn to_unsigned(self) -> Self::Unsigned {
		self as _
	}

	fn from_unsigned(unsigned: Self::Unsigned) -> Self {
		unsigned as _
	}

	fn try_from_unsigned(unsigned: Self::Unsigned) -> Option<Self> {
		unsigned.try_into().ok()
	}

	fn try_into_unsigned(self) -> Option<Self::Unsigned> {
		self.try_into().ok()
	}

	fn to_i8(self) -> i8 {
		self as _
	}

	fn to_i16(self) -> i16 {
		self as _
	}

	fn to_i32(self) -> i32 {
		self
	}

	fn to_i64(self) -> i64 {
		self.into()
	}

	fn to_i128(self) -> i128 {
		self.into()
	}

	fn to_isize(self) -> isize {
		self as _
	}
}

impl Signed for i64 {
	type Unsigned = u64;

	fn to_unsigned(self) -> Self::Unsigned {
		self as _
	}

	fn from_unsigned(unsigned: Self::Unsigned) -> Self {
		unsigned as _
	}

	fn try_from_unsigned(unsigned: Self::Unsigned) -> Option<Self> {
		unsigned.try_into().ok()
	}

	fn try_into_unsigned(self) -> Option<Self::Unsigned> {
		self.try_into().ok()
	}

	fn to_i8(self) -> i8 {
		self as _
	}

	fn to_i16(self) -> i16 {
		self as _
	}

	fn to_i32(self) -> i32 {
		self as _
	}

	fn to_i64(self) -> i64 {
		self
	}

	fn to_i128(self) -> i128 {
		self.into()
	}

	fn to_isize(self) -> isize {
		self as _
	}
}

impl Signed for i128 {
	type Unsigned = u128;

	fn to_unsigned(self) -> Self::Unsigned {
		self as _
	}

	fn from_unsigned(unsigned: Self::Unsigned) -> Self {
		unsigned as _
	}

	fn try_from_unsigned(unsigned: Self::Unsigned) -> Option<Self> {
		unsigned.try_into().ok()
	}

	fn try_into_unsigned(self) -> Option<Self::Unsigned> {
		self.try_into().ok()
	}

	fn to_i8(self) -> i8 {
		self as _
	}

	fn to_i16(self) -> i16 {
		self as _
	}

	fn to_i32(self) -> i32 {
		self as _
	}

	fn to_i64(self) -> i64 {
		self as _
	}

	fn to_i128(self) -> i128 {
		self
	}

	fn to_isize(self) -> isize {
		self as _
	}
}

impl Signed for isize {
    type Unsigned = usize;

    fn to_unsigned(self) -> Self::Unsigned {
        self as _
    }

    fn from_unsigned(unsigned: Self::Unsigned) -> Self {
        unsigned as _
    }

    fn try_from_unsigned(unsigned: Self::Unsigned) -> Option<Self> {
        unsigned.try_into().ok()
    }

    fn try_into_unsigned(self) -> Option<Self::Unsigned> {
        self.try_into().ok()
    }

    fn to_i8(self) -> i8 {
        self as _
    }

    fn to_i16(self) -> i16 {
        self as _
    }

    fn to_i32(self) -> i32 {
        self as _
    }

    fn to_i64(self) -> i64 {
        self as _
    }

    fn to_i128(self) -> i128 {
        self as _
    }

    fn to_isize(self) -> isize {
        self
    }
}
