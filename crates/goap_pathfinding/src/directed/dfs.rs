use std::{collections::HashSet, hash::Hash, iter::FusedIterator};

use rustc_hash::{FxHashMap, FxHashSet};

pub struct DfsReachable<N, FN> {
	to_see: Vec<N>,
	visited: HashSet<N>,
	successors: FN,
}

impl<N, FN> DfsReachable<N, FN>
where
	N: Eq + Hash,
{
	pub fn remaining_nodes_low_bound(&self) -> usize {
		self.to_see.iter().collect::<HashSet<_>>().len()
	}
}

impl<N, FN, IN> Iterator for DfsReachable<N, FN>
where
	N: Clone + Eq + Hash,
	FN: FnMut(&N) -> IN,
	IN: IntoIterator<Item = N>,
{
	type Item = N;

	fn next(&mut self) -> Option<Self::Item> {
		let n = self.to_see.pop()?;
		if self.visited.contains(&n) {
			return self.next();
		}

		self.visited.insert(n.clone());
		let mut to_insert = Vec::new();
		for s in (self.successors)(&n) {
			if !self.visited.contains(&s) {
				to_insert.push(s.clone());
			}
		}

		self.to_see.extend(to_insert.into_iter().rev());
		Some(n)
	}
}

impl<N, FN, IN> FusedIterator for DfsReachable<N, FN>
where
	N: Clone + Eq + Hash,
	FN: FnMut(&N) -> IN,
	IN: IntoIterator<Item = N>,
{
}

pub fn dfs<N, IN>(
	start: N,
	mut successors: impl FnMut(&N) -> IN,
	mut success: impl FnMut(&N) -> bool,
) -> Option<Vec<N>>
where
	N: Clone + Eq + Hash,
	IN: IntoIterator<Item = N>,
{
	let mut to_visit = vec![start];
	let mut visited = FxHashSet::default();
	let mut parents = FxHashMap::default();
	while let Some(node) = to_visit.pop() {
		if visited.insert(node.clone()) {
			if success(&node) {
				return Some(build_path(node, &parents));
			}

			for next in successors(&node)
				.into_iter()
				.collect::<Vec<_>>()
				.into_iter()
				.rev()
			{
				if !visited.contains(&next) {
					parents.insert(next.clone(), node.clone());
					to_visit.push(next);
				}
			}
		}
	}

	None
}

pub fn dfs_reach<N, FN, IN>(start: N, successors: FN) -> DfsReachable<N, FN>
where
	N: Clone + Eq + Hash,
	FN: FnMut(&N) -> IN,
	IN: IntoIterator<Item = N>,
{
	DfsReachable {
		to_see: vec![start],
		visited: HashSet::new(),
		successors,
	}
}

fn build_path<N>(mut node: N, parents: &FxHashMap<N, N>) -> Vec<N>
where
	N: Clone + Eq + Hash,
{
	let mut path = vec![node.clone()];
	while let Some(parent) = parents.get(&node).cloned() {
		path.push(parent.clone());
		node = parent;
	}

	path.into_iter().rev().collect()
}

#[cfg(test)]
mod tests {
	use super::dfs_reach;

	#[test]
	fn edge_case_511() {
		let it = dfs_reach(0, |&n| [n + 1, n + 5].into_iter().filter(|&x| x <= 10));
		assert_eq!(it.collect::<Vec<_>>(), [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
	}

	#[test]
	fn edge_case_511_branches() {
		let it = dfs_reach(0, |&n| [n + 2, n + 5].into_iter().filter(|&x| x <= 10));
		assert_eq!(it.collect::<Vec<_>>(), [0, 2, 4, 6, 8, 10, 9, 7, 5]);
	}
}
