use alloc::collections::{BTreeMap, BTreeSet};
use core::cell::RefCell;

pub trait KvMap<K, V: Clone>:
	Extend<(K, V)> + FromIterator<(K, V)> + IntoIterator<Item = (K, V)>
where
	K: Clone + Ord,
{
	fn get(&self, key: &K) -> Option<&V>;

	fn contains_key(&self, key: &K) -> bool {
		self.get(key).is_some()
	}

	fn len(&self) -> usize;

	fn is_empty(&self) -> bool {
		matches!(self.len(), 0)
	}

	fn insert(&mut self, key: K, value: V) -> Option<V>;

	fn remove(&mut self, key: &K) -> Option<V>;

	fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a K, &'a V)>
	where
		K: 'a,
		V: 'a;
}

impl<K, V: Clone> KvMap<K, V> for BTreeMap<K, V>
where
	K: Clone + Ord,
{
	fn get(&self, key: &K) -> Option<&V> {
		self.get(key)
	}

	fn contains_key(&self, key: &K) -> bool {
		self.contains_key(key)
	}

	fn len(&self) -> usize {
		self.len()
	}

	fn insert(&mut self, key: K, value: V) -> Option<V> {
		self.insert(key, value)
	}

	fn remove(&mut self, key: &K) -> Option<V> {
		self.remove(key)
	}

	fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a K, &'a V)>
	where
		K: 'a,
		V: 'a,
	{
		self.iter()
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RecordingMap<K, V> {
	data: BTreeMap<K, V>,
	updates: BTreeSet<K>,
	trace: RefCell<BTreeMap<K, V>>,
}

impl<K, V: Clone> RecordingMap<K, V>
where
	K: Clone + Ord,
{
	pub const fn inner(&self) -> &BTreeMap<K, V> {
		&self.data
	}

	pub fn finalize(self) -> (BTreeMap<K, V>, BTreeMap<K, V>) {
		(self.data, self.trace.take())
	}
}

impl<K, V: Clone> Extend<(K, V)> for RecordingMap<K, V>
where
	K: Clone + Ord,
{
	fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
		iter.into_iter().for_each(move |(k, v)| {
			self.insert(k, v);
		});
	}
}

impl<K, V: Clone> FromIterator<(K, V)> for RecordingMap<K, V>
where
	K: Clone + Ord,
{
	fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
		Self {
			data: iter.into_iter().collect(),
			updates: BTreeSet::new(),
			trace: RefCell::new(BTreeMap::new()),
		}
	}
}

impl<K, V: Clone> IntoIterator for RecordingMap<K, V>
where
	K: Clone + Ord,
{
	type IntoIter = alloc::collections::btree_map::IntoIter<K, V>;
	type Item = (K, V);

	fn into_iter(self) -> Self::IntoIter {
		self.data.into_iter()
	}
}

