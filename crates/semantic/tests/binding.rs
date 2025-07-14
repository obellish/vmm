mod utils;

use std::{cell::RefCell, rc::Rc};

use vmm_semantic::{
	PrimitiveTypes, PrimitiveValue,
	ast::{self, CodeLocation, GetLocation, GetName, Ident, ValueName},
	types::{
		Binding, BlockState, ExpressionResult, ExpressionResultValue, InnerValueName,
		SemanticStackContext, StateErrorType, Type, Value,
	},
};

use self::utils::{CustomExpression, CustomExpressionInstruction, SemanticTest};

#[test]
fn binding_transform() {
	let expr_ast = ast::Expression {
		value: ast::ExpressionValue::<
			CustomExpressionInstruction,
			CustomExpression<CustomExpressionInstruction>,
		>::Primitive(PrimitiveValue::U64(3)),
		operation: None,
	};
	let binding_ast = ast::Binding {
		name: ast::ValueName::new(Ident::new("x")),
		value: Box::new(expr_ast.clone()),
	};

	assert_eq!(binding_ast.location(), CodeLocation::new(1, 0));
	assert_eq!(binding_ast.name(), "x");

	let binding: Binding = binding_ast.clone().into();
	assert_eq!(binding.to_string(), "x");
	assert_eq!(binding.value, Box::new(expr_ast.into()));

	dbg!(format!("{binding_ast:?}"));
}

#[test]
fn binding_wrong_expression() {
	let block_state = Rc::new(RefCell::new(BlockState::new(None)));
	let mut t = SemanticTest::new();

	let expr = ast::Expression {
		value: ast::ExpressionValue::<
			CustomExpressionInstruction,
			CustomExpression<CustomExpressionInstruction>,
		>::ValueName(ast::ValueName::new(Ident::new("x"))),
		operation: None,
	};

	let binding = ast::Binding {
		name: ast::ValueName::new(Ident::new("x")),
		value: Box::new(expr),
	};

	t.state.binding(&binding, &block_state);
	assert_eq!(t.errors_len(), 1);
	assert!(t.check_error(StateErrorType::ValueNotFound));
}

#[test]
fn binding_value_not_exist() {
	let block_state = Rc::new(RefCell::new(BlockState::new(None)));
	let mut t = SemanticTest::new();

	let expr = ast::Expression {
		value: ast::ExpressionValue::<
			CustomExpressionInstruction,
			CustomExpression<CustomExpressionInstruction>,
		>::Primitive(PrimitiveValue::I16(23)),
		operation: None,
	};
	let binding = ast::Binding {
		name: ast::ValueName::new(Ident::new("x")),
		value: Box::new(expr),
	};

	t.state.binding(&binding, &block_state);
	assert_eq!(t.errors_len(), 1);
	assert!(t.check_error(StateErrorType::ValueNotFound));
}
