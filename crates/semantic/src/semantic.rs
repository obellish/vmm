use alloc::{boxed::Box, rc::Rc, string::ToString, vec::Vec};
use core::{cell::RefCell, marker::PhantomData};

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use tap::prelude::*;

use super::{
	ExpressionOperations, MAX_PRIORITY_LEVEL_FOR_EXPRESSIONS,
	ast::{self, CodeLocation, GetLocation, GetName},
	types::{
		Binding, BlockState, Constant, ConstantName, Expression, ExpressionResult,
		ExpressionResultValue, ExpressionStructValue, ExtendedExpression, Function, FunctionCall,
		FunctionName, FunctionParameter, FunctionStatement, GlobalSemanticContext, InnerValueName,
		LabelName, LetBinding, SemanticContext, SemanticContextInstruction, SemanticStack,
		StateError, StateErrorType, Type, TypeName, Value,
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

	pub fn run(&mut self, data: &ast::Main<'_, I, E>) {
		for main in data {
			if let ast::MainStatement::Types(types) = main {
				self.types(types);
			}
		}

		for main in data {
			match main {
				ast::MainStatement::Constant(constant) => self.constant(constant),
				ast::MainStatement::Function(function) => self.function_declaration(function),
				_ => {}
			}
		}

		for function in data.iter().filter_map(|main| {
			if let ast::MainStatement::Function(function) = main {
				Some(function)
			} else {
				None
			}
		}) {
			self.function_body(function);
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

	pub fn binding(
		&mut self,
		data: &ast::Binding<'_, I, E>,
		function_state: &Rc<RefCell<BlockState<I>>>,
	) {
		let Some(expr_result) = self.expression(&data.value, function_state) else {
			return;
		};

		let bind_data = data.clone().convert::<Binding>();

		let Some(value) = function_state.borrow().get_value(&bind_data.name) else {
			self.add_error(StateError::with_value(
				StateErrorType::ValueNotFound,
				data.location(),
				bind_data.to_string(),
			));
			return;
		};

		if !value.mutable {
			self.add_error(StateError::with_value(
				StateErrorType::ValueIsNotMutable,
				data.location(),
				bind_data.to_string(),
			));
			return;
		}

		function_state.borrow_mut().binding(value, expr_result);
	}

	pub fn condition_expression(
		&mut self,
		data: &ast::ExpressionLogicCondition<'_, I, E>,
		function_body_state: &Rc<RefCell<BlockState<I>>>,
	) -> u64 {
		let left_expr = &data.left.left;
		let left_res = self.expression(left_expr, function_body_state);

		let right_expr = &data.left.right;
		let right_res = self.expression(right_expr, function_body_state);

		let Some((left_res, right_res)) = left_res.zip(right_res) else {
			self.add_error(StateError::new(
				StateErrorType::ConditionIsEmpty,
				data.left.left.location(),
			));

			return function_body_state.borrow().last_register_number;
		};

		if left_res.ty != right_res.ty {
			self.add_error(StateError::with_value(
				StateErrorType::ConditionExpressionNotSupported,
				data.left.left.location(),
				left_res.ty.to_string(),
			));
			return function_body_state.borrow().last_register_number;
		}

		function_body_state.borrow_mut().inc_register();

		let register_number = function_body_state.borrow().last_register_number;

		function_body_state.borrow_mut().condition_expression(
			left_res,
			right_res,
			data.left.condition,
			register_number,
		);

		if let Some(right) = &data.right {
			let left_register_result = function_body_state.borrow().last_register_number;
			let right_register_result = self.condition_expression(&right.1, function_body_state);

			function_body_state.borrow_mut().inc_register();

			let register_number = function_body_state.borrow().last_register_number;

			function_body_state.borrow_mut().logic_condition(
				right.0,
				left_register_result,
				right_register_result,
				register_number,
			);
		}

		function_body_state.borrow().last_register_number
	}

	pub fn function_body(&mut self, data: &ast::FunctionStatement<'_, I, E>) {
		let body_state = Rc::new(RefCell::new(BlockState::new(None)));
		self.add_state_context(body_state.clone());
		self.init_func_params(&body_state, &data.parameters);
		let mut return_is_called = false;

		for body in &data.body {
			if return_is_called {
				self.add_error(StateError::new(
					StateErrorType::ForbiddenCodeAfterReturnDeprecated,
					CodeLocation::new(1, 1),
				));
			}

			match body {
				ast::BodyStatement::LetBinding(bind) => self.let_binding(bind, &body_state),
				ast::BodyStatement::Binding(bind) => self.binding(bind, &body_state),
				ast::BodyStatement::FunctionCall(fn_call) => {
					self.function_call(fn_call, &body_state);
				}
				ast::BodyStatement::If(if_condition) => {
					self.if_condition(if_condition, None, None, &body_state);
				}
				ast::BodyStatement::Loop(loop_statement) => {
					self.loop_statement(loop_statement, &body_state);
				}
				ast::BodyStatement::Expression(expression)
				| ast::BodyStatement::Return(expression) => {
					let expr_result = self.expression(expression, &body_state);
					let expr = expression.clone().convert::<Expression>();

					if return_is_called {
						self.add_error(StateError::with_value(
							StateErrorType::ReturnAlreadyCalled,
							expression.location(),
							expr.to_string(),
						));
					}

					if let Some(res) = expr_result {
						self.check_type_exists(&res.ty, &expr.to_string(), expression.location());
						let fn_ty = data.result_ty.clone().convert::<Type>();
						if fn_ty != res.ty {
							self.add_error(StateError::with_value(
								StateErrorType::WrongReturnType,
								expression.location(),
								expr.to_string(),
							));
						}

						return_is_called = true;

						if body_state.borrow().manual_return {
							body_state
								.borrow_mut()
								.expression_function_return_with_label(res);
						} else {
							body_state.borrow_mut().expression_function_return(res);
						}
					}
				}
			}
		}

		if !return_is_called {
			self.add_error(StateError::new(
				StateErrorType::ReturnNotFound,
				data.location(),
			));
		}
	}

	pub fn if_condition(
		&mut self,
		data: &ast::IfStatement<'_, I, E>,
		label_end: Option<&LabelName>,
		label_loop: Option<(LabelName, LabelName)>,
		function_body_state: &Rc<RefCell<BlockState<I>>>,
	) {
		if let (Some(_), Some(stm)) = (&data.else_statement, &data.else_if_statement) {
			self.add_error(StateError::with_value(
				StateErrorType::IfElseDuplicated,
				stm.location(),
				"if-condition",
			));
		}

		let if_body_state = Rc::new(RefCell::new(BlockState::new(Some(
			function_body_state.clone(),
		))));

		function_body_state
			.borrow_mut()
			.set_child(if_body_state.clone());

		let label_if_begin = if_body_state
			.borrow_mut()
			.get_and_set_next_label("if_begin".convert::<LabelName>());

		let label_if_else = if_body_state
			.borrow_mut()
			.get_and_set_next_label("if_else".convert());

		let label_if_end = label_end.map_or_else(
			|| {
				if_body_state
					.borrow_mut()
					.get_and_set_next_label("if_end".convert())
			},
			Clone::clone,
		);

		let is_set_label_if_end = label_end.is_some();
		let is_else = data.else_statement.is_some() || data.else_if_statement.is_some();

		self.if_condition_calculation(
			&data.condition,
			&if_body_state,
			label_if_begin.clone(),
			label_if_else.clone(),
			label_if_end.clone(),
			is_else,
		);

		if_body_state.borrow_mut().set_label(label_if_begin);

		let return_is_called = match &data.body {
			ast::IfBodyStatements::If(body) => self.if_condition_body(
				body,
				&if_body_state,
				label_if_end.clone(),
				label_loop.as_ref(),
			),
			ast::IfBodyStatements::Loop(body) => {
				let (label_loop_start, label_loop_end) =
					label_loop.clone().expect("loop label should be set");
				self.if_condition_loop_body(
					body,
					&if_body_state,
					label_if_end.clone(),
					label_loop_start,
					label_loop_end,
				)
			}
		};

		if !return_is_called {
			if_body_state.borrow_mut().jump_to(label_if_end.clone());
		}

		if is_else {
			if_body_state.borrow_mut().set_label(label_if_else);

			if let Some(else_body) = &data.else_statement {
				let if_else_body_state = Rc::new(RefCell::new(BlockState::new(Some(
					function_body_state.clone(),
				))));
				function_body_state
					.borrow_mut()
					.set_child(if_else_body_state);

				let return_is_called = match else_body {
					ast::IfBodyStatements::If(body) => self.if_condition_body(
						body,
						&if_body_state,
						label_if_end.clone(),
						label_loop.as_ref(),
					),
					ast::IfBodyStatements::Loop(body) => {
						let (label_loop_start, label_loop_end) =
							label_loop.expect("label should be set");
						self.if_condition_loop_body(
							body,
							&if_body_state,
							label_if_end.clone(),
							label_loop_start,
							label_loop_end,
						)
					}
				};

				if !return_is_called {
					if_body_state.borrow_mut().jump_to(label_if_end.clone());
				}
			} else if let Some(else_if_statement) = &data.else_if_statement {
				self.if_condition(
					else_if_statement,
					Some(&label_if_end),
					label_loop,
					function_body_state,
				);
			}
		}

		if !is_set_label_if_end {
			if_body_state.borrow_mut().set_label(label_if_end);
		}
	}

	pub fn if_condition_body(
		&mut self,
		body: &[ast::IfBodyStatement<'_, I, E>],
		if_body_state: &Rc<RefCell<BlockState<I>>>,
		label_end: LabelName,
		label_loop: Option<&(LabelName, LabelName)>,
	) -> bool {
		let mut return_is_called = false;

		for body in body {
			if return_is_called {
				self.add_error(StateError::new(
					StateErrorType::ForbiddenCodeAfterReturnDeprecated,
					CodeLocation::new(1, 1),
				));
			}

			match body {
				ast::IfBodyStatement::LetBinding(bind) => self.let_binding(bind, if_body_state),
				ast::IfBodyStatement::Binding(bind) => self.binding(bind, if_body_state),
				ast::IfBodyStatement::FunctionCall(fn_call) => {
					self.function_call(fn_call, if_body_state);
				}
				ast::IfBodyStatement::If(if_condition) => self.if_condition(
					if_condition,
					Some(&label_end),
					label_loop.cloned(),
					if_body_state,
				),
				ast::IfBodyStatement::Loop(loop_statement) => {
					self.loop_statement(loop_statement, if_body_state);
				}
				ast::IfBodyStatement::Return(e) => {
					let expr_result = self.expression(e, if_body_state);
					if let Some(res) = expr_result {
						let mut b = if_body_state.borrow_mut();

						b.jump_function_return(res);
						b.set_return();
						return_is_called = true;
					}
				}
			}
		}

		return_is_called
	}

	pub fn if_condition_loop_body(
		&mut self,
		body: &[ast::IfLoopBodyStatement<'_, I, E>],
		if_body_state: &Rc<RefCell<BlockState<I>>>,
		label_if_end: LabelName,
		label_loop_start: LabelName,
		label_loop_end: LabelName,
	) -> bool {
		let mut return_is_called = false;
		let mut break_is_called = false;
		let mut continue_is_called = false;

		for body in body {
			if return_is_called {
				self.add_error(StateError::new(
					StateErrorType::ForbiddenCodeAfterReturnDeprecated,
					CodeLocation::new(1, 1),
				));
			}

			if break_is_called {
				self.add_error(StateError::new(
					StateErrorType::ForbiddenCodeAfterBreakDeprecated,
					CodeLocation::new(1, 1),
				));
			}

			if continue_is_called {
				self.add_error(StateError::new(
					StateErrorType::ForbiddenCodeAfterContinueDeprecated,
					CodeLocation::new(1, 1),
				));
			}

			match body {
				ast::IfLoopBodyStatement::LetBinding(bind) => self.let_binding(bind, if_body_state),
				ast::IfLoopBodyStatement::Binding(bind) => self.binding(bind, if_body_state),
				ast::IfLoopBodyStatement::FunctionCall(fn_call) => {
					self.function_call(fn_call, if_body_state);
				}
				ast::IfLoopBodyStatement::Loop(loop_statement) => {
					self.loop_statement(loop_statement, if_body_state);
				}
				ast::IfLoopBodyStatement::Return(expression) => {
					let expr_result = self.expression(expression, if_body_state);
					if let Some(res) = expr_result {
						let mut b = if_body_state.borrow_mut();
						b.jump_function_return(res);
						b.set_return();
						return_is_called = true;
					}
				}
				ast::IfLoopBodyStatement::Continue => {
					if_body_state.borrow_mut().jump_to(label_loop_start.clone());
					continue_is_called = true;
				}
				ast::IfLoopBodyStatement::Break => {
					if_body_state.borrow_mut().jump_to(label_loop_end.clone());
					break_is_called = true;
				}
				ast::IfLoopBodyStatement::If(if_condition) => {
					self.if_condition(
						if_condition,
						Some(&label_if_end),
						Some((label_loop_start.clone(), label_loop_end.clone())),
						if_body_state,
					);
				}
			}
		}

		return_is_called
	}

	pub fn loop_statement(
		&mut self,
		data: &[ast::LoopBodyStatement<'_, I, E>],
		function_body_state: &Rc<RefCell<BlockState<I>>>,
	) {
		let loop_body_state = Rc::new(RefCell::new(BlockState::new(Some(
			function_body_state.clone(),
		))));
		function_body_state
			.borrow_mut()
			.set_child(loop_body_state.clone());

		let label_loop_begin = loop_body_state
			.borrow_mut()
			.get_and_set_next_label("loop_begin".into());
		let label_loop_end = loop_body_state
			.borrow_mut()
			.get_and_set_next_label("loop_end".into());

		loop_body_state
			.borrow_mut()
			.jump_to(label_loop_begin.clone());
		loop_body_state
			.borrow_mut()
			.set_label(label_loop_begin.clone());

		let mut return_is_called = false;
		let mut break_is_called = false;
		let mut continue_is_called = false;

		for body in data {
			if return_is_called {
				self.add_error(StateError::new(
					StateErrorType::ForbiddenCodeAfterReturnDeprecated,
					CodeLocation::new(1, 1),
				));
			}

			if break_is_called {
				self.add_error(StateError::new(
					StateErrorType::ForbiddenCodeAfterBreakDeprecated,
					CodeLocation::new(1, 1),
				));
			}

			if continue_is_called {
				self.add_error(StateError::new(
					StateErrorType::ForbiddenCodeAfterContinueDeprecated,
					CodeLocation::new(1, 1),
				));
			}

			match body {
				ast::LoopBodyStatement::LetBinding(bind) => {
					self.let_binding(bind, &loop_body_state);
				}
				ast::LoopBodyStatement::Binding(bind) => self.binding(bind, &loop_body_state),
				ast::LoopBodyStatement::FunctionCall(fn_call) => {
					self.function_call(fn_call, &loop_body_state);
				}
				ast::LoopBodyStatement::If(if_condition) => self.if_condition(
					if_condition,
					None,
					Some((label_loop_begin.clone(), label_loop_end.clone())),
					&loop_body_state,
				),
				ast::LoopBodyStatement::Loop(loop_statement) => {
					self.loop_statement(loop_statement, &loop_body_state);
				}
				ast::LoopBodyStatement::Return(expression) => {
					let expr_result = self.expression(expression, &loop_body_state);
					if let Some(res) = expr_result {
						let mut b = loop_body_state.borrow_mut();
						b.jump_function_return(res);
						b.set_return();
						return_is_called = true;
					}
				}
				ast::LoopBodyStatement::Break => {
					loop_body_state.borrow_mut().jump_to(label_loop_end.clone());
					break_is_called = true;
				}
				ast::LoopBodyStatement::Continue => {
					loop_body_state
						.borrow_mut()
						.jump_to(label_loop_begin.clone());
					continue_is_called = true;
				}
			}
		}

		if !return_is_called {
			loop_body_state.borrow_mut().jump_to(label_loop_begin);

			loop_body_state.borrow_mut().set_label(label_loop_end);
		}
	}

	pub fn if_condition_calculation(
		&mut self,
		condition: &ast::IfCondition<'_, I, E>,
		if_body_state: &Rc<RefCell<BlockState<I>>>,
		label_if_begin: LabelName,
		label_if_else: LabelName,
		label_if_end: LabelName,
		is_else: bool,
	) {
		match condition {
			ast::IfCondition::Single(expr) => {
				let Some(expr_result) = self.expression(expr, if_body_state) else {
					return;
				};

				if is_else {
					if_body_state.borrow_mut().if_condition_expression(
						expr_result,
						label_if_begin,
						label_if_else,
					);
				} else {
					if_body_state.borrow_mut().if_condition_expression(
						expr_result,
						label_if_begin,
						label_if_end,
					);
				}
			}
			ast::IfCondition::Logic(expr_logic) => {
				let result_register = self.condition_expression(expr_logic, if_body_state);

				if is_else {
					if_body_state.borrow_mut().if_condition_logic(
						label_if_begin,
						label_if_else,
						result_register,
					);
				} else {
					if_body_state.borrow_mut().if_condition_logic(
						label_if_begin,
						label_if_end,
						result_register,
					);
				}
			}
		}
	}

	pub fn let_binding(
		&mut self,
		data: &ast::LetBinding<'_, I, E>,
		function_state: &Rc<RefCell<BlockState<I>>>,
	) {
		let Some(expr_result) = self.expression(&data.value, function_state) else {
			return;
		};

		let let_data = data.clone().convert::<LetBinding>();

		if let Some(ty) = &let_data.ty
			&& &expr_result.ty != ty
		{
			self.add_error(StateError::with_value(
				StateErrorType::WrongLetType,
				data.location(),
				let_data.to_string(),
			));
			return;
		}

		let let_ty = expr_result.ty.clone();

		let value = function_state.borrow().get_value(&let_data.name);

		let inner_name = value.map_or_else(
			|| {
				function_state
					.borrow()
					.get_next_inner_name(&let_data.name.clone().convert::<InnerValueName>())
			},
			|val| function_state.borrow().get_next_inner_name(&val.name),
		);

		let value = Value {
			name: inner_name.clone(),
			ty: let_ty,
			mutable: let_data.mutable,
			alloca: false,
			malloc: false,
		};

		{
			let mut b = function_state.borrow_mut();

			b.values.insert(let_data.name, value.clone());

			b.set_inner_value_name(inner_name);

			b.let_binding(value, expr_result);
		}
	}

	pub fn function_call(
		&mut self,
		data: &ast::FunctionCall<'_, I, E>,
		body_state: &Rc<RefCell<BlockState<I>>>,
	) -> Option<Type> {
		let func_call_data = data.clone().convert::<FunctionCall>();

		let Some(func_data) = self.global.functions.get(&func_call_data.name).cloned() else {
			self.add_error(StateError::with_value(
				StateErrorType::FunctionNotFound,
				data.location(),
				func_call_data.to_string(),
			));
			return None;
		};

		let fn_type = func_data.ty.clone();

		let mut params = Vec::<ExpressionResult>::new();
		for (i, expr) in data.parameters.iter().enumerate() {
			let expr_result = self.expression(expr, body_state)?;
			if expr_result.ty != func_data.parameters[i] {
				self.add_error(StateError::with_value(
					StateErrorType::FunctionParameterTypeWrong,
					data.location(),
					expr_result.ty.to_string(),
				));
				continue;
			}

			params.push(expr_result);
		}

		body_state.borrow_mut().inc_register();
		let last_register_number = body_state.borrow().last_register_number;

		body_state
			.borrow_mut()
			.call(func_data, params, last_register_number);

		Some(fn_type)
	}

	pub fn expression(
		&mut self,
		data: &ast::Expression<'_, I, E>,
		body_state: &Rc<RefCell<BlockState<I>>>,
	) -> Option<ExpressionResult> {
		let expr = Self::expression_operation_priority(data.clone());

		self.expression_operation(None, &expr, None, body_state)
	}

	pub fn expression_operation(
		&mut self,
		left_value: Option<&ExpressionResult>,
		right_expression: &ast::Expression<'_, I, E>,
		op: Option<ExpressionOperations>,
		body_state: &Rc<RefCell<BlockState<I>>>,
	) -> Option<ExpressionResult> {
		let right_value = match &right_expression.value {
			ast::ExpressionValue::ValueName(value) => {
				let value_from_state = body_state.borrow_mut().get_value(&value.name().into());

				body_state.borrow_mut().inc_register();
				let last_register_number = body_state.borrow().last_register_number;
				let ty = if let Some(val) = value_from_state {
					body_state
						.borrow_mut()
						.expression_value(val.clone(), last_register_number);
					val.ty
				} else if let Some(const_val) = self
					.global
					.constants
					.get(&value.name().convert::<ConstantName>())
				{
					body_state
						.borrow_mut()
						.expression_const(const_val.clone(), last_register_number);
					const_val.ty.clone()
				} else {
					self.add_error(StateError::with_value(
						StateErrorType::ValueNotFound,
						value.location(),
						value.name(),
					));
					return None;
				};

				ExpressionResult {
					ty,
					value: ExpressionResultValue::Register(
						body_state.borrow().last_register_number,
					),
				}
			}
			ast::ExpressionValue::Primitive(value) => ExpressionResult {
				ty: value.ty().into(),
				value: ExpressionResultValue::Primitive(*value),
			},
			ast::ExpressionValue::FunctionCall(fn_call) => {
				let func_call_ty = self.function_call(fn_call, body_state)?;

				body_state.borrow_mut().inc_register();

				ExpressionResult {
					ty: func_call_ty,
					value: ExpressionResultValue::Register(
						body_state.borrow().last_register_number,
					),
				}
			}
			ast::ExpressionValue::Struct(value) => {
				let struct_value = (*value).convert::<ExpressionStructValue>();

				let val = body_state
					.borrow_mut()
					.get_value(&struct_value.name)
					.or_else(|| {
						self.add_error(StateError::with_value(
							StateErrorType::ValueNotFound,
							value.name.location(),
							value.name.name(),
						));
						None
					})?;

				let ty = val.ty.as_struct().or_else(|| {
					self.add_error(StateError::with_value(
						StateErrorType::ValueNotStruct,
						value.name.location(),
						value.name.name(),
					));
					None
				})?;

				if !self.check_type_exists(&val.ty, &value.name.name(), value.name.location()) {
					return None;
				}

				if &Type::Struct(ty.clone()) != self.global.types.get(&val.ty.name())? {
					self.add_error(StateError::with_value(
						StateErrorType::WrongExpressionType,
						value.name.location(),
						value.name.name(),
					));
					return None;
				}

				let attributes = ty
					.attributes
					.get(&struct_value.attribute)
					.or_else(|| {
						self.add_error(StateError::with_value(
							StateErrorType::ValueNotStructField,
							value.name.location(),
							value.name.name(),
						));
						None
					})?
					.clone();

				body_state.borrow_mut().inc_register();
				let last_register_number = body_state.borrow().last_register_number;
				body_state.borrow_mut().expression_struct_value(
					val.clone(),
					attributes.index,
					last_register_number,
				);

				body_state.borrow_mut().inc_register();

				ExpressionResult {
					ty: attributes.ty,
					value: ExpressionResultValue::Register(
						body_state.borrow().last_register_number,
					),
				}
			}
			ast::ExpressionValue::Expression(expr) => self.expression(expr, body_state)?,
			ast::ExpressionValue::ExtendedExpression(expr) => expr.expression(self, body_state),
			ast::ExpressionValue::Marker(..) => unreachable!(),
		};

		let expression_result = if let (Some(left_value), Some(op)) = (left_value, op) {
			if left_value.ty != right_value.ty {
				self.add_error(StateError::with_value(
					StateErrorType::WrongExpressionType,
					right_expression.location(),
					left_value.ty.to_string(),
				));
				return None;
			}

			body_state.borrow_mut().inc_register();
			let last_register_number = body_state.borrow().last_register_number;

			body_state.borrow_mut().expression_operation(
				op,
				left_value.clone(),
				right_value.clone(),
				last_register_number,
			);

			ExpressionResult {
				ty: right_value.ty,
				value: ExpressionResultValue::Register(body_state.borrow().last_register_number),
			}
		} else {
			right_value
		};

		if let Some((operation, expr)) = &right_expression.operation {
			self.expression_operation(Some(&expression_result), expr, Some(*operation), body_state)
		} else {
			Some(expression_result)
		}
	}

	fn init_func_params(
		&mut self,
		function_state: &Rc<RefCell<BlockState<I>>>,
		fn_params: &[ast::FunctionParameter<'_>],
	) {
		for fn_param in fn_params {
			let func_param = fn_param.clone().convert::<FunctionParameter>();
			let arg_name = func_param.to_string();

			let value = function_state.borrow().get_value(&arg_name.clone().into());

			let inner_value_name = match value {
				Some(..) => {
					self.add_error(StateError::with_value(
						StateErrorType::FunctionArgumentNameDuplicated,
						CodeLocation::new(1, 1),
						arg_name,
					));
					return;
				}
				None => arg_name.clone().convert::<InnerValueName>(),
			};

			let value = Value {
				name: inner_value_name.clone(),
				ty: func_param.ty.clone(),
				malloc: false,
				mutable: false,
				alloca: false,
			};

			function_state
				.borrow_mut()
				.values
				.insert(arg_name.into(), value.clone());

			function_state
				.borrow_mut()
				.set_inner_value_name(inner_value_name);

			function_state.borrow_mut().function_arg(value, func_param);
		}
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
