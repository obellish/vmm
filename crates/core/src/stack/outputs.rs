use alloc::vec::Vec;
use core::ops::Deref;

use super::{MIN_STACK_DEPTH, get_num_stack_values};
use crate::{
	Felt, Word, ZERO,
	errors::OutputError,
	utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable, range},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct StackOutputs {
	elements: [Felt; MIN_STACK_DEPTH],
}

impl StackOutputs {
	pub fn new(mut stack: Vec<Felt>) -> Result<Self, OutputError> {
		if stack.len() > MIN_STACK_DEPTH {
			return Err(OutputError::OutputSizeTooBig(stack.len()));
		}
		stack.resize(MIN_STACK_DEPTH, ZERO);

		Ok(Self {
			elements: stack.try_into().unwrap(),
		})
	}

	pub fn try_from_ints(iter: impl IntoIterator<Item = u64>) -> Result<Self, OutputError> {
		let stack = iter
			.into_iter()
			.map(Felt::try_from)
			.collect::<Result<_, _>>()
			.map_err(OutputError::InvalidStackElement)?;

		Self::new(stack)
	}

	#[must_use]
	pub fn get_stack_item(&self, idx: usize) -> Option<Felt> {
		self.elements.get(idx).copied()
	}

	#[must_use]
	pub fn get_stack_word(&self, idx: usize) -> Option<Word> {
		let word_elements: Word = {
			let word_elements: Vec<Felt> = range(idx, 4)
				.map(|idx| self.get_stack_item(idx))
				.rev()
				.collect::<Option<_>>()?;

			word_elements
				.try_into()
				.expect("a Word contains 4 elements")
		};

		Some(word_elements)
	}

	#[must_use]
	pub fn stack_truncated(&self, num_outputs: usize) -> &[Felt] {
		let len = self.elements.len().min(num_outputs);
		&self.elements[..len]
	}

	pub fn stack_mut(&mut self) -> &mut [Felt] {
		&mut self.elements
	}

	#[must_use]
	pub fn as_int_vec(&self) -> Vec<u64> {
		self.elements.iter().copied().map(|e| e.as_int()).collect()
	}
}

impl Deref for StackOutputs {
	type Target = [Felt; MIN_STACK_DEPTH];

	fn deref(&self) -> &Self::Target {
		&self.elements
	}
}

impl Deserializable for StackOutputs {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let num_elements = source.read_u8()?;

		let elements = source.read_many::<Felt>(num_elements.into())?;

		Self::new(elements).map_err(|_| {
			DeserializationError::InvalidValue(format!(
				"number of stack elements should not be greater than {MIN_STACK_DEPTH}, but {num_elements} was found"
			))
		})
	}
}

impl From<[Felt; MIN_STACK_DEPTH]> for StackOutputs {
	fn from(value: [Felt; MIN_STACK_DEPTH]) -> Self {
		Self { elements: value }
	}
}

impl Serializable for StackOutputs {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		let num_stack_values = get_num_stack_values(self);
		target.write_u8(num_stack_values);
		target.write_many(&self.elements[..num_stack_values as usize]);
	}
}
