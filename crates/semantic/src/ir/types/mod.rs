mod integer;

use alloc::{boxed::Box, vec::Vec};
use core::{
	fmt::{Display, Formatter, Result as FmtResult, Write as _},
	mem,
	ops::Deref,
};

use serde::{Deserialize, Serialize};

pub use self::integer::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Type(Box<TypeKind>);

impl Type {
	#[must_use]
	pub fn new(kind: TypeKind) -> Self {
		Self(Box::new(kind))
	}

	#[must_use]
	pub fn i32() -> Self {
		Self::new(TypeKind::Int(IntegerType::Int32))
	}

	#[must_use]
	pub fn unit() -> Self {
		Self::new(TypeKind::Unit)
	}

	#[must_use]
	pub fn array(base: Self, len: usize) -> Self {
		assert_ne!(len, 0, "cannot create zero-sized array");

		Self::new(TypeKind::Array(base, len))
	}

	#[must_use]
	pub fn ptr(base: Self) -> Self {
		Self::new(TypeKind::Ptr(base))
	}

	pub fn function(params: impl IntoIterator<Item = Self>, ret: Self) -> Self {
		Self::new(TypeKind::Function(params.into_iter().collect(), ret))
	}

	#[must_use]
	pub fn into_ptr(self) -> Self {
		Self::ptr(self)
	}

	#[must_use]
	pub fn into_array(self, len: usize) -> Self {
		Self::array(self, len)
	}
}

impl Deref for Type {
	type Target = TypeKind;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Display for Type {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&*self.0, f)
	}
}

impl From<TypeKind> for Type {
	fn from(value: TypeKind) -> Self {
		Self::new(value)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeKind {
	Int(IntegerType),
	Unit,
	Array(Type, usize),
	Ptr(Type),
	Function(Vec<Type>, Type),
}

impl TypeKind {
	#[must_use]
	pub const fn is_int(&self) -> bool {
		matches!(self, Self::Int(..))
	}

	#[must_use]
	pub const fn is_unit(&self) -> bool {
		matches!(self, Self::Unit)
	}

	#[must_use]
	pub fn size(&self) -> usize {
		match self {
			Self::Int(i) => i.size(),
			Self::Unit => 0,
			Self::Array(ty, len) => ty.size() * len,
			Self::Ptr(..) | Self::Function(..) => mem::size_of::<*const ()>(),
		}
	}
}

impl Display for TypeKind {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Int(i) => Display::fmt(&i, f)?,
			Self::Unit => f.write_str("unit")?,
			Self::Array(t, len) => {
				f.write_char('[')?;

				Display::fmt(&t, f)?;
				f.write_char(';')?;
				Display::fmt(&len, f)?;
				f.write_char(']')?;
			}
			Self::Ptr(t) => {
				f.write_char('*')?;
				Display::fmt(&t, f)?;
			}
			Self::Function(params, ret) => {
				f.write_char('(')?;
				let mut first = true;

				for param in params {
					if !first {
						f.write_str(", ")?;
					}

					Display::fmt(&param, f)?;
					first = false;
				}

				f.write_char(')')?;
				if !ret.is_unit() {
					f.write_str(": ")?;
					Display::fmt(&ret, f)?;
				}
			}
		}

		Ok(())
	}
}

impl From<Type> for TypeKind {
	fn from(value: Type) -> Self {
		*value.0
	}
}

#[cfg(test)]
mod tests {
	use alloc::string::ToString as _;
	use core::mem;

	use super::Type;

	#[test]
	fn fmt() {
		assert_eq!(Type::i32().to_string(), "i32");
		assert_eq!(Type::unit().to_string(), "unit");
		assert_eq!(Type::i32().into_array(10).to_string(), "[i32;10]");
		assert_eq!(
			Type::i32().into_array(2).into_array(3).to_string(),
			"[[i32;2];3]"
		);
		assert_eq!(Type::i32().into_ptr().into_ptr().to_string(), "**i32");
		assert_eq!(Type::function([], Type::unit()).to_string(), "()");
		assert_eq!(
			Type::function([Type::i32()], Type::unit()).to_string(),
			"(i32)"
		);
		assert_eq!(
			Type::function([Type::i32(), Type::i32()], Type::i32()).to_string(),
			"(i32, i32): i32"
		);
	}

	#[test]
	fn size() {
		assert_eq!(Type::i32().size(), 4);
		assert_eq!(Type::unit().size(), 0);
		assert_eq!(Type::i32().into_array(5).size(), 20);
		assert_eq!(Type::i32().into_array(6).into_array(5).size(), 120);
		assert_eq!(
			Type::i32().into_array(5).into_ptr().size(),
			mem::size_of::<*const ()>()
		);
		assert_eq!(
			Type::i32().into_ptr().into_array(5).size(),
			mem::size_of::<*const ()>() * 5
		);
		assert_eq!(
			Type::function([Type::i32(), Type::i32()], Type::unit()).size(),
			mem::size_of::<*const ()>()
		);
	}
}
