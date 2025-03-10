use std::{hash::Hash, iter::FromIterator, ops::Deref};

use rustc_hash::FxHashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct NodeRefs<'a, N>(FxHashSet<&'a N>)
where
	N: Clone + Eq + Hash;

impl<'a, N> Deref for NodeRefs<'a, N>
where
	N: Clone + Eq + Hash,
{
	type Target = FxHashSet<&'a N>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<'a, N> From<&'a N> for NodeRefs<'a, N>
where
	N: Clone + Eq + Hash,
{
	fn from(value: &'a N) -> Self {
		Self::from_iter([value])
	}
}

impl<'a, N> FromIterator<&'a N> for NodeRefs<'a, N>
where
	N: Clone + Eq + Hash,
{
	fn from_iter<T: IntoIterator<Item = &'a N>>(iter: T) -> Self {
		Self(iter.into_iter().collect())
	}
}

impl<'a, N> IntoIterator for NodeRefs<'a, N>
where
	N: Clone + Eq + Hash,
{
	type IntoIter = std::collections::hash_set::IntoIter<&'a N>;
	type Item = &'a N;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<'a, N> IntoIterator for &'a NodeRefs<'a, N>
where
	N: Clone + Eq + Hash,
{
	type IntoIter = std::iter::Copied<std::collections::hash_set::Iter<'a, &'a N>>;
	type Item = &'a N;

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter().copied()
	}
}

#[cfg(test)]
mod tests {
	use itertools::Itertools;
	use rustc_hash::FxHashSet;

	use super::NodeRefs;

	#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
	struct Node(u8);

	#[test]
	fn from_iter() {
		let nodes = [Node(1), Node(2), Node(3)];
		let refs = NodeRefs::from_iter(&nodes);
		assert_eq!(
			*refs,
			FxHashSet::from_iter([&nodes[0], &nodes[1], &nodes[2]])
		);
	}

	#[test]
	fn from_single_ref() {
		let node = Node(42);
		let refs = NodeRefs::from(&node);
		assert_eq!(*refs, FxHashSet::from_iter([&node]));
	}

	#[test]
	fn into_iter() {
		let nodes = [Node(3), Node(2), Node(1)];
		let refs = NodeRefs::from_iter(&nodes);

		let v = refs.into_iter().sorted().collect_vec();
		assert_eq!(v, [&nodes[2], &nodes[1], &nodes[0]]);
	}

	#[test]
	fn ref_into_iter() {
		let nodes = [Node(3), Node(2), Node(1)];
		let refs = NodeRefs::from_iter(&nodes);

		let v = (&refs).into_iter().sorted().collect_vec();
		assert_eq!(v, [&nodes[2], &nodes[1], &nodes[0]]);
	}
}
