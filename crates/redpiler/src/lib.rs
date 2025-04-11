#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

mod backend;
mod compile_graph;
mod passes;
mod task_monitor;

use std::{sync::Arc, time::Instant};

use tracing::{debug, error, trace, warn};
use vmm_blocks::{BlockPos, blocks::Block};
use vmm_world::{TickEntry, World, for_each_block_mut_optimized};

pub use self::task_monitor::TaskMonitor;
use self::{
	backend::{BackendDispatcher, JitBackend, direct::DirectBackend},
	passes::PassManager,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CompilerOptions {
	pub optimize: bool,
	pub export: bool,
	pub io_only: bool,
	pub update: bool,
	pub export_dot_graph: bool,
	pub wire_dot_out: bool,
	pub backend_variant: BackendVariant,
}

impl CompilerOptions {
	pub fn parse(str: &str) -> Self {
		let mut co = Self::default();

		let options = str.split_whitespace();
		for option in options {
			if option.starts_with("--") {
				match option {
					"--optimize" => co.optimize = true,
					"--export" => co.export = true,
					"--io-only" => co.io_only = true,
					"--update" => co.update = true,
					"--export-dot" => co.export_dot_graph = true,
					"--wire-dot-out" => co.wire_dot_out = true,
					_ => warn!("unrecognized option: {option}"),
				}
			} else if let Some(str) = option.strip_prefix('-') {
				for c in str.chars() {
					let lower = c.to_lowercase().to_string();
					match lower.as_str() {
						"o" => co.optimize = true,
						"e" => co.export = true,
						"i" => co.io_only = true,
						"u" => co.update = true,
						"d" => co.wire_dot_out = true,
						_ => warn!("unrecognized option: -{c}"),
					}
				}
			} else {
				warn!("unrecognized option: {option}");
			}
		}

		co
	}
}

#[derive(Default)]
pub struct Compiler {
	is_active: bool,
	jit: Option<BackendDispatcher>,
	options: CompilerOptions,
}

impl Compiler {
	#[must_use]
	pub const fn is_active(&self) -> bool {
		self.is_active
	}

	#[must_use]
	pub const fn current_flags(&self) -> Option<CompilerOptions> {
		if self.is_active() {
			Some(self.options)
		} else {
			None
		}
	}

	pub fn use_jit(&mut self, jit: BackendDispatcher) {
		self.jit = Some(jit);
	}

	pub fn compile<W: World>(
		&mut self,
		world: &W,
		bounds: (BlockPos, BlockPos),
		options: CompilerOptions,
		ticks: Vec<TickEntry>,
		monitor: Arc<TaskMonitor>,
	) {
		debug!("starting compile");
		let start = Instant::now();

		let input = CompilerInput { world, bounds };
		let pass_manager = PassManager::<W>::default();
		let graph = pass_manager.run_passes(options, &input, monitor.clone());

		if monitor.cancelled() {
			return;
		}

		let replace_jit = match self.jit {
			Some(BackendDispatcher::DirectBackend(_)) => {
				options.backend_variant != BackendVariant::Direct
			}
			None => true,
		};

		if replace_jit {
			debug!("switching JIT backend to {:?}", options.backend_variant);
			let jit = match options.backend_variant {
				BackendVariant::Direct => {
					BackendDispatcher::DirectBackend(DirectBackend::default())
				}
			};

			self.use_jit(jit);
		}

		if let Some(jit) = &mut self.jit {
			trace!("compiling backend");
			monitor.set_message("Compiling backend");
			let start = Instant::now();

			jit.compile(graph, ticks, options, monitor.clone());

			monitor.increment_progress();
			trace!("backend compiled in {:?}", start.elapsed());
		} else {
			error!("cannot execute without JIT variant selected");
		}

		self.options = options;
		self.is_active = true;
		debug!("compile completed in {:?}", start.elapsed());
	}

	pub fn reset<W: World>(&mut self, world: &mut W, bounds: (BlockPos, BlockPos)) {
		if self.is_active {
			self.is_active = false;
			if let Some(jit) = &mut self.jit {
				jit.reset(world, self.options.io_only);
			}
		}

		if self.options.update {
			let (first_pos, second_pos) = bounds;
			for_each_block_mut_optimized(world, first_pos, second_pos, |world, pos| {
				let block = world.get_block(pos);
				vmm_redstone::update(block, world, pos);
			});
		}

		self.options = CompilerOptions::default();
	}

	fn backend(&mut self) -> &mut BackendDispatcher {
		assert!(
			self.is_active,
			"tried to get redpiler backend when inactive"
		);

		if let Some(jit) = &mut self.jit {
			jit
		} else {
			panic!("redpiler is active but missing JIT backend")
		}
	}

	pub fn tick(&mut self) {
		self.backend().tick();
	}

	pub fn tickn(&mut self, ticks: u64) {
		self.backend().tickn(ticks);
	}

	pub fn on_use_block(&mut self, pos: BlockPos) {
		self.backend().on_use_block(pos);
	}

	pub fn set_pressure_plate(&mut self, pos: BlockPos, powered: bool) {
		self.backend().set_pressure_plate(pos, powered);
	}

	pub fn flush<W: World>(&mut self, world: &mut W) {
		let io_only = self.options.io_only;
		self.backend().flush(world, io_only);
	}

	pub fn inspect(&mut self, pos: BlockPos) {
		if let Some(backend) = &mut self.jit {
			backend.inspect(pos);
		} else {
			debug!("cannot inspect when backend is not running");
		}
	}

	pub fn has_pending_ticks(&mut self) -> bool {
		self.backend().has_pending_ticks()
	}
}

pub struct CompilerInput<'w, W: World> {
	pub world: &'w W,
	pub bounds: (BlockPos, BlockPos),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BackendVariant {
	#[default]
	Direct,
}

const fn block_powered_mut(block: &mut Block) -> Option<&mut bool> {
	Some(match block {
		Block::RedstoneComparator { comparator } => &mut comparator.powered,
		Block::RedstoneTorch { lit }
		| Block::RedstoneWallTorch { lit, .. }
		| Block::RedstoneLamp { lit } => lit,
		Block::RedstoneRepeater { repeater } => &mut repeater.powered,
		Block::Lever { lever } => &mut lever.powered,
		Block::StoneButton { button } => &mut button.powered,
		Block::StonePressurePlate { powered }
		| Block::IronTrapdoor { powered, .. }
		| Block::NoteBlock { powered, .. } => powered,
		_ => return None,
	})
}

#[cfg(test)]
mod tests {
	use super::CompilerOptions;

	#[test]
	fn parse_options() {
		let input = "-io -u --export";
		let expected = CompilerOptions {
			io_only: true,
			optimize: true,
			export: true,
			update: true,
			..Default::default()
		};

		let options = CompilerOptions::parse(input);

		assert_eq!(options, expected);
	}
}
