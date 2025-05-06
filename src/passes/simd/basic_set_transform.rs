use std::borrow::Cow;

use crate::{Change, Instruction, PeepholePass, SimdInstruction};

#[derive(Debug, Clone, Copy)]
pub struct BasicSimdClearTransformPass;

impl PeepholePass for BasicSimdClearTransformPass {
	const SIZE: usize = 3;

	fn run_pass(&self, window: &[Instruction]) -> Option<Change> {
		match window {
			[
				Instruction::Set(x),
				Instruction::MovePtr(1),
				Instruction::Set(y),
			] if *x == *y => Some(Change::ReplaceOne(Instruction::Simd(
				SimdInstruction::Set { len: 2, value: *x },
			))),
			[
				Instruction::Simd(SimdInstruction::Set {
					len: x_len,
					value: x,
				}),
				Instruction::MovePtr(1),
				Instruction::Simd(SimdInstruction::Set {
					value: y,
					len: y_len,
				}),
			] if *x == *y => Some(Change::ReplaceOne(Instruction::Simd(
				SimdInstruction::Set {
					len: x_len + y_len,
					value: *x,
				},
			))),
			_ => None,
		}
	}

	fn name(&self) -> Cow<'static, str> {
		Cow::Borrowed("basic simd clear transformation")
	}
}
