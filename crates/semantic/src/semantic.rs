use alloc::{boxed::Box, rc::Rc, string::ToString, vec::Vec};
use core::{cell::RefCell, marker::PhantomData};

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use tap::prelude::*;

use super::{
	MAX_PRIORITY_LEVEL_FOR_EXPRESSIONS,
	ast::{self, CodeLocation, GetLocation, GetName},
	types::{
		BlockState, Constant, ConstantName, Expression, ExpressionResult, ExpressionResultValue,
		ExpressionStructValue, ExtendedExpression, Function, FunctionName, GlobalSemanticContext,
		SemanticContextInstruction, SemanticStack, StateError, StateErrorType, Type, TypeName,
	},
};
use crate::{ExpressionOperations, types::FunctionStatement};

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

	pub fn types(&mut self, data: &ast::StructTypes<'_>) {
		if self
			.global
			.types
			.contains_key(&data.name().convert::<TypeName>())
		{
			self.add_error(StateError::with_value(
				StateErrorType::TypeAlreadyExists,
				data.location(),
				data.name(),
			));
			return;
		}

		let struct_type = Type::Struct(data.clone().into());
		self.global.types.insert(struct_type.name(), struct_type);
		self.global.context.types(data.clone().into());
	}

	pub fn check_constant_value_expression(
		&mut self,
		data: Option<&(ExpressionOperations, Box<ast::ConstantExpression<'_>>)>,
	) -> bool {
		if let Some((_, child_data)) = data {
			match child_data.value {
				ast::ConstantValue::Value(..) => true,
				ast::ConstantValue::Constant(const_name) => {
					if !self
						.global
						.constants
						.contains_key(&const_name.convert::<ConstantName>())
					{
						self.add_error(StateError::with_value(
							StateErrorType::ConstantNotFound,
							const_name.location(),
							const_name.name(),
						));

						return false;
					}
					self.check_constant_value_expression(child_data.operation.as_ref())
				}
			}
		} else {
			true
		}
	}

	pub fn constant(&mut self, data: &ast::Constant<'_>) {
		if self
			.global
			.constants
			.contains_key(&data.name().convert::<ConstantName>())
		{
			self.add_error(StateError::with_value(
				StateErrorType::ConstantAlreadyExists,
				data.location(),
				data.name(),
			));
			return;
		}

		if !self.check_constant_value_expression(data.value.operation.as_ref()) {
			return;
		}

		let const_val = data.clone().convert::<Constant>();
		if !self.check_type_exists(&const_val.ty, &const_val.name, data.location()) {
			return;
		}

		self.global
			.constants
			.insert(const_val.name.clone(), const_val.clone());
		self.global.context.constant(const_val);
	}

	pub fn function_declaration(&mut self, data: &ast::FunctionStatement<'_, I, E>) {
		if self
			.global
			.functions
			.contains_key(&data.name().convert::<FunctionName>())
		{
			self.add_error(StateError::with_value(
				StateErrorType::FunctionAlreadyExists,
				data.location(),
				data.name(),
			));
			return;
		}

		let func_decl = data.clone().convert::<FunctionStatement>();
		let mut force_quite =
			!self.check_type_exists(&func_decl.result_ty, &func_decl.name, data.location());

		let parameters = func_decl
			.parameters
			.iter()
			.map(|p| {
				force_quite |= !self.check_type_exists(&p.ty, &p.name, data.location());
				p.ty.clone()
			})
			.collect();

		self.global.functions.insert(
			data.name().into(),
			Function {
				name: func_decl.name,
				ty: func_decl.result_ty,
				parameters,
			},
		);
		self.global
			.context
			.function_declaration(data.clone().into());
	}

	fn add_error(&mut self, error: StateError) {
		self.errors.push(error);
	}

	fn add_state_context(&mut self, state_body: Rc<RefCell<BlockState<I>>>) {
		self.context.push(state_body);
	}

	fn check_type_exists(
		&mut self,
		ty: &Type,
		value_name: &str,
		location: impl GetLocation,
	) -> bool {
		if let Type::Primitive(_) = ty {
			return true;
		}

		if !self.global.types.contains_key(&ty.name()) {
			self.add_error(StateError::with_value(
				StateErrorType::TypeNotFound,
				location.location(),
				value_name,
			));
			return false;
		}

		true
	}

	fn expression_operation_priority(
		mut data: ast::Expression<'_, I, E>,
	) -> ast::Expression<'_, I, E> {
		for priority in (0..=MAX_PRIORITY_LEVEL_FOR_EXPRESSIONS).rev() {
			data = Self::fetch_op_priority(data, priority);
		}

		data
	}

	fn fetch_op_priority(
		data: ast::Expression<'_, I, E>,
		priority_level: u8,
	) -> ast::Expression<'_, I, E> {
		if let Some((op, expr)) = data.operation.clone() {
			if let Some((next_op, next_expr)) = expr.operation.clone() {
				if op.priority() == priority_level {
					let expression_value =
						ast::ExpressionValue::Expression(Box::new(ast::Expression {
							value: data.value,
							operation: Some((
								op,
								Box::new(ast::Expression {
									value: expr.value,
									operation: None,
								}),
							)),
						}));

					let new_expr = Self::fetch_op_priority(*next_expr, priority_level);

					ast::Expression {
						value: expression_value,
						operation: Some((next_op, Box::new(new_expr))),
					}
				} else {
					let new_expr =
						if next_op.priority() > op.priority() && next_expr.operation.is_some() {
							ast::Expression {
								value: ast::ExpressionValue::Expression(expr),
								operation: None,
							}
						} else {
							Self::fetch_op_priority(*expr, priority_level)
						};

					ast::Expression {
						value: data.value,
						operation: Some((op, Box::new(new_expr))),
					}
				}
			} else {
				data
			}
		} else {
			data
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
