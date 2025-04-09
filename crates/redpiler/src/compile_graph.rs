use petgraph::stable_graph::{NodeIndex, StableGraph};
use vmm_blocks::{
	BlockPos,
	blocks::{ComparatorMode, Instrument},
};
use vmm_redstone::bool_to_signal_strength;

#[derive(Debug, Default, Clone, Copy)]
pub struct NodeState {
	pub powered: bool,
	pub repeater_locked: bool,
	pub output_strength: u8,
}

impl NodeState {
	pub const fn simple(powered: bool) -> Self {
		Self {
			powered,
			output_strength: bool_to_signal_strength(powered),
			repeater_locked: false,
		}
	}

	pub const fn repeater(powered: bool, locked: bool) -> Self {
		Self {
			powered,
			repeater_locked: locked,
			output_strength: bool_to_signal_strength(powered),
		}
	}

	pub const fn signal_strength(ss: u8) -> Self {
		Self {
			output_strength: ss,
			powered: false,
			repeater_locked: false,
		}
	}

	pub const fn comparator(powered: bool, ss: u8) -> Self {
		Self {
			powered,
			output_strength: ss,
			repeater_locked: false,
		}
	}
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Annotations {}

#[derive(Debug, Clone, Copy)]
pub struct CompileNode {
	pub ty: NodeType,
	pub block: Option<(BlockPos, u32)>,
	pub state: NodeState,
	pub is_input: bool,
	pub is_output: bool,
	pub annotations: Annotations,
}

impl CompileNode {
	pub const fn is_removable(self) -> bool {
		matches!(
			self,
			Self {
				is_input: false,
				is_output: false,
				..
			}
		)
	}
}

#[derive(Debug, Clone, Copy)]
pub struct CompileLink {
	pub ty: LinkType,
	pub signal_strength: u8,
}

impl CompileLink {
	pub const fn new(ty: LinkType, signal_strength: u8) -> Self {
		Self {
			ty,
			signal_strength,
		}
	}

	pub const fn default(signal_strength: u8) -> Self {
		Self::new(LinkType::Default, signal_strength)
	}

	pub const fn side(signal_strength: u8) -> Self {
		Self::new(LinkType::Side, signal_strength)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
	Repeater {
		delay: u8,
		facing_diode: bool,
	},
	Torch,
	Comparator {
		mode: ComparatorMode,
		far_input: Option<u8>,
		facing_diode: bool,
	},
	Lamp,
	Button,
	Lever,
	PressurePlate,
	Trapdoor,
	Wire,
	Constant,
	NoteBlock {
		instrument: Instrument,
		note: u32,
	},
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkType {
	Default,
	Side,
}

pub type CompileGraph = StableGraph<CompileNode, CompileLink>;
