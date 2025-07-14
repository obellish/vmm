use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::ast::CodeLocation;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateError {
	kind: StateErrorType,
	location: CodeLocation,
	value: Option<String>,
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

	#[must_use]
	pub fn value(&self) -> Option<&str> {
		self.value.as_deref()
	}

	pub(crate) const fn new(kind: StateErrorType, location: CodeLocation) -> Self {
		Self::create(kind, location, None)
	}

	pub(crate) fn with_value(
		kind: StateErrorType,
		location: CodeLocation,
		value: impl Into<String>,
	) -> Self {
		Self::create(kind, location, Some(value.into()))
	}

	const fn create(kind: StateErrorType, location: CodeLocation, value: Option<String>) -> Self {
		Self {
			kind,
			location,
			value,
		}
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
