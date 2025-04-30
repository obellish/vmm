use std::borrow::Cow;

use tracing::trace;

use crate::{
	AnalysisResults, CellAccess, CellAccessType, Change, ExecutionUnit, Instruction, Pass,
	TAPE_SIZE,
};

// Currently only runs on the beginning cell, but can be expanded once cell analysis is introduced.
#[derive(Debug)]
pub struct SetCells;

impl Pass for SetCells {
	type State = AnalysisResults;

	fn run_pass(&mut self, unit: &mut ExecutionUnit, state: AnalysisResults) -> bool {
		let mut progress = false;

		for (i, cell) in state.cells.iter().cloned().enumerate() {
			if let CellAccessType::Set(v) = cell.kind {
				unit.program_mut().as_raw().remove(cell.instruction_index);

				unit.tape_mut()[i] += v;

				progress |= true;
			}
		}

		progress
	}

	fn name(&self) -> std::borrow::Cow<'static, str> {
		Cow::Borrowed("set untouched cells")
	}
}
