mod serde_impl;

use alloc::{
	borrow::{Cow, ToOwned},
	boxed::Box,
	vec::Vec,
};
use core::{
	convert::Infallible,
	fmt::{Display, Formatter, Result as FmtResult},
	marker::PhantomData,
};

use nom_locate::LocatedSpan;
use serde::{Deserialize, Serialize};

use super::{
	Condition, ExpressionOperations, LogicCondition, PrimitiveTypes, PrimitiveValue,
	types::{ExtendedExpression, SemanticContextInstruction},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Ident<'a>(LocatedSpan<&'a str>);

impl<'a> Ident<'a> {
	#[must_use]
	pub fn new(ident: &'a str) -> Self {
		Self(LocatedSpan::new(ident))
	}

	#[must_use]
	pub fn fragment(self) -> &'a str {
		self.0.fragment()
	}

	#[must_use]
	pub fn line(self) -> u32 {
		self.0.location_line()
	}

	#[must_use]
	pub fn offset(self) -> usize {
		self.0.location_offset()
	}
}

impl<'a> From<ConstantName<'a>> for Ident<'a> {
	fn from(value: ConstantName<'a>) -> Self {
		value.0
	}
}

impl<'a> From<ImportName<'a>> for Ident<'a> {
	fn from(value: ImportName<'a>) -> Self {
		value.0
	}
}

impl<'a> From<FunctionName<'a>> for Ident<'a> {
	fn from(value: FunctionName<'a>) -> Self {
		value.0
	}
}

impl<'a> From<ParameterName<'a>> for Ident<'a> {
	fn from(value: ParameterName<'a>) -> Self {
		value.0
	}
}

impl<'a> From<ValueName<'a>> for Ident<'a> {
	fn from(value: ValueName<'a>) -> Self {
		value.0
	}
}

impl Display for Ident<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.0, f)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ImportName<'a>(#[serde(borrow)] Ident<'a>);

impl<'a> ImportName<'a> {
	#[must_use]
	pub const fn new(ident: Ident<'a>) -> Self {
		Self(ident)
	}
}

impl Display for ImportName<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.0, f)
	}
}

impl<'a> From<Ident<'a>> for ImportName<'a> {
	fn from(value: Ident<'a>) -> Self {
		Self::new(value)
	}
}

impl GetName for ImportName<'_> {
	fn name(&self) -> Cow<'static, str> {
		Cow::Owned((*self.0.fragment()).to_owned())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ConstantName<'a>(#[serde(borrow)] Ident<'a>);

impl<'a> ConstantName<'a> {
	#[must_use]
	pub const fn new(ident: Ident<'a>) -> Self {
		Self(ident)
	}
}

impl Display for ConstantName<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.0, f)
	}
}

impl<'a> From<Ident<'a>> for ConstantName<'a> {
	fn from(value: Ident<'a>) -> Self {
		Self::new(value)
	}
}

impl GetLocation for ConstantName<'_> {
	fn location(&self) -> CodeLocation {
		self.0.into()
	}
}

impl GetName for ConstantName<'_> {
	fn name(&self) -> Cow<'static, str> {
		Cow::Owned((*self.0.fragment()).to_owned())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct FunctionName<'a>(#[serde(borrow)] Ident<'a>);

impl<'a> FunctionName<'a> {
	#[must_use]
	pub const fn new(ident: Ident<'a>) -> Self {
		Self(ident)
	}
}

impl Display for FunctionName<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.0, f)
	}
}

impl<'a> From<Ident<'a>> for FunctionName<'a> {
	fn from(value: Ident<'a>) -> Self {
		Self::new(value)
	}
}

impl GetLocation for FunctionName<'_> {
	fn location(&self) -> CodeLocation {
		self.0.into()
	}
}

impl GetName for FunctionName<'_> {
	fn name(&self) -> Cow<'static, str> {
		Cow::Owned((*self.0.fragment()).to_owned())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ParameterName<'a>(#[serde(borrow)] Ident<'a>);

impl<'a> ParameterName<'a> {
	#[must_use]
	pub const fn new(ident: Ident<'a>) -> Self {
		Self(ident)
	}
}

impl Display for ParameterName<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.0, f)
	}
}

