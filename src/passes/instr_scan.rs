use std::borrow::Cow;

use tracing::debug;

use crate::{Change, Instruction, PeepholePass};

#[derive(Debug
)]
pub struct InstrScanPass;

impl PeepholePass for InstrScanPass {
    const SIZE: usize = 6;

    fn run_pass(&self, window: &[Instruction]) -> Option<Change> {
        debug!("{window:?}");
        None
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("scan instructions for patterns")
    }
}
