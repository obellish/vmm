use alloc::{boxed::Box, rc::Rc, vec::Vec};
use core::{cell::RefCell, fmt::Debug};

use serde::{Deserialize, Serialize};

use super::{
	BlockState, Constant, ExpressionResult, Function, FunctionParameter, FunctionStatement,
	LabelName, StructTypes, Value,
};
use crate::{Condition, ExpressionOperations, LogicCondition, semantic::State};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct SemanticStack<I: SemanticContextInstruction>(Vec<SemanticStackContext<I>>);

impl<I: SemanticContextInstruction> SemanticStack<I> {
	#[must_use]
	pub const fn new() -> Self {
		Self(Vec::new())
	}

	pub fn push(&mut self, value: SemanticStackContext<I>) {
		self.0.push(value);
	}

	#[must_use]
	pub fn into_inner(self) -> Vec<SemanticStackContext<I>> {
		self.0
	}
}

impl<I: SemanticContextInstruction> Default for SemanticStack<I> {
	fn default() -> Self {
		Self::new()
	}
}

impl<I: SemanticContextInstruction> ExtendedSemanticContext<I> for SemanticStack<I> {
	fn extended_expression(&mut self, expr: &I) {
		self.push(SemanticStackContext::ExtendedExpression(Box::new(
			expr.clone(),
		)));
	}
}

impl<I: SemanticContextInstruction> GlobalSemanticContext for SemanticStack<I> {
	fn function_declaration(&mut self, fn_decl: FunctionStatement) {
		self.push(SemanticStackContext::FunctionDeclaration { fn_decl });
	}

	fn constant(&mut self, const_decl: Constant) {
		self.push(SemanticStackContext::Constant { const_decl });
	}

	fn types(&mut self, type_decl: StructTypes) {
		self.push(SemanticStackContext::Types { type_decl });
	}
}

impl<I: SemanticContextInstruction> SemanticContext for SemanticStack<I> {
	fn expression_value(&mut self, expression: Value, register_number: u64) {
		self.push(SemanticStackContext::ExpressionValue {
			expression,
			register_number,
		});
	}

	fn expression_const(&mut self, expression: Constant, register_number: u64) {
		self.push(SemanticStackContext::ExpressionConst {
			expression,
			register_number,
		});
	}

	fn expression_struct_value(&mut self, expression: Value, index: u32, register_number: u64) {
		self.push(SemanticStackContext::ExpressionStructValue {
			expression,
			index,
			register_number,
		});
	}

	fn expression_operation(
		&mut self,
		operation: ExpressionOperations,
		left_value: ExpressionResult,
		right_value: ExpressionResult,
		register_number: u64,
	) {
		self.push(SemanticStackContext::ExpressionOperation {
			operation,
			left_value,
			right_value,
			register_number,
		});
	}

	fn call(&mut self, call: Function, params: Vec<ExpressionResult>, register_number: u64) {
		self.push(SemanticStackContext::Call {
			call,
			params,
			register_number,
		});
	}

	fn let_binding(&mut self, let_decl: Value, expr_result: ExpressionResult) {
		self.push(SemanticStackContext::LetBinding {
			let_decl,
			expr_result,
		});
	}

	fn binding(&mut self, value: Value, expr_result: ExpressionResult) {
		self.push(SemanticStackContext::Binding { value, expr_result });
	}

	fn expression_function_return(&mut self, expr_result: ExpressionResult) {
		self.push(SemanticStackContext::ExpressionFunctionReturn { expr_result });
	}

	fn expression_function_return_with_label(&mut self, expr_result: ExpressionResult) {
		self.push(SemanticStackContext::ExpressionFunctionReturnWithLabel { expr_result });
	}

	fn set_label(&mut self, label: LabelName) {
		self.push(SemanticStackContext::SetLabel { label });
	}

	fn jump_to(&mut self, label: LabelName) {
		self.push(SemanticStackContext::JumpTo { label });
	}

	fn if_condition_expression(
		&mut self,
		expr_result: ExpressionResult,
		if_begin: LabelName,
		if_end: LabelName,
	) {
		self.push(SemanticStackContext::IfConditionExpression {
			expr_result,
			if_begin,
			if_end,
		});
	}

	fn condition_expression(
		&mut self,
		left_result: ExpressionResult,
		right_result: ExpressionResult,
		condition: Condition,
		register_number: u64,
	) {
		self.push(SemanticStackContext::ConditionExpression {
			left_result,
			right_result,
			condition,
			register_number,
		});
	}

	fn jump_function_return(&mut self, expr_result: ExpressionResult) {
		self.push(SemanticStackContext::JumpFunctionReturn { expr_result });
	}

	fn logic_condition(
		&mut self,
		logic_condition: LogicCondition,
		left_register_result: u64,
		right_register_result: u64,
		register_number: u64,
	) {
		self.push(SemanticStackContext::LogicCondition {
			logic_condition,
			left_register_result,
			right_register_result,
			register_number,
		});
	}

