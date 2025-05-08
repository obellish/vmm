use tracing::debug;

use crate::{Change, Instruction, PeepholePass};

#[derive(Debug, Default, Clone, Copy)]
pub struct InstrScanPass;

impl PeepholePass for InstrScanPass {
	const SIZE: usize = 6;

	fn run_pass(&mut self, window: &[Instruction]) -> Option<Change> {
		debug!("{window:?}");
		None
	}
}
