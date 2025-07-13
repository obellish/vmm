mod rc_serializer;

use alloc::{
	rc::Rc,
	string::{String, ToString as _},
	vec::Vec,
};
use core::cell::RefCell;

use hashbrown::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use tap::prelude::*;

use super::{
	Constant, ExpressionResult, ExtendedSemanticContext, Function, FunctionParameter,
	InnerValueName, LabelName, SemanticContext, SemanticContextInstruction, SemanticStack, Value,
	ValueName,
};
use crate::{Condition, ExpressionOperations, LogicCondition};

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockState<I: SemanticContextInstruction> {
	pub values: HashMap<ValueName, Value>,
	pub inner_value_names: HashSet<InnerValueName>,
	pub labels: HashSet<LabelName>,
	pub last_register_number: u64,
	pub manual_return: bool,
	#[serde(
		serialize_with = "rc_serializer::serialize_option",
		deserialize_with = "rc_serializer::deserialize_option"
	)]
	pub parent: Option<Rc<RefCell<Self>>>,
	#[serde(
		serialize_with = "rc_serializer::serialize_vec",
		deserialize_with = "rc_serializer::deserialize_vec"
	)]
	pub children: Vec<Rc<RefCell<Self>>>,
	context: SemanticStack<I>,
}

impl<I: SemanticContextInstruction> BlockState<I> {
	#[must_use]
	pub fn new(parent: Option<Rc<RefCell<Self>>>) -> Self {
		let (last_register_number, inner_value_names, labels, manual_return) =
			parent.clone().map_or_else(
				|| (0, HashSet::new(), HashSet::new(), false),
				|p| {
					let parent = p.borrow();
					(
						parent.last_register_number,
						parent.inner_value_names.clone(),
						parent.labels.clone(),
						parent.manual_return,
					)
				},
			);

		Self {
			values: HashMap::new(),
			children: Vec::new(),
			inner_value_names,
			labels,
			manual_return,
			parent,
			context: SemanticStack::new(),
			last_register_number,
		}
	}

	#[must_use]
	pub fn context(&self) -> SemanticStack<I> {
		self.context.clone()
	}

	fn set_register(&mut self, last_register_number: u64) {
		self.last_register_number = last_register_number;

		self.borrow_parent_mut(|p| p.set_register(last_register_number));
	}

	pub fn inc_register(&mut self) {
		self.set_register(self.last_register_number + 1);
	}

	pub fn set_child(&mut self, child: Rc<RefCell<Self>>) {
		self.children.push(child);
	}

	pub fn set_inner_value_name(&mut self, name: InnerValueName) {
		self.inner_value_names.insert(name.clone());

		self.borrow_parent_mut(|p| p.set_inner_value_name(name));
	}

	#[must_use]
	pub fn inner_value_name_exists(&self, name: &InnerValueName) -> bool {
		self.inner_value_names.contains(name)
			|| self.borrow_parent_or_default(|f| f.inner_value_name_exists(name))
	}

	#[must_use]
	pub fn get_value(&self, name: &ValueName) -> Option<Value> {
		if let Some(value) = self.values.get(name).cloned() {
			return Some(value);
		}

		self.borrow_parent_or(None, |p| p.get_value(name))
	}

	#[must_use]
	pub fn label_name_exists(&self, name: &LabelName) -> bool {
		self.labels.contains(name) || self.borrow_parent_or_default(|p| p.label_name_exists(name))
	}

	pub fn set_label_name(&mut self, name: LabelName) {
		self.labels.insert(name.clone());

		self.borrow_parent_mut(|p| p.set_label_name(name));
	}

	#[must_use]
	pub fn set_attr_counter(value: &str) -> String {
		let val_attr = value.split('.').collect::<Vec<_>>();
		if matches!(val_attr.len(), 2) {
			let i = val_attr[1].parse::<u64>().unwrap_or_default();
			alloc::format!("{}.{}", val_attr[0], i + 1)
		} else {
			alloc::format!("{}.0", val_attr[0])
		}
	}

	pub fn get_and_set_next_label(&mut self, label: LabelName) -> LabelName {
		if !self.label_name_exists(&label) {
			self.set_label_name(label.clone());
			return label;
		}

		let name = Self::set_attr_counter(&label.to_string()).convert::<LabelName>();
		if self.label_name_exists(&name) {
			self.get_and_set_next_label(name)
		} else {
			self.set_label_name(name.clone());
			name
		}
	}

	#[must_use]
	pub fn get_next_inner_name(&self, value: &InnerValueName) -> InnerValueName {
		let name = Self::set_attr_counter(&value.to_string()).convert::<InnerValueName>();
		if self.inner_value_name_exists(&name) {
			self.get_next_inner_name(&name)
		} else {
			name
		}
	}

	pub fn set_return(&mut self) {
		self.manual_return = true;
		self.borrow_parent_mut(Self::set_return);
	}

	fn borrow_parent_mut(&self, f: impl FnOnce(&mut Self)) {
		if let Some(parent) = &self.parent {
			f(&mut *parent.borrow_mut());
		}
	}

	fn borrow_parent_or<T>(&self, default: T, f: impl FnOnce(&Self) -> T) -> T {
		if let Some(parent) = &self.parent {
			f(&*parent.borrow())
		} else {
			default
		}
	}

