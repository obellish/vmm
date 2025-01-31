mod inputs;
mod outputs;

pub use self::{inputs::StackInputs, outputs::StackOutputs};
use super::Felt;

pub const MIN_STACK_DEPTH: usize = 16;

#[allow(clippy::trivially_copy_pass_by_ref)]
fn get_num_stack_values(values: &[Felt; MIN_STACK_DEPTH]) -> u8 {
	let mut num_trailing_zeros = 0;
	for v in values.iter().rev() {
		if matches!(v.as_int(), 0) {
			num_trailing_zeros += 1;
		} else {
			break;
		}
	}
	(MIN_STACK_DEPTH - num_trailing_zeros) as u8
}

#[cfg(test)]
mod tests {
	use alloc::vec::Vec;

	use crate::{
		StackInputs, StackOutputs,
		utils::{Deserializable, Serializable},
	};

	#[test]
	fn inputs_simple() -> eyre::Result<()> {
		let source = Vec::from([5u64, 4, 3, 2, 1]);
		let mut serialized = Vec::new();
		let inputs = StackInputs::try_from_ints(source.clone())?;

		inputs.write_into(&mut serialized);

		let mut expected_serialized = Vec::new();
		expected_serialized.push(source.len() as u8);
		source
			.iter()
			.rev()
			.for_each(|v| expected_serialized.append(&mut v.to_le_bytes().to_vec()));

		assert_eq!(serialized, expected_serialized);

		let result = StackInputs::read_from_bytes(&serialized)?;

		assert_eq!(*inputs, *result);

		Ok(())
	}

	#[test]
	fn inputs_full() -> eyre::Result<()> {
		let source = Vec::from([16u64, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
		let mut serialized = Vec::new();
		let inputs = StackInputs::try_from_ints(source.clone())?;

		inputs.write_into(&mut serialized);

		let mut expected_serialized = Vec::new();
		expected_serialized.push(source.len() as u8);
		source
			.iter()
			.rev()
			.for_each(|v| expected_serialized.append(&mut v.to_le_bytes().to_vec()));

		assert_eq!(serialized, expected_serialized);

		let result = StackInputs::read_from_bytes(&serialized)?;

		assert_eq!(*inputs, *result);

		Ok(())
	}

	#[test]
	fn inputs_empty() -> eyre::Result<()> {
		let mut serialized = Vec::new();
		let inputs = StackInputs::try_from_ints([])?;

		inputs.write_into(&mut serialized);

		let expected_serialized = vec![0];

		assert_eq!(serialized, expected_serialized);

		let result = StackInputs::read_from_bytes(&serialized)?;

		assert_eq!(*inputs, *result);

		Ok(())
	}

	#[test]
	fn outputs_simple() -> eyre::Result<()> {
		let source = Vec::from(core::array::from_fn::<u64, 5, _>(|i| i as u64 + 1));
		let mut serialized = Vec::new();
		let inputs = StackOutputs::try_from_ints(source.clone())?;

		inputs.write_into(&mut serialized);

		let mut expected_serialized = Vec::new();
		expected_serialized.push(source.len() as u8);
		source
			.iter()
			.for_each(|v| expected_serialized.append(&mut v.to_le_bytes().to_vec()));

		assert_eq!(serialized, expected_serialized);

		let result = StackOutputs::read_from_bytes(&serialized)?;

		assert_eq!(*inputs, *result);

		Ok(())
	}

	#[test]
	fn outputs_full() -> eyre::Result<()> {
		let source = Vec::from(core::array::from_fn::<u64, 16, _>(|i| i as u64 + 1));
		let mut serialized = Vec::new();
		let inputs = StackOutputs::try_from_ints(source.clone())?;

		inputs.write_into(&mut serialized);

		let mut expected_serialized = Vec::new();
		expected_serialized.push(source.len() as u8);
		source
			.iter()
			.for_each(|v| expected_serialized.append(&mut v.to_le_bytes().to_vec()));

		assert_eq!(serialized, expected_serialized);

		let result = StackOutputs::read_from_bytes(&serialized)?;

		assert_eq!(*inputs, *result);

		Ok(())
	}

	#[test]
	fn outputs_empty() -> eyre::Result<()> {
		let mut serialized = Vec::new();
		let inputs = StackOutputs::try_from_ints([])?;

		inputs.write_into(&mut serialized);

		let expected_serialized = vec![0];

		assert_eq!(serialized, expected_serialized);

		let result = StackOutputs::read_from_bytes(&serialized)?;

		assert_eq!(*inputs, *result);

		Ok(())
	}
}
