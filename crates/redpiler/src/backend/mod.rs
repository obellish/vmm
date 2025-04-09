pub mod direct;

use std::sync::Arc;

use enum_dispatch::enum_dispatch;
use vmm_blocks::BlockPos;
use vmm_world::{TickEntry, World};

use super::{CompilerOptions, TaskMonitor, compile_graph::CompileGraph};

#[enum_dispatch]
pub trait JitBackend {
	fn compile(
		&mut self,
		graph: CompileGraph,
		ticks: Vec<TickEntry>,
		options: CompilerOptions,
		monitor: Arc<TaskMonitor>,
	);

	fn tick(&mut self);

	fn on_use_block(&mut self, pos: BlockPos);
	fn set_pressure_plate(&mut self, pos: BlockPos, powered: bool);
	fn flush<W: World>(&mut self, world: &mut W, io_only: bool);
	fn reset<W: World>(&mut self, world: &mut W, io_only: bool);
	fn has_pending_ticks(&self) -> bool;

	fn inspect(&mut self, pos: BlockPos);

	fn tickn(&mut self, ticks: u64) {
		for _ in 0..ticks {
			self.tick();
		}
	}
}

pub enum BackendDispatcher {
	DirectBackend,
}
