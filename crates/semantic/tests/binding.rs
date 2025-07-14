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
        value: Box::new(expr_ast.clone())
    };

    assert_eq!(binding_ast.location(), CodeLocation::new(1, 0));
    assert_eq!(binding_ast.name(), "x");

    let binding: Binding = binding_ast.clone().into();
    assert_eq!(binding.to_string(), "x");
    assert_eq!(binding.value, Box::new(expr_ast.into()));

    dbg!(format!("{binding_ast:?}"));
}
