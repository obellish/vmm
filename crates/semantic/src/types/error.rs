use serde::{Deserialize, Serialize};

use crate::ast::CodeLocation;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateError {
	kind: StateErrorType,
	location: CodeLocation,
}

impl StateError {
	#[must_use]
	pub const fn kind(&self) -> StateErrorType {
		self.kind
	}

	#[must_use]
	pub const fn location(&self) -> CodeLocation {
		self.location
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum StateErrorType {
	Common,
	ConstantAlreadyExists,
	ConstantNotFound,
	WrongLetType,
	WrongExpressionType,
	TypeAlreadyExists,
	FunctionAlreadyExists,
	ValueNotFound,
	ValueNotStruct,
	ValueNotStructField,
	ValueIsNotMutable,
	FunctionNotFound,
	FunctionParameterTypeWrong,
	ReturnNotFound,
	ReturnAlreadyCalled,
	IfElseDuplicated,
	TypeNotFound,
	WrongReturnType,
	ConditionExpressionWrongType,
	ConditionIsEmpty,
	ConditionExpressionNotSupported,
	ForbiddenCodeAfterReturnDeprecated,
	ForbiddenCodeAfterContinueDeprecated,
	ForbiddenCodeAfterBreakDeprecated,
	FunctionArgumentNameDuplicated,
}
