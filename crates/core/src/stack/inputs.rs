use alloc::vec::Vec;
use core::{ops::Deref, slice};

use super::{MIN_STACK_DEPTH, get_num_stack_values};
use crate::{
	Felt, ZERO,
	errors::InputError,
	utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable},
};

#[derive(Debug, Default, Clone)]
#[repr(transparent)]
pub struct StackInputs {
	elements: [Felt; MIN_STACK_DEPTH],
}

impl StackInputs {
	pub fn new(mut values: Vec<Felt>) -> Result<Self, InputError> {
		if values.len() > MIN_STACK_DEPTH {
			return Err(InputError::InputLengthExceeded(
				MIN_STACK_DEPTH,
				values.len(),
			));
		}
		values.reverse();
		values.resize(MIN_STACK_DEPTH, ZERO);

		Ok(Self {
			elements: values.try_into().unwrap(),
		})
	}

	pub fn try_from_ints(iter: impl IntoIterator<Item = u64>) -> Result<Self, InputError> {
		let values = iter
			.into_iter()
			.map(|v| Felt::try_from(v).map_err(|e| InputError::NotFieldElement(v, e)))
			.collect::<Result<_, _>>()?;

		Self::new(values)
	}

	pub fn iter(&self) -> slice::Iter<'_, Felt> {
		self.elements.iter()
	}
}

impl Deref for StackInputs {
	type Target = [Felt; MIN_STACK_DEPTH];

	fn deref(&self) -> &Self::Target {
		&self.elements
	}
}

impl Deserializable for StackInputs {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let num_elements = source.read_u8()?;

		let mut elements = source.read_many::<Felt>(num_elements.into())?;
		elements.reverse();

		Self::new(elements).map_err(|_| {
			DeserializationError::InvalidValue(format!(
				"number of stack elements should not be greater than {MIN_STACK_DEPTH}, but {num_elements} was found"
			))
		})
	}
}

impl From<[Felt; MIN_STACK_DEPTH]> for StackInputs {
	fn from(value: [Felt; MIN_STACK_DEPTH]) -> Self {
		Self { elements: value }
	}
}

impl<'a> IntoIterator for &'a StackInputs {
	type IntoIter = slice::Iter<'a, Felt>;
	type Item = &'a Felt;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl IntoIterator for StackInputs {
	type IntoIter = core::array::IntoIter<Felt, MIN_STACK_DEPTH>;
	type Item = Felt;

	fn into_iter(self) -> Self::IntoIter {
		self.elements.into_iter()
	}
}

impl Serializable for StackInputs {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		let num_stack_values = get_num_stack_values(self);
		target.write_u8(num_stack_values);
		target.write_many(&self.elements[..num_stack_values as usize]);
	}
}
