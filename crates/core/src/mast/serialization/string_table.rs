use alloc::{collections::BTreeMap, string::String, sync::Arc, vec::Vec};
use core::cell::RefCell;

use super::{StringDataOffset, StringIndex};
use crate::{
	crypto::hash::{Blake3_256, Blake3Digest},
	utils::{
		ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable, SliceReader,
	},
};

pub struct StringTable {
	data: Vec<u8>,
	table: Vec<StringIndex>,
	refc_strings: Vec<RefCell<Option<Arc<str>>>>,
}

impl StringTable {
	pub fn new(data: Vec<u8>, table: Vec<StringIndex>) -> Self {
		let mut refc_strings = Vec::with_capacity(table.len());
		refc_strings.resize(table.len(), RefCell::new(None));

		Self {
			data,
			table,
			refc_strings,
		}
	}

	pub fn read_arc_string(&self, str_idx: StringIndex) -> Result<Arc<str>, DeserializationError> {
		if let Some(cached) = self
			.refc_strings
			.get(str_idx)
			.and_then(|cell| cell.borrow().clone())
		{
			return Ok(cached);
		}

		let string = Arc::from(self.read_string(str_idx)?);
		*self.refc_strings[str_idx].borrow_mut() = Some(Arc::clone(&string));
		Ok(string)
	}

	pub fn read_string(&self, str_idx: StringIndex) -> Result<String, DeserializationError> {
		let str_offset = self.table.get(str_idx).copied().ok_or_else(|| {
			DeserializationError::InvalidValue(format!("invalid index in strings table: {str_idx}"))
		})?;

		let mut reader = SliceReader::new(&self.data[str_offset..]);
		reader.read()
	}
}

impl Deserializable for StringTable {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let table = source.read()?;
		let data = source.read()?;

		Ok(Self::new(data, table))
	}
}

impl Serializable for StringTable {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		let Self { table, data, .. } = self;

		table.write_into(target);
		data.write_into(target);
	}
}

#[derive(Debug, Default)]
pub struct StringTableBuilder {
	table: Vec<StringDataOffset>,
	str_to_index: BTreeMap<Blake3Digest<32>, StringIndex>,
	strings_data: Vec<u8>,
}

impl StringTableBuilder {
	pub fn add_string(&mut self, string: &str) -> StringIndex {
		if let Some(str_idx) = self.str_to_index.get(&Blake3_256::hash(string.as_bytes())) {
			*str_idx
		} else {
			let str_offset = self.strings_data.len();

			assert!(
				str_offset + string.len() < u32::MAX as usize,
				"strings table larger than 2^32 bytes"
			);

			let str_idx = self.table.len();

			string.write_into(&mut self.strings_data);
			self.table.push(str_offset);
			self.str_to_index
				.insert(Blake3_256::hash(string.as_bytes()), str_idx);

			str_idx
		}
	}

	pub fn into_table(self) -> StringTable {
		StringTable::new(self.strings_data, self.table)
	}
}
