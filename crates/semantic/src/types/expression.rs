use alloc::{boxed::Box, string::String};
use core::fmt::{Display, Formatter, Result as FmtResult};

use serde::{Deserialize, Serialize};

use super::{ExtendedExpression, FunctionCall, SemanticContextInstruction, Type, ValueName};
use crate::{ExpressionOperations, PrimitiveValue};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expression {
	pub value: ExpressionValue,
	pub operation: Option<(ExpressionOperations, Box<Self>)>,
}

impl Display for Expression {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.value, f)
	}
}

impl<I: SemanticContextInstruction, E> From<crate::ast::Expression<'_, I, E>> for Expression
where
	E: ExtendedExpression<I>,
{
	fn from(value: crate::ast::Expression<'_, I, E>) -> Self {
		Self {
			value: value.value.into(),
			operation: value
				.operation
				.map(|(op, expr)| (op, Box::new((*expr).into()))),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ExtendedExpressionValue(String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionResult {
	pub ty: Type,
	pub value: ExpressionResultValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpressionStructValue {
	pub name: ValueName,
	pub attribute: ValueName,
}

impl Display for ExpressionStructValue {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.name, f)
	}
}

impl From<crate::ast::ExpressionStructValue<'_>> for ExpressionStructValue {
	fn from(value: crate::ast::ExpressionStructValue<'_>) -> Self {
		Self {
			name: value.name.into(),
			attribute: value.attribute.into(),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum ExpressionResultValue {
	Primitive(PrimitiveValue),
	Register(u64),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum ExpressionValue {
	ValueName(ValueName),
	Primitive(PrimitiveValue),
	Struct(ExpressionStructValue),
	FunctionCall(FunctionCall),
	Expression(Box<Expression>),
	ExtendedExpression(ExtendedExpressionValue),
}

impl Display for ExpressionValue {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::ValueName(v) => Display::fmt(&v, f),
			Self::Primitive(v) => Display::fmt(&v, f),
			Self::Struct(s) => Display::fmt(&s, f),
			Self::FunctionCall(fn_call) => Display::fmt(&fn_call, f),
			Self::Expression(e) => Display::fmt(&e, f),
			Self::ExtendedExpression(e) => f.write_str(&e.0),
		}
	}
}

impl<I: SemanticContextInstruction, E> From<crate::ast::ExpressionValue<'_, I, E>>
	for ExpressionValue
where
	E: ExtendedExpression<I>,
{
	fn from(value: crate::ast::ExpressionValue<'_, I, E>) -> Self {
		match value {
			crate::ast::ExpressionValue::Marker(..) => unreachable!(),
			crate::ast::ExpressionValue::ValueName(n) => Self::ValueName(n.into()),
			crate::ast::ExpressionValue::ExtendedExpression(e) => {
				Self::ExtendedExpression(ExtendedExpressionValue(alloc::format!("{e:?}")))
			}
			crate::ast::ExpressionValue::FunctionCall(f) => Self::FunctionCall(f.into()),
			crate::ast::ExpressionValue::Primitive(v) => Self::Primitive(v),
			crate::ast::ExpressionValue::Struct(s) => Self::Struct(s.into()),
			crate::ast::ExpressionValue::Expression(e) => Self::Expression(Box::new((*e).into())),
		}
	}
}
