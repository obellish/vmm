use alloc::{
	boxed::Box,
	collections::{BTreeMap, btree_map::IntoIter},
	vec::Vec,
};

use crate::{
	Felt,
	crypto::hash::RpoDigest,
	utils::{
		ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable,
		collections::KvMap,
	},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct AdviceMap(BTreeMap<RpoDigest, Vec<Felt>>);

#[allow(clippy::trivially_copy_pass_by_ref)]
impl AdviceMap {
	#[must_use]
	pub const fn new() -> Self {
		Self(BTreeMap::new())
	}

	pub fn get(&self, key: &RpoDigest) -> Option<&[Felt]> {
		self.0.get(key).map(Vec::as_slice)
	}

	pub fn insert(&mut self, key: RpoDigest, value: Vec<Felt>) -> Option<Vec<Felt>> {
		self.0.insert(key, value)
	}

	pub fn remove(&mut self, key: &RpoDigest) -> Option<Vec<Felt>> {
		self.0.remove(key)
	}

	#[must_use]
	pub fn len(&self) -> usize {
		self.0.len()
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}

impl Deserializable for AdviceMap {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let count = source.read_usize()?;
		let mut map = BTreeMap::new();
		for _ in 0..count {
			let (key, value) = source.read()?;
			map.insert(key, value);
		}

		Ok(Self(map))
	}
}

impl Extend<(RpoDigest, Vec<Felt>)> for AdviceMap {
	fn extend<T>(&mut self, iter: T)
	where
		T: IntoIterator<Item = (RpoDigest, Vec<Felt>)>,
	{
		self.0.extend(iter);
	}
}

impl From<BTreeMap<RpoDigest, Vec<Felt>>> for AdviceMap {
	fn from(value: BTreeMap<RpoDigest, Vec<Felt>>) -> Self {
		Self(value)
	}
}

impl FromIterator<(RpoDigest, Vec<Felt>)> for AdviceMap {
	fn from_iter<T>(iter: T) -> Self
	where
		T: IntoIterator<Item = (RpoDigest, Vec<Felt>)>,
	{
		Self(BTreeMap::from_iter(iter))
	}
}

impl IntoIterator for AdviceMap {
	type IntoIter = IntoIter<RpoDigest, Vec<Felt>>;
	type Item = (RpoDigest, Vec<Felt>);

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl KvMap<RpoDigest, Vec<Felt>> for AdviceMap {
	fn get(&self, key: &RpoDigest) -> Option<&Vec<Felt>> {
		self.0.get(key)
	}

	fn contains_key(&self, key: &RpoDigest) -> bool {
		self.0.contains_key(key)
	}

	fn len(&self) -> usize {
		self.len()
	}

	fn insert(&mut self, key: RpoDigest, value: Vec<Felt>) -> Option<Vec<Felt>> {
		self.insert(key, value)
	}

	fn remove(&mut self, key: &RpoDigest) -> Option<Vec<Felt>> {
		self.remove(key)
	}

	fn iter(&self) -> Box<dyn Iterator<Item = (&RpoDigest, &Vec<Felt>)> + '_> {
		Box::new(self.0.iter())
	}
}

impl Serializable for AdviceMap {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		target.write_usize(self.len());
		for (key, value) in &self.0 {
			target.write((key, value));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::AdviceMap;
	use crate::{
		Felt,
		crypto::hash::RpoDigest,
		utils::{Deserializable, DeserializationError, Serializable},
	};

	#[test]
	fn advice_map_serialization() -> Result<(), DeserializationError> {
		let mut map1 = AdviceMap::new();
		map1.insert(RpoDigest::default(), vec![
			Felt::from(1u32),
			Felt::from(2u32),
		]);

		let bytes = map1.to_bytes();

		let map2 = AdviceMap::read_from_bytes(&bytes)?;

		assert_eq!(map1, map2);

		Ok(())
	}
}