	fn if_condition_logic(&mut self, if_begin: LabelName, if_end: LabelName, result_register: u64) {
		self.push(SemanticStackContext::IfConditionLogic {
			if_begin,
			if_end,
			result_register,
		});
	}

	fn function_arg(&mut self, value: Value, func_arg: FunctionParameter) {
		self.push(SemanticStackContext::FunctionArg { value, func_arg });
	}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum SemanticStackContext<I: SemanticContextInstruction> {
	ExpressionValue {
		expression: Value,
		register_number: u64,
	},
	ExpressionConst {
		expression: Constant,
		register_number: u64,
	},
	ExpressionStructValue {
		expression: Value,
		register_number: u64,
		index: u32,
	},
	ExpressionOperation {
		operation: ExpressionOperations,
		left_value: ExpressionResult,
		right_value: ExpressionResult,
		register_number: u64,
	},
	Call {
		call: Function,
		params: Vec<ExpressionResult>,
		register_number: u64,
	},
	LetBinding {
		let_decl: Value,
		expr_result: ExpressionResult,
	},
	Binding {
		value: Value,
		expr_result: ExpressionResult,
	},
	FunctionDeclaration {
		fn_decl: FunctionStatement,
	},
	Constant {
		const_decl: Constant,
	},
	Types {
		type_decl: StructTypes,
	},
	ExpressionFunctionReturn {
		expr_result: ExpressionResult,
	},
	ExpressionFunctionReturnWithLabel {
		expr_result: ExpressionResult,
	},
	SetLabel {
		label: LabelName,
	},
	JumpTo {
		label: LabelName,
	},
	IfConditionExpression {
		expr_result: ExpressionResult,
		if_begin: LabelName,
		if_end: LabelName,
	},
	ConditionExpression {
		left_result: ExpressionResult,
		right_result: ExpressionResult,
		condition: Condition,
		register_number: u64,
	},
	JumpFunctionReturn {
		expr_result: ExpressionResult,
	},
	LogicCondition {
		logic_condition: LogicCondition,
		left_register_result: u64,
		right_register_result: u64,
		register_number: u64,
	},
	IfConditionLogic {
		if_begin: LabelName,
		if_end: LabelName,
		result_register: u64,
	},
	FunctionArg {
		value: Value,
		func_arg: FunctionParameter,
	},
	ExtendedExpression(Box<I>),
}

pub trait GlobalSemanticContext {
	fn function_declaration(&mut self, fn_decl: FunctionStatement);

	fn constant(&mut self, const_decl: Constant);

	fn types(&mut self, type_decl: StructTypes);
}

pub trait SemanticContextInstruction: Clone + Debug + PartialEq {}

pub trait ExtendedExpression<I: SemanticContextInstruction>: Clone + Debug + PartialEq {
	fn expression(
		&self,
		state: &mut State<I, Self>,
		block_state: &Rc<RefCell<BlockState<I>>>,
	) -> ExpressionResult;
}

pub trait SemanticContext {
	fn expression_value(&mut self, expression: Value, register_number: u64);
	fn expression_const(&mut self, expression: Constant, register_number: u64);
	fn expression_struct_value(&mut self, expression: Value, index: u32, register_number: u64);
	fn expression_operation(
		&mut self,
		operation: ExpressionOperations,
		left_value: ExpressionResult,
		right_value: ExpressionResult,
		register_number: u64,
	);
	fn call(&mut self, call: Function, params: Vec<ExpressionResult>, register_number: u64);
	fn let_binding(&mut self, let_decl: Value, expr_result: ExpressionResult);
	fn binding(&mut self, val: Value, expr_result: ExpressionResult);
	fn expression_function_return(&mut self, expr_result: ExpressionResult);
	fn expression_function_return_with_label(&mut self, expr_result: ExpressionResult);
	fn set_label(&mut self, label: LabelName);
	fn jump_to(&mut self, label: LabelName);
	fn if_condition_expression(
		&mut self,
		expr_result: ExpressionResult,
		label_if_begin: LabelName,
		label_if_end: LabelName,
	);
	fn condition_expression(
		&mut self,
		left_result: ExpressionResult,
		right_result: ExpressionResult,
		condition: Condition,
		register_number: u64,
	);
	fn jump_function_return(&mut self, expr_result: ExpressionResult);
	fn logic_condition(
		&mut self,
		logic_condition: LogicCondition,
		left_register_result: u64,
		right_register_result: u64,
		register_number: u64,
	);
	fn if_condition_logic(
		&mut self,
		label_if_begin: LabelName,
		label_if_end: LabelName,
		result_register: u64,
	);
	fn function_arg(&mut self, value: Value, func_arg: FunctionParameter);
}

pub trait ExtendedSemanticContext<I: SemanticContextInstruction> {
	fn extended_expression(&mut self, expr: &I);
}
