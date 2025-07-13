use alloc::borrow::Cow;
use core::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use serde::{Deserialize, Serialize};

use super::ast::{GetName, Type};

pub const MAX_PRIORITY_LEVEL_FOR_EXPRESSIONS: u8 = 9;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum PrimitiveValue {
	U8(u8),
	U16(u16),
	U32(u32),
	U64(u64),
	U128(u128),
	I8(i8),
	I16(i16),
	I32(i32),
	I64(i64),
	I128(i128),
	F32(f32),
	F64(f64),
	Bool(bool),
	Char(char),
	Ptr,
	Unit,
}

impl PrimitiveValue {
	#[must_use]
	pub const fn ty(&self) -> Type<'_> {
		match self {
			Self::U8(_) => Type::Primitive(PrimitiveTypes::U8),
			Self::U16(_) => Type::Primitive(PrimitiveTypes::U16),
			Self::U32(_) => Type::Primitive(PrimitiveTypes::U32),
			Self::U64(_) => Type::Primitive(PrimitiveTypes::U64),
			Self::U128(_) => Type::Primitive(PrimitiveTypes::U128),
			Self::I8(_) => Type::Primitive(PrimitiveTypes::I8),
			Self::I16(_) => Type::Primitive(PrimitiveTypes::I16),
			Self::I32(_) => Type::Primitive(PrimitiveTypes::I32),
			Self::I64(_) => Type::Primitive(PrimitiveTypes::I64),
			Self::I128(_) => Type::Primitive(PrimitiveTypes::I128),
			Self::F32(_) => Type::Primitive(PrimitiveTypes::F32),
			Self::F64(_) => Type::Primitive(PrimitiveTypes::F64),
			Self::Char(_) => Type::Primitive(PrimitiveTypes::Char),
			Self::Bool(_) => Type::Primitive(PrimitiveTypes::Bool),
			Self::Ptr => Type::Primitive(PrimitiveTypes::Ptr),
			Self::Unit => Type::Primitive(PrimitiveTypes::Unit),
		}
	}
}

impl Display for PrimitiveValue {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::U8(v) => Display::fmt(&v, f)?,
			Self::U16(v) => Display::fmt(&v, f)?,
			Self::U32(v) => Display::fmt(&v, f)?,
			Self::U64(v) => Display::fmt(&v, f)?,
			Self::U128(v) => Display::fmt(&v, f)?,
			Self::I8(v) => Display::fmt(&v, f)?,
			Self::I16(v) => Display::fmt(&v, f)?,
			Self::I32(v) => Display::fmt(&v, f)?,
			Self::I64(v) => Display::fmt(&v, f)?,
			Self::I128(v) => Display::fmt(&v, f)?,
			Self::F32(v) => Display::fmt(&v, f)?,
			Self::F64(v) => Display::fmt(&v, f)?,
			Self::Bool(v) => Display::fmt(&v, f)?,
			Self::Char(c) => f.write_char(*c)?,
			Self::Ptr => f.write_str("ptr")?,
			Self::Unit => f.write_str("unit")?,
		}

		Ok(())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum ExpressionOperations {
	Add,
	Sub,
	Mul,
	Div,
	Shl,
	Shr,
	And,
	Or,
	Xor,
	Eq,
	NotEq,
	Gt,
	Lt,
	Ge,
	Le,
}

impl ExpressionOperations {
	#[must_use]
	pub const fn priority(self) -> u8 {
		match self {
			Self::Add => 5,
			Self::Sub => 4,
			Self::Div => 8,
			Self::Mul | Self::Shl | Self::Shr => MAX_PRIORITY_LEVEL_FOR_EXPRESSIONS,
			Self::Or | Self::Xor => 6,
			Self::And | Self::Eq | Self::NotEq | Self::Gt | Self::Lt | Self::Ge | Self::Le => 7,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum PrimitiveTypes {
	U8,
	U16,
	U32,
	U64,
	U128,
	I8,
	I16,
	I32,
	I64,
	I128,
	F32,
	F64,
	Bool,
	Char,
	Ptr,
	Unit,
}

impl Display for PrimitiveTypes {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(&self.name())
	}
}

impl GetName for PrimitiveTypes {
	fn name(&self) -> Cow<'static, str> {
		Cow::Borrowed(match self {
			Self::U8 => "u8",
			Self::U16 => "u16",
			Self::U32 => "u32",
			Self::U64 => "u64",
			Self::U128 => "u128",
			Self::I8 => "i8",
			Self::I16 => "i16",
			Self::I32 => "i32",
			Self::I64 => "i64",
			Self::I128 => "i128",
			Self::F32 => "f32",
			Self::F64 => "f64",
			Self::Bool => "bool",
			Self::Char => "char",
			Self::Ptr => "ptr",
			Self::Unit => "()",
		})
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum Condition {
	Great,
	Less,
	Eq,
	GreatEq,
	LessEq,
	NotEq,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum LogicCondition {
	And,
	Or,
}
