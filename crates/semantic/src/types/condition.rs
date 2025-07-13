use alloc::{boxed::Box, vec::Vec};

use serde::{Deserialize, Serialize};

use super::{
	Binding, Expression, ExtendedExpression, FunctionCall, LetBinding, SemanticContextInstruction,
};
use crate::{Condition, LogicCondition};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionCondition {
	pub left: Expression,
	pub condition: Condition,
	pub right: Expression,
}

impl<I: SemanticContextInstruction, E> From<crate::ast::ExpressionCondition<'_, I, E>>
	for ExpressionCondition
where
	E: ExtendedExpression<I>,
{
	fn from(value: crate::ast::ExpressionCondition<'_, I, E>) -> Self {
		Self {
			left: value.left.into(),
			condition: value.condition,
			right: value.right.into(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionLogicCondition {
	pub left: ExpressionCondition,
	pub right: Option<(LogicCondition, Box<Self>)>,
}

impl<I: SemanticContextInstruction, E> From<crate::ast::ExpressionLogicCondition<'_, I, E>>
	for ExpressionLogicCondition
where
	E: ExtendedExpression<I>,
{
	fn from(value: crate::ast::ExpressionLogicCondition<'_, I, E>) -> Self {
		Self {
			left: value.left.into(),
			right: value.right.map(|(v, expr)| (v, Box::new((*expr).into()))),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfStatement {
	pub condition: IfCondition,
	pub body: IfBodyStatements,
	pub else_statements: Option<IfBodyStatements>,
	pub else_if_statements: Option<Box<Self>>,
}

impl<I: SemanticContextInstruction, E> From<crate::ast::IfStatement<'_, I, E>> for IfStatement
where
	E: ExtendedExpression<I>,
{
	fn from(value: crate::ast::IfStatement<'_, I, E>) -> Self {
		Self {
			condition: value.condition.into(),
			body: value.body.into(),
			else_statements: value.else_statement.map(Into::into),
			else_if_statements: value.else_if_statement.map(|v| Box::new((*v).into())),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum IfCondition {
	Single(Expression),
	Logic(ExpressionLogicCondition),
}

impl<I: SemanticContextInstruction, E> From<crate::ast::IfCondition<'_, I, E>> for IfCondition
where
	E: ExtendedExpression<I>,
{
	fn from(value: crate::ast::IfCondition<'_, I, E>) -> Self {
		match value {
			crate::ast::IfCondition::Logic(e) => Self::Logic(e.into()),
			crate::ast::IfCondition::Single(e) => Self::Single(e.into()),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum LoopBodyStatement {
	LetBinding(LetBinding),
	Binding(Binding),
	FunctionCall(FunctionCall),
	If(IfStatement),
	Loop(Vec<LoopBodyStatement>),
	Return(Expression),
	Break,
	Continue,
}

impl<I: SemanticContextInstruction, E> From<crate::ast::LoopBodyStatement<'_, I, E>>
	for LoopBodyStatement
where
	E: ExtendedExpression<I>,
{
	fn from(value: crate::ast::LoopBodyStatement<'_, I, E>) -> Self {
		match value {
			crate::ast::LoopBodyStatement::Binding(v) => Self::Binding(v.into()),
			crate::ast::LoopBodyStatement::Break => Self::Break,
			crate::ast::LoopBodyStatement::Continue => Self::Continue,
			crate::ast::LoopBodyStatement::FunctionCall(f) => Self::FunctionCall(f.into()),
			crate::ast::LoopBodyStatement::If(i) => Self::If(i.into()),
			crate::ast::LoopBodyStatement::LetBinding(l) => Self::LetBinding(l.into()),
			crate::ast::LoopBodyStatement::Return(e) => Self::Return(e.into()),
			crate::ast::LoopBodyStatement::Loop(v) => {
				Self::Loop(v.into_iter().map(Into::into).collect())
			}
		}
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum IfBodyStatement {
	LetBinding(LetBinding),
	Binding(Binding),
	FunctionCall(FunctionCall),
	If(IfStatement),
	Loop(Vec<LoopBodyStatement>),
	Return(Expression),
}

impl<I: SemanticContextInstruction, E> From<crate::ast::IfBodyStatement<'_, I, E>>
	for IfBodyStatement
where
	E: ExtendedExpression<I>,
{
	fn from(value: crate::ast::IfBodyStatement<'_, I, E>) -> Self {
		match value {
			crate::ast::IfBodyStatement::Binding(b) => Self::Binding(b.into()),
			crate::ast::IfBodyStatement::FunctionCall(f) => Self::FunctionCall(f.into()),
			crate::ast::IfBodyStatement::LetBinding(l) => Self::LetBinding(l.into()),
			crate::ast::IfBodyStatement::Loop(l) => {
				Self::Loop(l.into_iter().map(Into::into).collect())
			}
			crate::ast::IfBodyStatement::Return(e) => Self::Return(e.into()),
			crate::ast::IfBodyStatement::If(i) => Self::If(i.into()),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum IfLoopBodyStatement {
	LetBinding(LetBinding),
	Binding(Binding),
	FunctionCall(FunctionCall),
	If(IfStatement),
	Loop(Vec<LoopBodyStatement>),
	Return(Expression),
	Break,
	Continue,
}

impl<I: SemanticContextInstruction, E> From<crate::ast::IfLoopBodyStatement<'_, I, E>>
	for IfLoopBodyStatement
where
	E: ExtendedExpression<I>,
{
	fn from(value: crate::ast::IfLoopBodyStatement<'_, I, E>) -> Self {
		match value {
			crate::ast::IfLoopBodyStatement::Binding(b) => Self::Binding(b.into()),
			crate::ast::IfLoopBodyStatement::Break => Self::Break,
			crate::ast::IfLoopBodyStatement::Continue => Self::Continue,
			crate::ast::IfLoopBodyStatement::FunctionCall(f) => Self::FunctionCall(f.into()),
			crate::ast::IfLoopBodyStatement::LetBinding(l) => Self::LetBinding(l.into()),
			crate::ast::IfLoopBodyStatement::Return(e) => Self::Return(e.into()),
			crate::ast::IfLoopBodyStatement::If(i) => Self::If(i.into()),
			crate::ast::IfLoopBodyStatement::Loop(l) => {
				Self::Loop(l.into_iter().map(Into::into).collect())
			}
		}
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum IfBodyStatements {
	If(Vec<IfBodyStatement>),
	Loop(Vec<IfLoopBodyStatement>),
}

impl<I: SemanticContextInstruction, E> From<crate::ast::IfBodyStatements<'_, I, E>>
	for IfBodyStatements
where
	E: ExtendedExpression<I>,
{
	fn from(value: crate::ast::IfBodyStatements<'_, I, E>) -> Self {
		match value {
			crate::ast::IfBodyStatements::If(v) => {
				Self::If(v.into_iter().map(Into::into).collect())
			}
			crate::ast::IfBodyStatements::Loop(v) => {
				Self::Loop(v.into_iter().map(Into::into).collect())
			}
		}
	}
}
