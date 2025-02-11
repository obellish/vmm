#[cfg(feature = "serde")]
mod serde;

use alloc::vec::Vec;
use core::{
	fmt::{Display, Formatter, Result as FmtResult},
	num::ParseIntError,
	ops::{Index, IndexMut, Not},
	str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Length {
	pub(crate) index: usize,
}

impl Length {
	#[must_use]
	pub const fn new(index: usize) -> Self {
		Self { index }
	}

	#[must_use]
	pub const fn index(self) -> usize {
		self.index
	}
}

impl Display for Length {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.index, f)
	}
}

impl From<usize> for Length {
	fn from(value: usize) -> Self {
		Self::new(value)
	}
}

impl From<Length> for usize {
	fn from(value: Length) -> Self {
		value.index()
	}
}

impl From<SmallLength> for Length {
	fn from(value: SmallLength) -> Self {
		Self::new(value.index())
	}
}

#[cfg(feature = "var-num")]
impl From<Length> for vmm_var_num::VarNum {
	fn from(value: Length) -> Self {
		value.index().into()
	}
}

#[cfg(feature = "var-num")]
impl From<Length> for vmm_var_num::VarUInt {
	fn from(value: Length) -> Self {
		value.index().into()
	}
}

impl FromStr for Length {
	type Err = ParseIntError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let u = s.parse()?;
		Ok(Self::new(u))
	}
}

impl<T> Index<Length> for [T] {
	type Output = T;

	fn index(&self, index: Length) -> &Self::Output {
		&self[index.index()]
	}
}

impl<T> IndexMut<Length> for [T] {
	fn index_mut(&mut self, index: Length) -> &mut Self::Output {
		&mut self[index.index()]
	}
}

impl Not for Length {
	type Output = Self;

	fn not(self) -> Self::Output {
		(!self.index()).into()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SmallLength {
	Byte(u8),
	Word(u16),
	Double(u32),
	Quad(u64),
}

impl SmallLength {
	#[must_use]
	pub const fn index(self) -> usize {
		match self {
			Self::Byte(b) => b as usize,
			Self::Word(w) => w as usize,
			Self::Double(d) => d as usize,
			Self::Quad(q) => q as usize,
		}
	}

	#[must_use]
	pub fn from_bytes(bytes: &[u8]) -> Self {
		assert!(
			!bytes.is_empty(),
			"cannot convert empty bytes to small_length"
		);

		match bytes {
			[253, rest @ ..] => {
				assert_eq!(rest.len(), 2, "insufficient bytes for u16");
				Self::Word(u16::from_be_bytes([rest[0], rest[1]]))
			}
			[254, rest @ ..] => {
				assert_eq!(rest.len(), 4, "insufficient bytes for u32");
				Self::Double(u32::from_be_bytes([rest[0], rest[1], rest[2], rest[3]]))
			}
			[255, rest @ ..] => {
				assert_eq!(rest.len(), 8, "insufficient bytes for u64");
				Self::Quad(u64::from_be_bytes([
					rest[0], rest[1], rest[2], rest[3], rest[4], rest[5], rest[6], rest[7],
				]))
			}
			[value, ..] => Self::Byte(*value),
			_ => panic!("unexpected byte format"),
		}
	}

	#[must_use]
	pub fn to_bytes(self) -> Vec<u8> {
		match self {
			Self::Byte(byte) => {
				if byte <= 252 {
					byte.to_be_bytes().to_vec()
				} else {
					let mut bytes = vec![253, 0, 0];
					let [a, b] = u16::from(byte).to_be_bytes();
					bytes[1] = a;
					bytes[2] = b;
					bytes
				}
			}
			Self::Word(word) => {
				let mut result = vec![253];
				result.extend_from_slice(&word.to_be_bytes());
				result
			}
			Self::Double(double) => {
				let mut result = vec![254];
				result.extend_from_slice(&double.to_be_bytes());
				result
			}
			Self::Quad(quad) => {
				let mut result = vec![255];
				result.extend_from_slice(&quad.to_be_bytes());
				result
			}
		}
	}
}

impl Default for SmallLength {
	fn default() -> Self {
		Self::Byte(0)
	}
}

impl Display for SmallLength {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match *self {
			Self::Byte(b) => Display::fmt(&b, f),
			Self::Word(w) => Display::fmt(&w, f),
			Self::Double(d) => Display::fmt(&d, f),
			Self::Quad(q) => Display::fmt(&q, f),
		}
	}
}

impl From<u8> for SmallLength {
	fn from(value: u8) -> Self {
		Self::Byte(value)
	}
}

impl From<u16> for SmallLength {
	fn from(value: u16) -> Self {
		if u8::try_from(value).is_ok() {
			Self::Byte(value as u8)
		} else {
			Self::Word(value)
		}
	}
}

impl From<u32> for SmallLength {
	fn from(value: u32) -> Self {
		if u16::try_from(value).is_ok() {
			(value as u16).into()
		} else {
			Self::Double(value)
		}
	}
}

impl From<u64> for SmallLength {
	fn from(value: u64) -> Self {
		if u32::try_from(value).is_ok() {
			(value as u32).into()
		} else {
			Self::Quad(value)
		}
	}
}

impl From<usize> for SmallLength {
	fn from(value: usize) -> Self {
		if u8::try_from(value).is_ok() {
			Self::Byte(value as u8)
		} else if u16::try_from(value).is_ok() {
			Self::Word(value as u16)
		} else {
			(value as u64).into()
		}
	}
}

impl From<Length> for SmallLength {
	fn from(value: Length) -> Self {
		Self::from(value.index())
	}
}

#[cfg(feature = "var-num")]
impl From<SmallLength> for vmm_var_num::VarNum {
	fn from(value: SmallLength) -> Self {
		Self::UInt(value.into())
	}
}

#[cfg(feature = "var-num")]
impl From<SmallLength> for vmm_var_num::VarUInt {
	fn from(value: SmallLength) -> Self {
		match value {
			SmallLength::Byte(b) => Self::U8(b),
			SmallLength::Word(b) => Self::U16(b),
			SmallLength::Double(b) => Self::U32(b),
			SmallLength::Quad(b) => Self::U64(b),
		}
	}
}

impl FromStr for SmallLength {
	type Err = ParseIntError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let u = s.parse::<usize>()?;
		Ok(u.into())
	}
}

