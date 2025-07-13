mod block_state;
mod condition;
mod error;
mod expression;
mod inner;
mod semantic;

use alloc::{
	borrow::Cow,
	boxed::Box,
	string::{String, ToString as _},
	vec::Vec,
};
use core::fmt::{Display, Formatter, Result as FmtResult};

use serde::{Deserialize, Serialize};

pub use self::{block_state::*, condition::*, error::*, expression::*, inner::*, semantic::*};
use super::{PrimitiveValue, ast::GetName as _};
use crate::ExpressionOperations;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ValueName(Cow<'static, str>);

impl Display for ValueName {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(&self.0)
	}
}

impl From<super::ast::ValueName<'_>> for ValueName {
	fn from(value: super::ast::ValueName<'_>) -> Self {
		Self(value.name())
	}
}

impl From<String> for ValueName {
	fn from(value: String) -> Self {
		Self(Cow::Owned(value))
	}
}

impl From<&'static str> for ValueName {
	fn from(value: &'static str) -> Self {
		Self(Cow::Borrowed(value))
	}
}

impl From<Cow<'static, str>> for ValueName {
	fn from(value: Cow<'static, str>) -> Self {
		Self(value)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ParameterName(Cow<'static, str>);

impl Display for ParameterName {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(&self.0)
	}
}

impl From<super::ast::ParameterName<'_>> for ParameterName {
	fn from(value: super::ast::ParameterName<'_>) -> Self {
		Self(Cow::Owned(value.to_string()))
	}
}

impl From<String> for ParameterName {
	fn from(value: String) -> Self {
		Self(Cow::Owned(value))
	}
}

impl From<&'static str> for ParameterName {
	fn from(value: &'static str) -> Self {
		Self(Cow::Borrowed(value))
	}
}

impl From<Cow<'static, str>> for ParameterName {
	fn from(value: Cow<'static, str>) -> Self {
		Self(value)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct InnerValueName(Cow<'static, str>);

impl Display for InnerValueName {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(&self.0)
	}
}

impl From<ValueName> for InnerValueName {
	fn from(value: ValueName) -> Self {
		Self(value.0)
	}
}

impl From<String> for InnerValueName {
	fn from(value: String) -> Self {
		Self(Cow::Owned(value))
	}
}

impl From<&'static str> for InnerValueName {
	fn from(value: &'static str) -> Self {
		Self(Cow::Borrowed(value))
	}
}

impl From<Cow<'static, str>> for InnerValueName {
	fn from(value: Cow<'static, str>) -> Self {
		Self(value)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LabelName(Cow<'static, str>);

impl Display for LabelName {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(&self.0)
	}
}

impl From<String> for LabelName {
	fn from(value: String) -> Self {
		Self(Cow::Owned(value))
	}
}

impl From<&'static str> for LabelName {
	fn from(value: &'static str) -> Self {
		Self(Cow::Borrowed(value))
	}
}

impl From<Cow<'static, str>> for LabelName {
	fn from(value: Cow<'static, str>) -> Self {
		Self(value)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct FunctionName(Cow<'static, str>);

impl Display for FunctionName {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(&self.0)
	}
}

impl From<super::ast::FunctionName<'_>> for FunctionName {
	fn from(value: super::ast::FunctionName<'_>) -> Self {
		Self(value.name())
	}
}

impl From<String> for FunctionName {
	fn from(value: String) -> Self {
		Self(Cow::Owned(value))
	}
}

impl From<&'static str> for FunctionName {
	fn from(value: &'static str) -> Self {
		Self(Cow::Borrowed(value))
	}
}

impl From<Cow<'static, str>> for FunctionName {
	fn from(value: Cow<'static, str>) -> Self {
		Self(value)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ConstantName(Cow<'static, str>);

impl Display for ConstantName {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(&self.0)
	}
}

impl From<super::ast::ConstantName<'_>> for ConstantName {
	fn from(value: super::ast::ConstantName<'_>) -> Self {
		Self(value.name())
	}
}

impl From<String> for ConstantName {
	fn from(value: String) -> Self {
		Self(Cow::Owned(value))
	}
}

impl From<&'static str> for ConstantName {
	fn from(value: &'static str) -> Self {
		Self(Cow::Borrowed(value))
	}
}

