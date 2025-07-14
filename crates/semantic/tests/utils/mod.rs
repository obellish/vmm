#![allow(dead_code)]

use std::{cell::RefCell, marker::PhantomData};

use serde::{Deserialize, Serialize};
use vmm_semantic::{
	PrimitiveTypes, PrimitiveValue,
	semantic::State,
	types::{
		BlockState, ExpressionResult, ExpressionResultValue, ExtendedExpression,
		SemanticContextInstruction, StateErrorType, Type,
	},
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct CustomExpression<I: SemanticContextInstruction>(PhantomData<I>);

#[expect(clippy::expl_impl_clone_on_copy)]
impl<I: SemanticContextInstruction> Clone for CustomExpression<I> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<I: SemanticContextInstruction> Copy for CustomExpression<I> {}

impl<I: SemanticContextInstruction> ExtendedExpression<I> for CustomExpression<I> {
	fn expression(
		&self,
		_: &mut State<I, Self>,
		_: &std::rc::Rc<RefCell<BlockState<I>>>,
	) -> ExpressionResult {
		ExpressionResult {
			ty: Type::Primitive(PrimitiveTypes::Ptr),
			value: ExpressionResultValue::Primitive(PrimitiveValue::Ptr),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct CustomExpressionInstruction;

impl SemanticContextInstruction for CustomExpressionInstruction {}

#[repr(transparent)]
pub struct SemanticTest<I: SemanticContextInstruction = CustomExpressionInstruction> {
	pub state: State<I, CustomExpression<I>>,
}

impl SemanticTest {
	pub fn new() -> Self {
		Self {
			state: State::new(),
		}
	}

	pub const fn has_error(&self) -> bool {
		!self.state.errors.is_empty()
	}

	pub fn clear_errors(&mut self) {
		self.state.errors.clear();
	}

	pub const fn errors_len(&self) -> usize {
		self.state.errors.len()
	}

	pub fn check_error(&self, kind: StateErrorType) -> bool {
		self.state.errors.first().is_some_and(|s| s.kind() == kind)
	}

	pub fn check_error_at(&self, index: usize, kind: StateErrorType) -> bool {
		self.state
			.errors
			.get(index)
			.is_some_and(|s| s.kind() == kind)
	}
}

impl Default for SemanticTest {
	fn default() -> Self {
		Self::new()
	}
}