impl<T> Index<SmallLength> for [T] {
	type Output = T;

	fn index(&self, index: SmallLength) -> &Self::Output {
		&self[index.index()]
	}
}

impl<T> IndexMut<SmallLength> for [T] {
	fn index_mut(&mut self, index: SmallLength) -> &mut Self::Output {
		&mut self[index.index()]
	}
}

macro_rules! impl_ops {
	($type:ident; $($name:ident: $method:ident($op:tt)),*) => {
		$(
			impl ::core::ops::$name<$type> for usize {
				type Output = $type;

				fn $method(self, rhs: $type) -> Self::Output {
					(self $op rhs.index()).into()
				}
			}

			impl ::core::ops::$name<usize> for $type {
				type Output = Self;

				fn $method(self, rhs: usize) -> Self::Output {
					(self.index() $op rhs).into()
				}
			}

			impl ::core::ops::$name for $type {
				type Output = Self;

				fn $method(self, rhs: Self) -> Self::Output {
					(self.index() $op rhs.index()).into()
				}
			}
		)*
	};
}

macro_rules! impl_ops_assign {
	($type:ident; $($name:ident: $method:ident($op:tt)),*) => {
		$(
			impl ::core::ops::$name for $type {
				fn $method(&mut self, rhs: Self) {
					*self = *self $op rhs
				}
			}

			impl ::core::ops::$name<usize> for $type {
				fn $method(&mut self, rhs: usize) {
					*self = *self $op rhs
				}
			}
		)*
	};
}

macro_rules! to_primitive {
	($($t:ty)*) => {
		$(
			#[allow(clippy::cast_lossless, clippy::checked_conversions)]
			impl ::core::convert::From<$crate::SmallLength> for $t {
				fn from(value: $crate::SmallLength) -> Self {
					match value {
						$crate::SmallLength::Byte(value) => {
							value as Self
						}
						$crate::SmallLength::Word(value) => {
							if value <= (<$t>::MAX as u16) {
								value as Self
							} else {
								panic!("word overflow: {} -> {}", value, stringify!($t))
							}
						}
						$crate::SmallLength::Double(value) => {
							if value <= (<$t>::MAX as u32) {
								value as Self
							} else {
								panic!("double overflow: {} -> {}", value, stringify!($t))
							}
						}
						$crate::SmallLength::Quad(value) => {
							if value <= (<$t>::MAX as u64) {
								value as Self
							} else {
								panic!("quad overflow: {} -> {}", value, stringify!($t))
							}
						}
					}
				}
			}
		)*
	};
}

