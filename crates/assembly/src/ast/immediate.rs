use core::fmt::{Debug, Display, Formatter, Result as FmtResult};

use vmm_core::{
	Felt,
	prettier::{Document, PrettyPrint, text},
};

use crate::{SourceSpan, Span, Spanned, ast::Ident};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Immediate<T> {
	Value(Span<T>),
	Constant(Ident),
}

impl<T> Immediate<T> {
	pub const fn is_literal(&self) -> bool {
		matches!(self, Self::Value(_))
	}

	pub fn map<U>(self, map: impl FnMut(T) -> U) -> Immediate<U> {
		match self {
			Self::Constant(id) => Immediate::Constant(id),
			Self::Value(value) => Immediate::Value(value.map(map)),
		}
	}
}

impl<T: Copy> Immediate<T> {
	pub fn expect_value(&self) -> T {
		match self {
			Self::Value(value) => value.into_inner(),
			Self::Constant(name) => panic!("tried to unwrap unresolved constant: '{name}'"),
		}
	}

	pub fn expect_spanned_value(&self) -> Span<T> {
		match self {
			Self::Value(value) => *value,
			Self::Constant(name) => panic!("tried to unwrap unresolved constant: '{name}'"),
		}
	}
}

impl<T: Display> Display for Immediate<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Value(value) => Display::fmt(&value, f),
			Self::Constant(ident) => Display::fmt(&ident, f),
		}
	}
}

impl<T> From<T> for Immediate<T> {
	fn from(value: T) -> Self {
		Self::from(Span::unknown(value))
	}
}

impl<T> From<Span<T>> for Immediate<T> {
	fn from(value: Span<T>) -> Self {
		Self::Value(value)
	}
}

impl<T: PartialEq> PartialEq<T> for Immediate<T> {
	fn eq(&self, other: &T) -> bool {
		match self {
			Self::Value(l) => l == other,
			Self::Constant(_) => false,
		}
	}
}

impl<T: PrettyPrint> PrettyPrint for Immediate<T> {
	fn render(&self) -> Document {
		match self {
			Self::Value(value) => value.render(),
			Self::Constant(ident) => text(ident),
		}
	}
}

impl<T> Spanned for Immediate<T> {
	fn span(&self) -> SourceSpan {
		match self {
			Self::Value(value) => value.span(),
			Self::Constant(ident) => ident.span(),
		}
	}
}

pub type ImmU8 = Immediate<u8>;

pub type ImmU16 = Immediate<u16>;

pub type ImmU32 = Immediate<u32>;

pub type ImmFelt = Immediate<Felt>;

pub type ErrorCode = Immediate<u32>;
