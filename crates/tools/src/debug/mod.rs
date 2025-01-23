mod prepare;
mod run;

use vmm::board::{Bus, MotherBoard};

pub use self::{prepare::*, run::*};

#[must_use]
pub fn exec_vm(components: Vec<Box<dyn Bus>>, config: RunConfig) -> (MotherBoard, StoppedState) {
	let mut motherboard = prepare_vm(components);
	let status = run_vm(motherboard.cpu_mut(), config);
	(motherboard, status)
}