impl<'a> From<Ident<'a>> for ParameterName<'a> {
	fn from(value: Ident<'a>) -> Self {
		Self::new(value)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ValueName<'a>(#[serde(borrow)] Ident<'a>);

impl<'a> ValueName<'a> {
	#[must_use]
	pub const fn new(ident: Ident<'a>) -> Self {
		Self(ident)
	}
}

impl Display for ValueName<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.0, f)
	}
}

impl GetLocation for ValueName<'_> {
	fn location(&self) -> CodeLocation {
		self.0.into()
	}
}

impl GetName for ValueName<'_> {
	fn name(&self) -> Cow<'static, str> {
		Cow::Owned((*self.0.fragment()).to_owned())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeLocation(u32, usize);

impl CodeLocation {
	#[must_use]
	pub const fn new(location: u32, offset: usize) -> Self {
		Self(location, offset)
	}

	#[must_use]
	pub const fn line(self) -> u32 {
		self.0
	}

	#[must_use]
	pub const fn offset(self) -> usize {
		self.1
	}
}

impl<'a> From<Ident<'a>> for CodeLocation {
	fn from(value: Ident<'a>) -> Self {
		Self::new(value.line(), value.offset())
	}
}

impl GetLocation for CodeLocation {
	fn location(&self) -> CodeLocation {
		*self
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpressionStructValue<'a> {
	#[serde(borrow)]
	pub name: ValueName<'a>,
	pub attribute: ValueName<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructTypes<'a> {
	#[serde(borrow)]
	pub name: Ident<'a>,
	pub attributes: Vec<StructType<'a>>,
}

impl GetLocation for StructTypes<'_> {
	fn location(&self) -> CodeLocation {
		self.name.into()
	}
}

impl GetName for StructTypes<'_> {
	fn name(&self) -> Cow<'static, str> {
		Cow::Owned((*self.name.fragment()).to_owned())
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstantExpression<'a> {
	#[serde(borrow)]
	pub value: ConstantValue<'a>,
	pub operation: Option<(ExpressionOperations, Box<Self>)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Constant<'a> {
	#[serde(borrow)]
	pub name: ConstantName<'a>,
	pub ty: Type<'a>,
	pub value: ConstantExpression<'a>,
}

impl GetLocation for Constant<'_> {
	fn location(&self) -> CodeLocation {
		self.name.location()
	}
}

impl GetName for Constant<'_> {
	fn name(&self) -> Cow<'static, str> {
		self.name.name()
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionParameter<'a> {
	#[serde(borrow)]
	pub name: ParameterName<'a>,
	pub ty: Type<'a>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionStatement<'a, I: SemanticContextInstruction, E>
where
	E: ExtendedExpression<I>,
{
	#[serde(borrow)]
	pub name: FunctionName<'a>,
	pub parameters: Vec<FunctionParameter<'a>>,
	pub result_ty: Type<'a>,
	pub body: Vec<BodyStatement<'a, I, E>>,
	marker: PhantomData<I>,
}

impl<'a, I: SemanticContextInstruction, E> FunctionStatement<'a, I, E>
where
	E: ExtendedExpression<I>,
{
	pub fn new(
		name: FunctionName<'a>,
		parameters: impl IntoIterator<Item = FunctionParameter<'a>>,
		result_ty: Type<'a>,
		body: impl IntoIterator<Item = BodyStatement<'a, I, E>>,
	) -> Self {
		Self {
			name,
			parameters: parameters.into_iter().collect(),
			result_ty,
			body: body.into_iter().collect(),
			marker: PhantomData,
		}
	}
}

impl<I: SemanticContextInstruction, E> GetLocation for FunctionStatement<'_, I, E>
where
	E: ExtendedExpression<I>,
{
	fn location(&self) -> CodeLocation {
		self.name.location()
	}
}

impl<I: SemanticContextInstruction, E> GetName for FunctionStatement<'_, I, E>
where
	E: ExtendedExpression<I>,
{
	fn name(&self) -> Cow<'static, str> {
		self.name.name()
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructType<'a> {
	#[serde(borrow)]
	pub attr_name: Ident<'a>,
	pub attr_type: Type<'a>,
}

