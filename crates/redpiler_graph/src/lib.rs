#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

use std::io::Write;

use bincode::{BincodeRead, Result};
use serde::{Deserialize, Serialize};

pub type NodeId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockPos {
	pub x: i32,
	pub y: i32,
	pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
	pub ty: LinkType,
	pub weight: u8,
	pub to: NodeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeState {
	pub powered: bool,
	pub repeated_locked: bool,
	pub output_strength: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
	pub ty: NodeType,
	pub blocks: Option<(BlockPos, u32)>,
	pub state: NodeState,
	pub facing_diode: bool,
	pub comparator_far_input: Option<u8>,
	pub inputs: Vec<Link>,
	pub updates: Vec<NodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkType {
	Default,
	Side,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparatorMode {
	Compare,
	Subtract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
	Repeater(u8),
	Torch,
	Comparator(ComparatorMode),
	Lamp,
	Button,
	Lever,
	PressurePlate,
	Trapdoor,
	Wire,
	Constant,
	NoteBlock,
}

pub fn serialize(nodes: &[Node]) -> Result<Vec<u8>> {
	bincode::serialize(nodes)
}

pub fn serialize_into(writer: impl Write, value: &[Node]) -> Result<()> {
	bincode::serialize_into(writer, value)
}

pub fn deserialize(bytes: &[u8]) -> Result<Vec<Node>> {
	bincode::deserialize(bytes)
}

pub fn deserialize_from<'a>(reader: impl BincodeRead<'a>) -> Result<Vec<Node>> {
	bincode::deserialize_from(reader)
}
