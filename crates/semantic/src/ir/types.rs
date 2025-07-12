use std::{
	fmt::{Debug, Display, Formatter, Result as FmtResult, Write as _},
	hash::{Hash, Hasher},
	mem,
	ops::Deref,
};

#[derive(Clone, Eq)]
#[repr(transparent)]
pub struct Type(Box<TypeKind>);

impl Type {
	#[must_use]
	pub fn new(type_kind: TypeKind) -> Self {
		Self(Box::new(type_kind))
	}

	#[must_use]
	pub fn i32() -> Self {
		Self::new(TypeKind::Int32)
	}

	#[must_use]
	pub fn unit() -> Self {
		Self::new(TypeKind::Unit)
	}

	#[must_use]
	pub fn array(base: Self, len: usize) -> Self {
		assert_ne!(len, 0, "can not make zero sized array");
		Self::new(TypeKind::Array(base, len))
	}

	#[must_use]
	pub fn ptr(base: Self) -> Self {
		Self::new(TypeKind::Pointer(base))
	}

	#[must_use]
	pub fn func(params: impl IntoIterator<Item = Self>, ret: Self) -> Self {
		Self::new(TypeKind::Function(params.into_iter().collect(), ret))
	}

	#[must_use]
	pub fn kind(&self) -> &TypeKind {
		&self.0
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

impl Debug for Type {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self, f)
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
		Display::fmt(&self.0, f)
	}
}

impl Hash for Type {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.hash(state);
	}
}

impl PartialEq for Type {
	fn eq(&self, other: &Self) -> bool {
		PartialEq::eq(&*self.0, &*other.0)
	}
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
	Int32,
	Unit,
	Array(Type, usize),
	Pointer(Type),
	Function(Vec<Type>, Type),
}

impl TypeKind {
	#[must_use]
	pub const fn is_unit(&self) -> bool {
		matches!(self, Self::Unit)
	}

	#[must_use]
	pub const fn is_i32(&self) -> bool {
		matches!(self, Self::Int32)
	}

	#[must_use]
	pub fn size(&self) -> usize {
		match self {
			Self::Int32 => 4,
			Self::Unit => 0,
			Self::Array(ty, len) => ty.size() * len,
			Self::Pointer(..) | Self::Function(..) => mem::size_of::<*const ()>(),
		}
	}
}

impl Debug for TypeKind {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self, f)
	}
}

impl Display for TypeKind {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Int32 => f.write_str("i32")?,
			Self::Unit => f.write_str("unit")?,
			Self::Array(t, len) => {
				f.write_char('[')?;
				Display::fmt(&t, f)?;
				f.write_str("; ")?;
				Display::fmt(&len, f)?;
				f.write_char(']')?;
			}
			Self::Pointer(t) => {
				f.write_char('*')?;
				Display::fmt(&t, f)?;
			}
			Self::Function(params, ret) => {
				let mut first = true;
				f.write_char('(')?;
				for param in params {
					if !first {
						f.write_str(", ")?;
					}

					Display::fmt(&param, f)?;
					first = false;
				}

				if ret.is_unit() {
					f.write_char(')')?;
				} else {
					f.write_str("): ")?;
					Display::fmt(&ret, f)?;
				}
			}
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use std::mem::size_of;

	use super::Type;

	const SIZE_OF_PTR: usize = size_of::<*const ()>();

	#[test]
	fn fmt() {
		assert_eq!(Type::i32().to_string(), "i32");
		assert_eq!(Type::unit().to_string(), "unit");
		assert_eq!(Type::array(Type::i32(), 10).to_string(), "[i32; 10]");
		assert_eq!(
			Type::array(Type::array(Type::i32(), 2), 3).to_string(),
			"[[i32; 2]; 3]"
		);
		assert_eq!(Type::ptr(Type::ptr(Type::i32())).to_string(), "**i32");
		assert_eq!(Type::func(Vec::new(), Type::unit()).to_string(), "()");

		assert_eq!(Type::func([Type::i32()], Type::unit()).to_string(), "(i32)");
		assert_eq!(
			Type::func([Type::i32(), Type::i32()], Type::i32()).to_string(),
			"(i32, i32): i32"
		);
	}

	#[test]
	fn size() {
		assert_eq!(Type::i32().size(), 4);
		assert_eq!(Type::unit().size(), 0);
		assert_eq!(Type::array(Type::i32(), 5).size(), 20);
		assert_eq!(Type::array(Type::array(Type::i32(), 6), 5).size(), 120);
		assert_eq!(
			Type::array(Type::ptr(Type::i32()), 5).size(),
			SIZE_OF_PTR * 5
		);
		assert_eq!(
			Type::func([Type::i32(), Type::i32()], Type::unit()).size(),
			SIZE_OF_PTR
		);
	}
}
