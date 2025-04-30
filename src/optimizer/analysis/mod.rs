mod cell_access;

use std::{array, borrow::Cow};

use serde::{Deserialize, Serialize};
use tracing::trace;
use vmm_serde_array::BigArray;

pub use self::cell_access::*;
use crate::{ExecutionUnit, Instruction, Program, TAPE_SIZE};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analyzer<'a> {
	unit: &'a ExecutionUnit,
	cells: [CellAccess; TAPE_SIZE],
}

impl<'a> Analyzer<'a> {
	#[must_use]
	pub fn new(unit: &'a ExecutionUnit) -> Self {
		Self {
			unit,
			cells: array::from_fn(|_| CellAccess::default()),
		}
	}

	#[must_use]
	#[tracing::instrument(skip(self))]
	pub fn analyze(mut self) -> AnalysisResults {
		if let Some(Instruction::Add(n)) = self.unit.program().first().copied() {
			trace!("setting cell 0 to {}", n as u8);
			// self.cells[0] = CellAccess::Set(n as u8);
			self.cells[0] = if let CellAccess::Set(i) = self.cells[0] {
				CellAccess::Set(i + n as u8)
			} else {
				CellAccess::Set(n as u8)
			};
		}

		for instr in self.unit.program() {}

		AnalysisResults { cells: self.cells }
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisResults {
	#[serde(with = "BigArray")]
	cells: [CellAccess; TAPE_SIZE],
}
