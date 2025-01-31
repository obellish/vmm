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

#[derive(Debug, Clone, Error)]
pub enum OutputError {
	#[error("overflow addresses contains invalid field element: {0}")]
	InvalidOverflowAddress(String),
	#[error("overflow addresses length is {0}, but expected {1}")]
	InvalidOverflowAddressLength(usize, usize),
	#[error("stack contains an invalid field element: {0}")]
	InvalidStackElement(String),
	#[error("too many elements for output stack, {0} elements")]
	OutputSizeTooBig(usize),
}

#[derive(Debug, Clone, Error)]
pub enum KernelError {
	#[error("kernel cannot have duplicated procedures")]
	DuplicatedProcedures,
	#[error("kernel can have at most {0} procedures, received {1}")]
	TooManyProcedures(usize, usize),
}