macro_rules! from_primitive {
	($($ty:ty { $uty:ty })*) => {
		$(
			impl ::core::convert::From<$ty> for $crate::SmallLength {
				fn from(value: $ty) -> Self {
					if value < 0 {
						panic!("cannot convert negative value to length")
					}

					(value as $uty).into()
				}
			}
		)*
	};
}

macro_rules! from_length_bytes {
	($($method:ident: $impl_method:ident),*) => {
		$(
			#[must_use]
			pub fn $method(bytes: ::alloc::vec::Vec<u8>) -> Self {
				$crate::SmallLength::$impl_method(bytes).into()
			}
		)*
	};
}

macro_rules! to_length_bytes {
	($($method:ident: $impl_method:ident),*) => {
		$(
			#[must_use]
			pub fn $method(self) -> ::alloc::vec::Vec<u8> {
				let s: $crate::SmallLength = self.index().into();
				s.$impl_method()
			}
		)*
	};
}

macro_rules! from_bytes {
	($($name:ident: $from:ident),*) => {
		$(
			#[must_use]
			pub fn $name(raw: ::alloc::vec::Vec<u8>) -> Self {
				let len = raw.len();
				assert_ne!(len, 0, "cannot convert empty bytes to length");

				match len {
					1 => {
						let mut bytes = [0];
						bytes.copy_from_slice(&raw);
						Self::Byte(u8::$from(bytes))
					}
					2 => {
						let mut bytes = [0; 2];
						bytes.copy_from_slice(&raw);
						Self::Word(u16::$from(bytes))
					}
					4 => {
						let mut bytes = [0; 4];
						bytes.copy_from_slice(&raw);
						Self::Double(u32::$from(bytes))
					}
					8 => {
						let mut bytes = [0; 8];
						bytes.copy_from_slice(&raw);
						Self::Quad(u64::$from(bytes))
					}
					_ => panic!("invalid byte length: {}", len)
				}
			}
		)*
	};
}

macro_rules! to_bytes {
	($($name:ident: $to:ident),*) => {
		$(
			#[must_use]
			pub fn $name(self) -> ::alloc::vec::Vec<u8> {
				match self {
					Self::Byte(value) => {
						value.$to().to_vec()
					}
					Self::Word(value) => {
						value.$to().to_vec()
					}
					Self::Double(value) => {
						value.$to().to_vec()
					}
					Self::Quad(value) => {
						value.$to().to_vec()
					}
				}
			}
		)*
	};
}

impl Length {
	from_length_bytes! { from_le_bytes: from_le_bytes, from_be_bytes: from_be_bytes }

	to_length_bytes! { to_le_bytes: to_le_bytes, to_be_bytes: to_be_bytes }
}

impl SmallLength {
	from_bytes! { from_le_bytes: from_le_bytes, from_be_bytes: from_be_bytes }

	to_bytes! { to_le_bytes: to_le_bytes, to_be_bytes: to_be_bytes }
}

to_primitive! { usize u8 u16 u32 u64 u128 }

from_primitive! { i8{u8} i16{u16} i32{u32} i64{u64} isize{usize} }

impl_ops! {
	Length;
	Add: add(+),
	Sub: sub(-),
	Mul: mul(*),
	Div: div(/),
	Rem: rem(%),
	BitAnd: bitand(&),
	BitOr: bitor(|),
	BitXor: bitxor(^)
}

impl_ops_assign! {
	Length;
	AddAssign: add_assign(+),
	SubAssign: sub_assign(-),
	MulAssign: mul_assign(*),
	DivAssign: div_assign(/),
	RemAssign: rem_assign(%),
	BitAndAssign: bitand_assign(&),
	BitOrAssign: bitor_assign(|),
	BitXorAssign: bitxor_assign(^)
}

impl_ops! {
	SmallLength;
	Add: add(+),
	Sub: sub(-),
	Mul: mul(*),
	Div: div(/),
	Rem: rem(%),
	BitAnd: bitand(&),
	BitOr: bitor(|),
	BitXor: bitxor(^)
}

impl_ops_assign! {
	SmallLength;
	AddAssign: add_assign(+),
	SubAssign: sub_assign(-),
	MulAssign: mul_assign(*),
	DivAssign: div_assign(/),
	RemAssign: rem_assign(%),
	BitAndAssign: bitand_assign(&),
	BitOrAssign: bitor_assign(|),
	BitXorAssign: bitxor_assign(^)
}