impl From<Cow<'static, str>> for ConstantName {
	fn from(value: Cow<'static, str>) -> Self {
		Self(value)
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Constant {
	pub name: ConstantName,
	pub ty: Type,
	pub value: ConstantExpression,
}

impl From<super::ast::Constant<'_>> for Constant {
	fn from(value: super::ast::Constant<'_>) -> Self {
		Self {
			name: value.name.into(),
			ty: value.ty.into(),
			value: value.value.into(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstantExpression {
	pub value: ConstantValue,
	pub operation: Option<(ExpressionOperations, Box<Self>)>,
}

impl From<super::ast::ConstantExpression<'_>> for ConstantExpression {
	fn from(value: super::ast::ConstantExpression<'_>) -> Self {
		Self {
			value: value.value.into(),
			operation: value
				.operation
				.map(|(op, expr)| (op, Box::new((*expr).into()))),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall {
	pub name: FunctionName,
	pub parameters: Vec<Expression>,
}

impl Display for FunctionCall {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.name, f)
	}
}

impl<I: SemanticContextInstruction, E> From<crate::ast::FunctionCall<'_, I, E>> for FunctionCall
where
	E: ExtendedExpression<I>,
{
	fn from(value: crate::ast::FunctionCall<'_, I, E>) -> Self {
		Self {
			name: value.name.into(),
			parameters: value.parameters.into_iter().map(Into::into).collect(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Binding {
	pub name: ValueName,
	pub value: Box<Expression>,
}

impl Display for Binding {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.name, f)
	}
}

impl<I: SemanticContextInstruction, E> From<crate::ast::Binding<'_, I, E>> for Binding
where
	E: ExtendedExpression<I>,
{
	fn from(value: crate::ast::Binding<'_, I, E>) -> Self {
		Self {
			name: value.name.into(),
			value: Box::new((*value.value).into()),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LetBinding {
	pub name: ValueName,
	pub mutable: bool,
	pub ty: Option<Type>,
	pub value: Box<Expression>,
}

impl Display for LetBinding {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.name, f)
	}
}

impl<I: SemanticContextInstruction, E> From<super::ast::LetBinding<'_, I, E>> for LetBinding
where
	E: ExtendedExpression<I>,
{
	fn from(value: super::ast::LetBinding<'_, I, E>) -> Self {
		Self {
			name: value.name.into(),
			mutable: value.mutable,
			ty: value.ty.map(Into::into),
			value: Box::new((*value.value).into()),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionStatement {
	pub name: FunctionName,
	pub parameters: Vec<FunctionParameter>,
	pub result_ty: Type,
	pub body: Vec<BodyStatement>,
}

impl<I: SemanticContextInstruction, E> From<super::ast::FunctionStatement<'_, I, E>>
	for FunctionStatement
where
	E: ExtendedExpression<I>,
{
	fn from(value: super::ast::FunctionStatement<'_, I, E>) -> Self {
		Self {
			name: value.name.into(),
			parameters: value.parameters.into_iter().map(Into::into).collect(),
			result_ty: value.result_ty.into(),
			body: value.body.into_iter().map(Into::into).collect(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionParameter {
	pub name: ParameterName,
	pub ty: Type,
}

impl Display for FunctionParameter {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.name, f)
	}
}

impl From<super::ast::FunctionParameter<'_>> for FunctionParameter {
	fn from(value: super::ast::FunctionParameter<'_>) -> Self {
		Self {
			name: value.name.into(),
			ty: value.ty.into(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Value {
	pub name: InnerValueName,
	pub ty: Type,
	pub mutable: bool,
	pub alloca: bool,
	pub malloc: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Function {
	pub name: FunctionName,
	pub ty: Type,
	pub parameters: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum ConstantValue {
	Constant(ConstantName),
	Value(PrimitiveValue),
}

impl From<super::ast::ConstantValue<'_>> for ConstantValue {
	fn from(value: super::ast::ConstantValue<'_>) -> Self {
		match value {
			super::ast::ConstantValue::Constant(v) => Self::Constant(v.into()),
			super::ast::ConstantValue::Value(v) => Self::Value(v),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BodyStatement {
	LetBinding(LetBinding),
	Binding(Binding),
	FunctionCall(FunctionCall),
	If(IfStatement),
	Loop(Vec<LoopBodyStatement>),
	Expression(Expression),
	Return(Expression),
}

impl<I: SemanticContextInstruction, E> From<super::ast::BodyStatement<'_, I, E>> for BodyStatement
where
	E: ExtendedExpression<I>,
{
	fn from(value: super::ast::BodyStatement<'_, I, E>) -> Self {
		match value {
			crate::ast::BodyStatement::Binding(b) => Self::Binding(b.into()),
			crate::ast::BodyStatement::Expression(e) => Self::Expression(e.into()),
			crate::ast::BodyStatement::FunctionCall(f) => Self::FunctionCall(f.into()),
			crate::ast::BodyStatement::If(i) => Self::If(i.into()),
			crate::ast::BodyStatement::LetBinding(l) => Self::LetBinding(l.into()),
			crate::ast::BodyStatement::Loop(l) => {
				Self::Loop(l.into_iter().map(Into::into).collect())
			}
			crate::ast::BodyStatement::Return(e) => Self::Return(e.into()),
		}
	}
}
