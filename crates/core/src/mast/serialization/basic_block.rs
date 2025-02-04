use alloc::vec::Vec;

use super::NodeDataOffset;
use crate::{
	Operation,
	mast::BasicBlockNode,
	utils::{ByteReader, DeserializationError, Serializable, SliceReader},
};

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct BasicBlockDataBuilder {
	node_data: Vec<u8>,
}

impl BasicBlockDataBuilder {
	pub const fn new() -> Self {
		Self {
			node_data: Vec::new(),
		}
	}

	#[allow(clippy::collection_is_never_read)]
	pub fn encode_last_block(&mut self, basic_block: &BasicBlockNode) -> NodeDataOffset {
		let ops_offset = self.node_data.len() as NodeDataOffset;

		let operations = basic_block.operations().copied().collect::<Vec<_>>();
		operations.write_into(&mut self.node_data);

		ops_offset
	}

	pub fn finalize(self) -> Vec<u8> {
		self.node_data
	}
}

#[repr(transparent)]
pub struct BasicBlockDataDecoder<'a> {
	node_data: &'a [u8],
}

impl<'a> BasicBlockDataDecoder<'a> {
	pub const fn new(node_data: &'a [u8]) -> Self {
		Self { node_data }
	}

	pub fn decode_operations(
		&self,
		ops_offset: NodeDataOffset,
	) -> Result<Vec<Operation>, DeserializationError> {
		let mut ops_data_reader = SliceReader::new(&self.node_data[ops_offset as usize..]);
		let operations = ops_data_reader.read()?;

		Ok(operations)
	}
}