impl GetName for StructType<'_> {
	fn name(&self) -> Cow<'static, str> {
		Cow::Owned((*self.attr_name.fragment()).to_owned())
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expression<'a, I: SemanticContextInstruction, E>
where
	E: ExtendedExpression<I>,
{
	#[serde(borrow)]
	pub value: ExpressionValue<'a, I, E>,
	pub operation: Option<(ExpressionOperations, Box<Self>)>,
}

impl<I: SemanticContextInstruction, E> GetLocation for Expression<'_, I, E>
where
	E: ExtendedExpression<I>,
{
	fn location(&self) -> CodeLocation {
		CodeLocation::new(1, 0)
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub struct LetBinding<'a, I: SemanticContextInstruction, E>
where
	E: ExtendedExpression<I>,
{
	#[serde(borrow)]
	pub name: ValueName<'a>,
	pub mutable: bool,
	pub ty: Option<Type<'a>>,
	pub value: Box<Expression<'a, I, E>>,
}

impl<I: SemanticContextInstruction, E> GetLocation for LetBinding<'_, I, E>
where
	E: ExtendedExpression<I>,
{
	fn location(&self) -> CodeLocation {
		self.name.location()
	}
}

impl<I: SemanticContextInstruction, E> GetName for LetBinding<'_, I, E>
where
	E: ExtendedExpression<I>,
{
	fn name(&self) -> Cow<'static, str> {
		self.name.name()
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Binding<'a, I: SemanticContextInstruction, E>
where
	E: ExtendedExpression<I>,
{
	#[serde(borrow)]
	pub name: ValueName<'a>,
	pub value: Box<Expression<'a, I, E>>,
}

impl<I: SemanticContextInstruction, E> GetLocation for Binding<'_, I, E>
where
	E: ExtendedExpression<I>,
{
	fn location(&self) -> CodeLocation {
		self.name.location()
	}
}

impl<I: SemanticContextInstruction, E> GetName for Binding<'_, I, E>
where
	E: ExtendedExpression<I>,
{
	fn name(&self) -> Cow<'static, str> {
		self.name.name()
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall<'a, I: SemanticContextInstruction, E>
where
	E: ExtendedExpression<I>,
{
	#[serde(borrow)]
	pub name: FunctionName<'a>,
	pub parameters: Vec<Expression<'a, I, E>>,
}

impl<I: SemanticContextInstruction, E> GetLocation for FunctionCall<'_, I, E>
where
	E: ExtendedExpression<I>,
{
	fn location(&self) -> CodeLocation {
		self.name.location()
	}
}

impl<I: SemanticContextInstruction, E> GetName for FunctionCall<'_, I, E>
where
	E: ExtendedExpression<I>,
{
	fn name(&self) -> Cow<'static, str> {
		self.name.name()
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionCondition<'a, I: SemanticContextInstruction, E>
where
	E: ExtendedExpression<I>,
{
	#[serde(borrow)]
	pub left: Expression<'a, I, E>,
	pub condition: Condition,
	pub right: Expression<'a, I, E>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionLogicCondition<'a, I: SemanticContextInstruction, E>
where
	E: ExtendedExpression<I>,
{
	#[serde(borrow)]
	pub left: ExpressionCondition<'a, I, E>,
	pub right: Option<(LogicCondition, Box<Self>)>,
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfStatement<'a, I: SemanticContextInstruction, E>
where
	E: ExtendedExpression<I>,
{
	#[serde(borrow)]
	pub condition: IfCondition<'a, I, E>,
	pub body: IfBodyStatements<'a, I, E>,
	pub else_statement: Option<IfBodyStatements<'a, I, E>>,
	pub else_if_statement: Option<Box<Self>>,
}

impl<I: SemanticContextInstruction, E> GetLocation for IfStatement<'_, I, E>
where
	E: ExtendedExpression<I>,
{
	fn location(&self) -> CodeLocation {
		CodeLocation::new(1, 0)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type<'a> {
	Primitive(PrimitiveTypes),
	#[serde(borrow)]
	Struct(StructTypes<'a>),
	Array(Box<Self>, u32),
}

impl GetName for Type<'_> {
	fn name(&self) -> Cow<'static, str> {
		match self {
			Self::Primitive(p) => p.name(),
			Self::Struct(s) => Cow::Owned((*s.name.fragment()).to_owned()),
			Self::Array(array_type, size) => {
				Cow::Owned(alloc::format!("[{:?};{size}]", array_type.name()))
			}
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum ConstantValue<'a> {
	#[serde(borrow)]
	Constant(ConstantName<'a>),
	Value(PrimitiveValue),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum ExpressionValue<'a, I: SemanticContextInstruction, E>
where
	E: ExtendedExpression<I>,
{
	#[serde(borrow)]
	ValueName(ValueName<'a>),
	Primitive(PrimitiveValue),
	FunctionCall(FunctionCall<'a, I, E>),
	Struct(ExpressionStructValue<'a>),
	Expression(Box<Expression<'a, I, E>>),
	ExtendedExpression(Box<E>),
	#[serde(skip)]
	Marker(Infallible, PhantomData<I>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum BodyStatement<'a, I: SemanticContextInstruction, E>
where
	E: ExtendedExpression<I>,
{
	#[serde(borrow)]
	LetBinding(LetBinding<'a, I, E>),
	Binding(Binding<'a, I, E>),
	FunctionCall(FunctionCall<'a, I, E>),
	If(IfStatement<'a, I, E>),
	Loop(Vec<LoopBodyStatement<'a, I, E>>),
	Expression(Expression<'a, I, E>),
	Return(Expression<'a, I, E>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum IfCondition<'a, I: SemanticContextInstruction, E>
where
	E: ExtendedExpression<I>,
{
	#[serde(borrow)]
	Single(Expression<'a, I, E>),
	Logic(ExpressionLogicCondition<'a, I, E>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum IfBodyStatement<'a, I: SemanticContextInstruction, E>
where
	E: ExtendedExpression<I>,
{
	#[serde(borrow)]
	LetBinding(LetBinding<'a, I, E>),
	Binding(Binding<'a, I, E>),
	FunctionCall(FunctionCall<'a, I, E>),
	If(IfStatement<'a, I, E>),
	Loop(Vec<LoopBodyStatement<'a, I, E>>),
	Return(Expression<'a, I, E>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum IfBodyStatements<'a, I: SemanticContextInstruction, E>
where
	E: ExtendedExpression<I>,
{
	#[serde(borrow)]
	If(Vec<IfBodyStatement<'a, I, E>>),
	Loop(Vec<IfLoopBodyStatement<'a, I, E>>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum IfLoopBodyStatement<'a, I: SemanticContextInstruction, E>
where
	E: ExtendedExpression<I>,
{
	#[serde(borrow)]
	LetBinding(LetBinding<'a, I, E>),
	Binding(Binding<'a, I, E>),
	FunctionCall(FunctionCall<'a, I, E>),
	If(IfStatement<'a, I, E>),
	Loop(Vec<LoopBodyStatement<'a, I, E>>),
	Return(Expression<'a, I, E>),
	Break,
	Continue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum LoopBodyStatement<'a, I: SemanticContextInstruction, E>
where
	E: ExtendedExpression<I>,
{
	#[serde(borrow)]
	LetBinding(LetBinding<'a, I, E>),
	Binding(Binding<'a, I, E>),
	FunctionCall(FunctionCall<'a, I, E>),
	If(IfStatement<'a, I, E>),
	Loop(Vec<Self>),
	Return(Expression<'a, I, E>),
	Break,
	Continue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum MainStatement<'a, I: SemanticContextInstruction, E>
where
	E: ExtendedExpression<I>,
{
	#[serde(borrow)]
	Import(ImportPath<'a>),
	Constant(Constant<'a>),
	Types(StructTypes<'a>),
	Function(FunctionStatement<'a, I, E>),
}

pub trait GetName {
	fn name(&self) -> Cow<'static, str>;
}

impl<T> GetName for Box<T>
where
	T: ?Sized + GetName,
{
	fn name(&self) -> Cow<'static, str> {
		(**self).name()
	}
}

pub trait GetLocation {
	fn location(&self) -> CodeLocation;
}

impl<T> GetLocation for Box<T>
where
	T: ?Sized + GetLocation,
{
	fn location(&self) -> CodeLocation {
		(**self).location()
	}
}

pub type ImportPath<'a> = Vec<ImportName<'a>>;

pub type Main<'a, I, E> = Vec<MainStatement<'a, I, E>>;