impl<K, V: Clone> KvMap<K, V> for RecordingMap<K, V>
where
	K: Clone + Ord,
{
	fn get(&self, key: &K) -> Option<&V> {
		self.data.get(key).inspect(|&value| {
			if !self.updates.contains(key) {
				self.trace.borrow_mut().insert(key.clone(), value.clone());
			}
		})
	}

	fn len(&self) -> usize {
		self.data.len()
	}

	fn insert(&mut self, key: K, value: V) -> Option<V> {
		let new_update = self.updates.insert(key.clone());
		self.data.insert(key.clone(), value).inspect(|old_value| {
			if new_update {
				self.trace.borrow_mut().insert(key, old_value.clone());
			}
		})
	}

	fn remove(&mut self, key: &K) -> Option<V> {
		self.data.remove(key).inspect(|old_value| {
			let new_update = self.updates.insert(key.clone());
			if new_update {
				self.trace
					.borrow_mut()
					.insert(key.clone(), old_value.clone());
			}
		})
	}

	fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a K, &'a V)>
	where
		K: 'a,
		V: 'a,
	{
		self.data.iter()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const ITEMS: [(u64, u64); 5] = [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4)];

	#[test]
	fn get_item() {
		let map = RecordingMap::from_iter(ITEMS);

		let get_items = [0, 1, 2];
		for key in &get_items {
			assert!(map.get(key).is_some());
		}

		let (_, proof) = map.finalize();

		for (key, value) in ITEMS {
			if get_items.contains(&key) {
				assert_eq!(proof.get(&key), Some(&value));
			} else {
				assert_eq!(proof.get(&key), None);
			}
		}
	}

	#[test]
	fn contains_key() {
		let map = RecordingMap::from_iter(ITEMS);

		let get_items = [0, 1, 2];
		for key in get_items {
			assert!(map.contains_key(&key));
		}

		let (_, proof) = map.finalize();

		for (key, _) in ITEMS {
			if get_items.contains(&key) {
				assert!(proof.contains_key(&key));
			} else {
				assert!(!proof.contains_key(&key));
			}
		}
	}

	#[test]
	fn len() {
		let mut map = RecordingMap::from_iter(ITEMS);

		assert_eq!(map.len(), ITEMS.len());

		assert_eq!(map.insert(4, 5), Some(4));
		assert_eq!(map.len(), ITEMS.len());
		assert_eq!(map.trace.borrow().len(), 1);
		assert_eq!(map.updates.len(), 1);

		assert!(map.insert(5, 5).is_none());
		assert_eq!(map.len(), ITEMS.len() + 1);
		assert_eq!(map.trace.borrow().len(), 1);
		assert_eq!(map.updates.len(), 2);

		let get_items = [0, 1, 2];
		for key in get_items {
			assert!(map.contains_key(&key));
		}

		assert_eq!(map.trace.borrow().len(), 4);
		assert_eq!(map.updates.len(), 2);

		let get_items = [0, 1, 2];
		for key in get_items {
			assert!(map.contains_key(&key));
		}

		assert_eq!(map.trace.borrow().len(), 4);
		assert_eq!(map.updates.len(), 2);

		let _ = map.get(&5).unwrap();
		assert_eq!(map.trace.borrow().len(), 4);
		assert_eq!(map.updates.len(), 2);

		map.insert(5, 11);
		assert_eq!(map.trace.borrow().len(), 4);
		assert_eq!(map.updates.len(), 2);

		let (_, proof) = map.finalize();

		assert_eq!(proof.len(), get_items.len() + 1);
	}

	#[test]
	fn iter() {
		let mut map = RecordingMap::from_iter(ITEMS);
		assert!(map.iter().all(|(x, y)| ITEMS.contains(&(*x, *y))));

		let new_value = 5;
		map.insert(4, 5);
		assert_eq!(map.iter().count(), ITEMS.len());
		assert!(map.iter().all(|(x, y)| if matches!(*x, 4) {
			y == &new_value
		} else {
			ITEMS.contains(&(*x, *y))
		}));
	}

	#[test]
	fn is_empty() {
		let empty_map = RecordingMap::<u64, u64>::default();
		assert!(empty_map.is_empty());

		let map = RecordingMap::from_iter(ITEMS);
		assert!(!map.is_empty());
	}

	#[test]
	#[allow(clippy::unnecessary_get_then_check)]
	fn remove() {
		let mut map = RecordingMap::from_iter(ITEMS);

		let key = 0;
		let value = map.remove(&key).unwrap();
		assert_eq!(value, ITEMS[0].1);
		assert_eq!(map.len(), ITEMS.len() - 1);
		assert_eq!(map.trace.borrow().len(), 1);
		assert_eq!(map.updates.len(), 1);

		let key = 0;
		let value = 0;
		map.insert(key, value);
		let value = map.remove(&key).unwrap();
		assert_eq!(value, 0);
		assert_eq!(map.len(), ITEMS.len() - 1);
		assert_eq!(map.trace.borrow().len(), 1);
		assert_eq!(map.updates.len(), 1);

		let key = 100;
		let value = map.remove(&key);
		assert!(value.is_none());
		assert_eq!(map.len(), ITEMS.len() - 1);
		assert_eq!(map.trace.borrow().len(), 1);
		assert_eq!(map.updates.len(), 1);

		let key = 100;
		let value = 100;
		map.insert(key, value);
		let value = map.remove(&key).unwrap();
		assert_eq!(value, 100);
		assert_eq!(map.len(), ITEMS.len() - 1);
		assert_eq!(map.trace.borrow().len(), 1);
		assert_eq!(map.updates.len(), 2);

		let (_, proof) = map.finalize();

		for (key, value) in ITEMS {
			match key {
				0 => assert_eq!(proof.get(&key), Some(&value)),
				_ => assert!(proof.get(&key).is_none()),
			}
		}
	}
}
