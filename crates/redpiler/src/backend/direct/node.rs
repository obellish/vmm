use std::{
	fmt::{Debug, Formatter, Result as FmtResult},
	num::NonZeroU8,
	ops::{Index, IndexMut},
};

use vmm_blocks::blocks::ComparatorMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NodeId(u32);

impl NodeId {
	pub const fn index(self) -> usize {
		self.0 as usize
	}

	pub const unsafe fn from_index(index: usize) -> Self {
		Self(index as u32)
	}
}

#[derive(Default)]
pub struct Nodes {
	pub nodes: Box<[Node]>,
}

impl Nodes {
	pub fn new(nodes: impl IntoIterator<Item = Node>) -> Self {
		Self {
			nodes: nodes.into_iter().collect(),
		}
	}

	pub fn get(&self, idx: usize) -> NodeId {
		if self.nodes.get(idx).is_some() {
			NodeId(idx as u32)
		} else {
			panic!("node index out of bounds: {idx}")
		}
	}

	pub fn inner(&self) -> &[Node] {
		&self.nodes
	}

	pub fn inner_mut(&mut self) -> &mut [Node] {
		&mut self.nodes
	}

	pub fn into_inner(self) -> Box<[Node]> {
		self.nodes
	}
}

impl Index<NodeId> for Nodes {
	type Output = Node;

	fn index(&self, index: NodeId) -> &Self::Output {
		unsafe { self.nodes.get_unchecked(index.0 as usize) }
	}
}

impl IndexMut<NodeId> for Nodes {
	fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
		unsafe { self.nodes.get_unchecked_mut(index.0 as usize) }
	}
}

#[derive(Debug, Clone)]
pub struct Node {
	pub ty: NodeType,
	pub default_inputs: NodeInput,
	pub side_inputs: NodeInput,
	pub updates: Vec<ForwardLink>,
	pub is_io: bool,
	pub powered: bool,
	pub locked: bool,
	pub output_power: u8,
	pub changed: bool,
	pub pending_tick: bool,
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct NonMaxU8(NonZeroU8);

impl NonMaxU8 {
	pub fn new(value: u8) -> Option<Self> {
		NonZeroU8::new(value + 1).map(Self)
	}

	pub const fn get(self) -> u8 {
		self.0.get() - 1
	}
}

#[repr(align(16))]
#[derive(Debug, Default, Clone)]
pub struct NodeInput {
	pub signal_strength_counts: [u8; 16],
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ForwardLink {
	data: u32,
}

impl ForwardLink {
	pub fn new(id: NodeId, side: bool, signal_strength: u8) -> Self {
		assert!(id.index() < (1 << 27));
		assert!(signal_strength < 15);
		Self {
			data: (id.index() as u32) << 5
				| if side { 1 << 4 } else { 0 }
				| u32::from(signal_strength),
		}
	}

	pub const fn node(self) -> NodeId {
		unsafe { NodeId::from_index((self.data >> 5) as usize) }
	}

	pub const fn side(self) -> bool {
		!matches!(self.data & (1 << 4), 0)
	}

	pub const fn signal_strength(self) -> u8 {
		(self.data & 0b1111) as u8
	}
}

impl Debug for ForwardLink {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("ForwardLink")
			.field("node", &self.node())
			.field("side", &self.side())
			.field("signal_strength", &self.signal_strength())
			.finish()
	}
}

#[derive(Debug, Clone, Copy)]
pub enum NodeType {
	Repeater {
		delay: u8,
		facing_diode: bool,
	},
	Torch,
	Comparator {
		mode: ComparatorMode,
		far_input: Option<NonMaxU8>,
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
		noteblock_id: u16,
	},
}
