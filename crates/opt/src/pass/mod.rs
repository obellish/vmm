mod runners;

use std::{fmt::Debug, ops::RangeInclusive};

use vmm_ir::Instruction;

pub use self::runners::*;
use super::Change;

pub trait Pass: Debug {
	fn run_pass(&mut self, program: &mut Vec<Instruction>) -> bool;

	fn should_run_on_dyn_loop(&self) -> bool {
		true
	}

	fn should_run_on_if(&self) -> bool {
		true
	}
}

pub trait PeepholePass {
	const SIZE: usize;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change>;

	#[allow(unused)]
	fn should_run(&self, window: &[Instruction]) -> bool {
		true
	}

	fn should_run_on_dyn_loop(&self) -> bool {
		true
	}

	fn should_run_on_if(&self) -> bool {
		true
	}
}

pub trait RangePeepholePass {
	const RANGE: RangeInclusive<usize>;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change>;

	#[allow(unused)]
	fn should_run(&self, window: &[Instruction]) -> bool {
		true
	}

	fn should_run_on_dyn_loop(&self) -> bool {
		true
	}

	fn should_run_on_if(&self) -> bool {
		true
	}
}

pub trait LoopPass {
	fn run_pass(&mut self, loop_values: &[Instruction]) -> Option<Change>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		(0, None)
	}

	#[allow(unused)]
	fn should_run(&self, loop_values: &[Instruction]) -> bool {
		true
	}
}
