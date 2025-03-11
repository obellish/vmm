use std::{hash::Hash, iter::FusedIterator};

use indexmap::map::Entry::Vacant;

use super::reverse_path;
use crate::{FxIndexMap, FxIndexSet, NodeRefs};

pub struct BfsReachable<N, FN> {
	i: usize,
	seen: FxIndexSet<N>,
	successors: FN,
}

impl<N, FN> BfsReachable<N, FN> {
	pub fn remaining_nodes_low_bound(&self) -> usize {
		self.seen.len() - self.i
	}
}

impl<N, FN, IN> Iterator for BfsReachable<N, FN>
where
	N: Clone + Eq + Hash,
	FN: FnMut(&N) -> IN,
	IN: IntoIterator<Item = N>,
{
	type Item = N;

	fn next(&mut self) -> Option<Self::Item> {
		let n = self.seen.get_index(self.i)?.clone();
		for s in (self.successors)(&n) {
			self.seen.insert(s);
		}
		self.i += 1;
		Some(n)
	}
}

impl<N, FN, IN> FusedIterator for BfsReachable<N, FN>
where
	N: Clone + Eq + Hash,
	FN: FnMut(&N) -> IN,
	IN: IntoIterator<Item = N>,
{
}

pub fn bfs<'a, N, S, IN>(
	start: S,
	successors: impl FnMut(&N) -> IN,
	success: impl FnMut(&N) -> bool,
) -> Option<Vec<N>>
where
	N: Clone + Eq + Hash + 'a,
	S: Into<NodeRefs<'a, N>>,
	IN: IntoIterator<Item = N>,
{
	bfs_core(&start.into(), successors, success, true)
}

pub fn bfs_loop<'a, N, S, IN>(start: S, successors: impl FnMut(&N) -> IN) -> Option<Vec<N>>
where
	N: Clone + Eq + Hash + 'a,
	S: Into<NodeRefs<'a, N>>,
	IN: IntoIterator<Item = N>,
{
	let start = start.into();
	bfs_core(&start, successors, |n| start.contains(n), false)
}

pub fn bfs_bidirectional<'a, N, S, E, IN>(
	start: S,
	end: E,
	successors_fn: impl Fn(&N) -> IN,
	predecessors_fn: impl Fn(&N) -> IN,
) -> Option<Vec<N>>
where
	N: Clone + Eq + Hash + 'a,
	S: Into<NodeRefs<'a, N>>,
	E: Into<NodeRefs<'a, N>>,
	IN: IntoIterator<Item = N>,
{
	let start: NodeRefs<'a, N> = start.into();
	let end: NodeRefs<'a, N> = end.into();

	let mut predecessors = FxIndexMap::<_, Option<usize>>::default();
	predecessors.extend(start.into_iter().cloned().map(|n| (n, None)));
	let mut successors = FxIndexMap::<_, Option<usize>>::default();
	successors.extend(end.into_iter().cloned().map(|n| (n, None)));

	let mut i_forwards = 0;
	let mut i_backwards = 0;
	let middle = 'l: loop {
		for _ in 0..(predecessors.len() - i_forwards) {
			let node = predecessors.get_index(i_forwards)?.0;
			for successor_node in successors_fn(node) {
				if !predecessors.contains_key(&successor_node) {
					predecessors.insert(successor_node.clone(), Some(i_forwards));
				}

				if successors.contains_key(&successor_node) {
					break 'l Some(successor_node);
				}
			}
			i_forwards += 1;
		}

		for _ in 0..(successors.len() - i_backwards) {
			let node = successors.get_index(i_backwards)?.0;
			for predecessor_node in predecessors_fn(node) {
				if !successors.contains_key(&predecessor_node) {
					successors.insert(predecessor_node.clone(), Some(i_backwards));
				}

				if predecessors.contains_key(&predecessor_node) {
					break 'l Some(predecessor_node);
				}
			}
			i_backwards += 1;
		}

		if i_forwards == predecessors.len() && i_backwards == successors.len() {
			break 'l None;
		}
	};

	middle.map(|middle| {
		let mut path = Vec::new();
		let mut node = Some(middle.clone());
		while let Some(n) = node {
			path.push(n.clone());
			node = predecessors[&n]
				.and_then(|i| predecessors.get_index(i))
				.map(|n| n.0.clone());
		}

		path.reverse();

		let mut node = successors[&middle]
			.and_then(|i| successors.get_index(i))
			.map(|n| n.0.clone());
		while let Some(n) = node {
			path.push(n.clone());
			node = successors[&n]
				.and_then(|i| successors.get_index(i))
				.map(|n| n.0.clone());
		}

		path
	})
}

pub fn bfs_reach<N, FN, IN>(start: N, successors: FN) -> BfsReachable<N, FN>
where
	N: Clone + Eq + Hash,
	FN: FnMut(&N) -> IN,
	IN: IntoIterator<Item = N>,
{
	let mut seen = FxIndexSet::default();
	seen.insert(start);
	BfsReachable {
		i: 0,
		seen,
		successors,
	}
}

fn bfs_core<'a, N, IN>(
	start: &NodeRefs<'a, N>,
	mut successors: impl FnMut(&N) -> IN,
	mut success: impl FnMut(&N) -> bool,
	check_first: bool,
) -> Option<Vec<N>>
where
	N: Clone + Eq + Hash + 'a,
	IN: IntoIterator<Item = N>,
{
	if check_first {
		for start_node in start {
			if success(start_node) {
				return Some(vec![start_node.clone()]);
			}
		}
	}

	let mut parents = FxIndexMap::default();
	parents.extend(start.into_iter().map(|n| (n.clone(), usize::MAX)));

	let mut i = 0;
	while let Some((node, ..)) = parents.get_index(i) {
		for successor in successors(node) {
			if success(&successor) {
				let mut path = reverse_path(&parents, |&p| p, i);
				path.push(successor);
				return Some(path);
			}
			if let Vacant(e) = parents.entry(successor) {
				e.insert(i);
			}
		}

		i += 1;
	}

	None
}
