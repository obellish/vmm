use serde::{Deserialize, Serialize};

pub type NbtCompound = nbt::Map<String, nbt::Value>;

#[derive(Clone, Serialize, Deserialize)]
pub struct NbtMap<T> {
	#[serde(rename = "type")]
	kind: String,
	value: Vec<NbtMapEntry<T>>,
}

impl<T> NbtMap<T> {
	pub fn new(kind: impl Into<String>) -> Self {
		Self {
			kind: kind.into(),
			value: Vec::new(),
		}
	}

	pub fn push_element(&mut self, name: String, element: T) {
		let id = self.value.len() as i32;
		self.value.push(NbtMapEntry { name, id, element });
	}
}

#[derive(Clone, Serialize, Deserialize)]
struct NbtMapEntry<T> {
	name: String,
	id: i32,
	element: T,
}
