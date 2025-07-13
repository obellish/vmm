use alloc::{rc::Rc, vec::Vec};
use core::{cell::RefCell, marker::PhantomData};

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use super::{
	MAX_PRIORITY_LEVEL_FOR_EXPRESSIONS,
	ast::{CodeLocation, GetLocation, GetName},
	types::{
		BlockState, Constant, ConstantName, Expression, ExpressionResult, ExpressionResultValue,
		ExpressionStructValue, ExtendedExpression, Function, FunctionName,
		SemanticContextInstruction, SemanticStack, StateError, Type, TypeName,
	},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalState<I: SemanticContextInstruction> {
	pub constants: HashMap<ConstantName, Constant>,
	pub types: HashMap<TypeName, Type>,
	pub functions: HashMap<FunctionName, Function>,
	pub context: SemanticStack<I>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State<I: SemanticContextInstruction, E>
where
	E: ExtendedExpression<I>,
{
	pub global: GlobalState<I>,
	#[serde(skip)]
	pub context: Vec<Rc<RefCell<BlockState<I>>>>,
	pub errors: Vec<StateError>,
	marker: PhantomData<E>,
}

impl<I: SemanticContextInstruction, E> State<I, E>
where
	E: ExtendedExpression<I>,
{
	#[must_use]
	pub fn new() -> Self {
		Self {
			global: GlobalState {
				functions: HashMap::new(),
				types: HashMap::new(),
				constants: HashMap::new(),
				context: SemanticStack::new(),
			},
			context: Vec::new(),
			errors: Vec::new(),
			marker: PhantomData,
		}
	}
}

impl<I: SemanticContextInstruction, E> Default for State<I, E>
where
	E: ExtendedExpression<I>,
{
	fn default() -> Self {
		Self::new()
	}
}