	fn borrow_parent_or_default<T: Default>(&self, f: impl FnOnce(&Self) -> T) -> T {
		self.borrow_parent_or(T::default(), f)
	}
}

impl<I: SemanticContextInstruction> ExtendedSemanticContext<I> for BlockState<I> {
	fn extended_expression(&mut self, expr: &I) {
		self.context.extended_expression(expr);
		self.borrow_parent_mut(|p| p.extended_expression(expr));
	}
}

impl<I: SemanticContextInstruction> SemanticContext for BlockState<I> {
	fn expression_value(&mut self, expression: Value, register_number: u64) {
		self.context
			.expression_value(expression.clone(), register_number);
		self.borrow_parent_mut(|p| p.expression_value(expression, register_number));
	}

	fn expression_const(&mut self, expression: Constant, register_number: u64) {
		self.context
			.expression_const(expression.clone(), register_number);

		self.borrow_parent_mut(|p| p.expression_const(expression, register_number));
	}

	fn expression_struct_value(&mut self, expression: Value, index: u32, register_number: u64) {
		self.context
			.expression_struct_value(expression.clone(), index, register_number);
		self.borrow_parent_mut(|p| p.expression_struct_value(expression, index, register_number));
	}

	fn expression_operation(
		&mut self,
		operation: ExpressionOperations,
		left_value: ExpressionResult,
		right_value: ExpressionResult,
		register_number: u64,
	) {
		self.context.expression_operation(
			operation,
			left_value.clone(),
			right_value.clone(),
			register_number,
		);
		self.borrow_parent_mut(|p| {
			p.expression_operation(operation, left_value, right_value, register_number);
		});
	}

	fn call(&mut self, call: Function, params: Vec<ExpressionResult>, register_number: u64) {
		self.context
			.call(call.clone(), params.clone(), register_number);

		self.borrow_parent_mut(|p| p.call(call, params, register_number));
	}

	fn let_binding(&mut self, let_decl: Value, expr_result: ExpressionResult) {
		self.context
			.let_binding(let_decl.clone(), expr_result.clone());
		self.borrow_parent_mut(|p| p.let_binding(let_decl, expr_result));
	}

	fn binding(&mut self, val: Value, expr_result: ExpressionResult) {
		self.context.binding(val.clone(), expr_result.clone());
		self.borrow_parent_mut(|p| p.binding(val, expr_result));
	}

	fn expression_function_return(&mut self, expr_result: ExpressionResult) {
		self.context.expression_function_return(expr_result.clone());
		self.borrow_parent_mut(|p| p.expression_function_return(expr_result));
	}

	fn expression_function_return_with_label(&mut self, expr_result: ExpressionResult) {
		self.context
			.expression_function_return_with_label(expr_result.clone());
		self.borrow_parent_mut(|p| p.expression_function_return_with_label(expr_result));
	}

	fn set_label(&mut self, label: LabelName) {
		self.context.set_label(label.clone());
		self.borrow_parent_mut(|p| p.set_label(label));
	}

	fn jump_to(&mut self, label: LabelName) {
		self.context.jump_to(label.clone());
		self.borrow_parent_mut(|p| p.jump_to(label));
	}

	fn if_condition_expression(
		&mut self,
		expr_result: ExpressionResult,
		label_if_begin: LabelName,
		label_if_end: LabelName,
	) {
		self.context.if_condition_expression(
			expr_result.clone(),
			label_if_begin.clone(),
			label_if_end.clone(),
		);
		self.borrow_parent_mut(|p| {
			p.if_condition_expression(expr_result, label_if_begin, label_if_end);
		});
	}

	fn if_condition_logic(
		&mut self,
		label_if_begin: LabelName,
		label_if_end: LabelName,
		result_register: u64,
	) {
		self.context.if_condition_logic(
			label_if_begin.clone(),
			label_if_end.clone(),
			result_register,
		);
		self.borrow_parent_mut(|p| {
			p.if_condition_logic(label_if_begin, label_if_end, result_register);
		});
	}

	fn condition_expression(
		&mut self,
		left_result: ExpressionResult,
		right_result: ExpressionResult,
		condition: Condition,
		register_number: u64,
	) {
		self.context.condition_expression(
			left_result.clone(),
			right_result.clone(),
			condition,
			register_number,
		);
		self.borrow_parent_mut(|p| {
			p.condition_expression(left_result, right_result, condition, register_number);
		});
	}

	fn jump_function_return(&mut self, expr_result: ExpressionResult) {
		self.context.jump_function_return(expr_result.clone());
		self.borrow_parent_mut(|p| p.jump_function_return(expr_result));
	}

	fn logic_condition(
		&mut self,
		logic_condition: LogicCondition,
		left_register_result: u64,
		right_register_result: u64,
		register_number: u64,
	) {
		self.context.logic_condition(
			logic_condition,
			left_register_result,
			right_register_result,
			register_number,
		);
		self.borrow_parent_mut(|p| {
			p.logic_condition(
				logic_condition,
				left_register_result,
				right_register_result,
				register_number,
			);
		});
	}

	fn function_arg(&mut self, value: Value, func_arg: FunctionParameter) {
		self.context.function_arg(value.clone(), func_arg.clone());
		self.borrow_parent_mut(|p| p.function_arg(value, func_arg));
	}
}
