use alloc::string::String;

use thiserror::Error;
use vmm_formatting::hex::DisplayHex;

#[derive(Debug, Clone, Error)]
pub enum InputError {
	#[error("{:#x} is a duplicate of the current merkle set", DisplayHex(.0.as_slice()))]
	DuplicateAdviceRoot([u8; 32]),
	#[error("number of input values can not exceed {0}, but {1} was provided")]
	InputLengthExceeded(usize, usize),
	#[error("{0} is not a valid field element: {1}")]
	NotFieldElement(u64, String),
}
