use std::{
	collections::{BTreeMap, BTreeSet, HashMap, HashSet},
	hash::{BuildHasher, Hash},
	io::Write,
};

use super::cautious_capacity;
use crate::{Decode, Encode, ProtocolError, VarInt};

impl<'a, T> Decode<'a> for BTreeSet<T>
where
	T: Decode<'a> + Ord,
{
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		let len = VarInt::decode(r)?.0;

		assert!(
			len >= 0,
			"attempt to decode B-tree set with negative length"
		);
		let len = len as usize;

		let mut set = Self::new();

		for _ in 0..len {
			assert!(
				set.insert(T::decode(r)?),
				"encountered duplicate item while decoding B-tree set"
			);
		}

		Ok(set)
	}
}

impl<T: Encode> Encode for BTreeSet<T> {
	fn encode(&self, mut w: impl Write) -> Result<(), ProtocolError> {
		let len = self.len();

		assert!(
			i32::try_from(len).is_ok(),
			"length of B-tree set ({len}) exceeds i32::MAX"
		);

		VarInt(len as i32).encode(&mut w)?;

		for val in self {
			val.encode(&mut w)?;
		}

		Ok(())
	}
}

impl<'a, T, S> Decode<'a> for HashSet<T, S>
where
	T: Decode<'a> + Eq + Hash,
	S: BuildHasher + Default,
{
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		let len = VarInt::decode(r)?.0;
		assert!(len >= 0, "attempt to decode hash set with negative length");
		let len = len as usize;

		let mut set = Self::with_capacity_and_hasher(cautious_capacity::<T>(len), S::default());

		for _ in 0..len {
			assert!(
				set.insert(T::decode(r)?),
				"encountered duplicate item while decoding hash set"
			);
		}

		Ok(set)
	}
}

impl<T: Encode, S> Encode for HashSet<T, S> {
	fn encode(&self, mut w: impl Write) -> Result<(), ProtocolError> {
		let len = self.len();

		assert!(
			i32::try_from(len).is_ok(),
			"length of hash set ({len}) exceeds i32::MAX"
		);

		VarInt(len as i32).encode(&mut w)?;

		for val in self {
			val.encode(&mut w)?;
		}

		Ok(())
	}
}

impl<'a, K, V> Decode<'a> for BTreeMap<K, V>
where
	K: Decode<'a> + Ord,
	V: Decode<'a>,
{
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		let len = VarInt::decode(r)?.0;
		assert!(
			len >= 0,
			"attempt to decode B-tree map with negative length"
		);
		let len = len as usize;

		let mut map = Self::new();

		for _ in 0..len {
			assert!(
				map.insert(K::decode(r)?, V::decode(r)?).is_none(),
				"encountered duplicate key while decoding B-tree map"
			);
		}

		Ok(map)
	}
}

impl<K: Encode, V: Encode> Encode for BTreeMap<K, V> {
	fn encode(&self, mut w: impl Write) -> Result<(), ProtocolError> {
		let len = self.len();

		assert!(
			i32::try_from(len).is_ok(),
			"length of B-tree map ({len}) exceeds i32::MAX"
		);

		VarInt(len as i32).encode(&mut w)?;
		for pair in self {
			pair.encode(&mut w)?;
		}

		Ok(())
	}
}

impl<'a, K, V, S> Decode<'a> for HashMap<K, V, S>
where
	K: Decode<'a> + Eq + Hash,
	V: Decode<'a>,
	S: BuildHasher + Default,
{
	fn decode(r: &mut &'a [u8]) -> Result<Self, ProtocolError> {
		let len = VarInt::decode(r)?.0;
		assert!(len >= 0, "attempt to decode hash map with negative length");
		let len = len as usize;

		let mut map =
			Self::with_capacity_and_hasher(cautious_capacity::<(K, V)>(len), S::default());

		for _ in 0..len {
			assert!(
				map.insert(K::decode(r)?, V::decode(r)?).is_none(),
				"encountered duplicate item while decoding hash map"
			);
		}

		Ok(map)
	}
}

impl<K: Encode, V: Encode, S> Encode for HashMap<K, V, S> {
	fn encode(&self, mut w: impl Write) -> Result<(), ProtocolError> {
		let len = self.len();

		assert!(
			i32::try_from(len).is_ok(),
			"length of hash map ({len}) exceeds i32::MAX"
		);

		VarInt(len as i32).encode(&mut w)?;

		for pair in self {
			pair.encode(&mut w)?;
		}

		Ok(())
	}
}
