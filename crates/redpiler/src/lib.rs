#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

mod backend;
mod compile_graph;
mod passes;
mod task_monitor;

use vmm_blocks::blocks::{Block, RedstoneComparator};

pub use self::task_monitor::TaskMonitor;

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
