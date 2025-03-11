use std::{
	cmp::Ordering,
	collections::{BinaryHeap, HashMap},
	hash::{BuildHasher, Hash},
};

use indexmap::map::Entry::{Occupied, Vacant};
use rustc_hash::{FxHashMap, FxHashSet};

use super::reverse_path;
use crate::{FxIndexMap, num_traits::Zero};

pub struct DijkstraReachable<N, C, FN> {
	to_see: BinaryHeap<SmallestHolder<C>>,
	seen: FxHashSet<usize>,
	parents: FxIndexMap<N, (usize, C)>,
	total_costs: FxHashMap<N, C>,
	successors: FN,
}

impl<N, C, FN, IN> Iterator for DijkstraReachable<N, C, FN>
where
	N: Clone + Eq + Hash,
	C: Copy + Hash + Ord + Zero,
	FN: FnMut(&N) -> IN,
	IN: IntoIterator<Item = (N, C)>,
{
	type Item = DijkstraReachableItem<N, C>;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(SmallestHolder { cost, index }) = self.to_see.pop() {
			if !self.seen.insert(index) {
				continue;
			}

			let item;
			let successors = {
				let (node, (parent_index, _)) = self.parents.get_index(index)?;
				let total_cost = self.total_costs[node];
				item = Some(DijkstraReachableItem {
					node: node.clone(),
					parent: self.parents.get_index(*parent_index).map(|x| x.0.clone()),
					total_cost,
				});
				(self.successors)(node)
			};
			for (successor, move_cost) in successors {
				let new_cost = cost + move_cost;
				let n;
				match self.parents.entry(successor.clone()) {
					Vacant(e) => {
						n = e.index();
						e.insert((index, new_cost));
						self.total_costs.insert(successor.clone(), new_cost);
					}
					Occupied(mut e) if e.get().1 > new_cost => {
						n = e.index();
						e.insert((index, new_cost));
						self.total_costs.insert(successor.clone(), new_cost);
					}
					Occupied(_) => continue,
				}

				self.to_see.push(SmallestHolder {
					cost: new_cost,
					index: n,
				});
			}

			return item;
		}

		None
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DijkstraReachableItem<N, C> {
	pub node: N,
	pub parent: Option<N>,
	pub total_cost: C,
}

struct SmallestHolder<K> {
	cost: K,
	index: usize,
}

impl<K: PartialEq> Eq for SmallestHolder<K> {}

impl<K: Ord> Ord for SmallestHolder<K> {
	fn cmp(&self, other: &Self) -> Ordering {
		other.cost.cmp(&self.cost)
	}
}

impl<K: PartialEq> PartialEq for SmallestHolder<K> {
	fn eq(&self, other: &Self) -> bool {
		self.cost == other.cost
	}
}

impl<K: Ord> PartialOrd for SmallestHolder<K> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

pub fn dijkstra_reach<N, C, FN, IN>(start: &N, successors: FN) -> DijkstraReachable<N, C, FN>
where
	N: Clone + Eq + Hash,
	C: Copy + Ord + Zero,
	FN: FnMut(&N) -> IN,
	IN: IntoIterator<Item = (N, C)>,
{
	let mut to_see = BinaryHeap::new();
	to_see.push(SmallestHolder {
		cost: Zero::zero(),
		index: 0,
	});

	let mut parents = FxIndexMap::default();
	parents.insert(start.clone(), (usize::MAX, Zero::zero()));

	let mut total_costs = FxHashMap::default();
	total_costs.insert(start.clone(), Zero::zero());

	let seen = FxHashSet::default();

	DijkstraReachable {
		to_see,
		seen,
		parents,
		total_costs,
		successors,
	}
}

pub fn build_path<N, C, S: BuildHasher>(target: &N, parents: &HashMap<N, (N, C), S>) -> Vec<N>
where
	N: Clone + Eq + Hash,
{
	let mut rev = vec![target.clone()];
	let mut next = target.clone();
	while let Some((parent, _)) = parents.get(&next) {
		rev.push(parent.clone());
		next = parent.clone();
	}
	rev.reverse();
	rev
}

pub fn dijkstra<N, C, IN>(
	start: &N,
	mut successors: impl FnMut(&N) -> IN,
	mut success: impl FnMut(&N) -> bool,
) -> Option<(Vec<N>, C)>
where
	N: Clone + Eq + Hash,
	C: Copy + Ord + Zero,
	IN: IntoIterator<Item = (N, C)>,
{
	dijkstra_internal(start, &mut successors, &mut success)
}

pub fn dijkstra_all<N, C, IN>(start: &N, successors: impl FnMut(&N) -> IN) -> HashMap<N, (N, C)>
where
	N: Clone + Eq + Hash,
	C: Copy + Ord + Zero,
	IN: IntoIterator<Item = (N, C)>,
{
	dijkstra_partial(start, successors, |_| false).0
}

pub fn dijkstra_partial<N, C, IN>(
	start: &N,
	mut successors: impl FnMut(&N) -> IN,
	mut stop: impl FnMut(&N) -> bool,
) -> (HashMap<N, (N, C)>, Option<N>)
where
	N: Clone + Eq + Hash,
	C: Copy + Ord + Zero,
	IN: IntoIterator<Item = (N, C)>,
{
	let (parents, reached) = run_dijkstra(start, &mut successors, &mut stop);
	(
		parents
			.iter()
			.skip(1)
			.map(|(n, (p, c))| (n.clone(), (parents.get_index(*p).unwrap().0.clone(), *c)))
			.collect(),
		reached
			.and_then(|i| parents.get_index(i))
			.map(|n| n.0.clone()),
	)
}

pub(crate) fn dijkstra_internal<N, C, IN>(
	start: &N,
	successors: &mut impl FnMut(&N) -> IN,
	success: &mut impl FnMut(&N) -> bool,
) -> Option<(Vec<N>, C)>
where
	N: Clone + Eq + Hash,
	C: Copy + Ord + Zero,
	IN: IntoIterator<Item = (N, C)>,
{
	let (parents, reached) = run_dijkstra(start, successors, success);
	reached.and_then(|target| {
		Some((
			reverse_path(&parents, |&(p, _)| p, target),
			parents.get_index(target)?.1.1,
		))
	})
}

fn run_dijkstra<N, C, IN>(
	start: &N,
	successors: &mut impl FnMut(&N) -> IN,
	stop: &mut impl FnMut(&N) -> bool,
) -> (FxIndexMap<N, (usize, C)>, Option<usize>)
where
	N: Clone + Eq + Hash,
	C: Copy + Ord + Zero,
	IN: IntoIterator<Item = (N, C)>,
{
	let mut to_see = BinaryHeap::new();
	to_see.push(SmallestHolder {
		cost: C::zero(),
		index: 0,
	});
	let mut parents = FxIndexMap::default();
	parents.insert(start.clone(), (usize::MAX, C::zero()));
	let mut target_reached = None;
	while let Some(SmallestHolder { cost, index }) = to_see.pop() {
		let successors = {
			let (node, &(_, c)) = parents.get_index(index).unwrap();
			if stop(node) {
				target_reached = Some(index);
				break;
			}

			if cost > c {
				continue;
			}
			successors(node)
		};

		for (successor, move_cost) in successors {
			let new_cost = cost + move_cost;
			let n;
			match parents.entry(successor) {
				Vacant(e) => {
					n = e.index();
					e.insert((index, new_cost));
				}
				Occupied(mut e) if e.get().1 > new_cost => {
					n = e.index();
					e.insert((index, new_cost));
				}
				Occupied(_) => continue,
			}

			to_see.push(SmallestHolder {
				cost: new_cost,
				index: n,
			});
		}
	}

	(parents, target_reached)
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;

	use itertools::Itertools;

	use super::{DijkstraReachableItem, dijkstra_reach};

	#[test]
	fn numbers() {
		let reach = dijkstra_reach(&0, |prev| [(prev + 1, 1), (prev * 2, *prev)])
			.take_while(|x| x.total_cost < 100)
			.collect_vec();

		assert!(reach.iter().all(|x| x.node == x.total_cost));
		assert!((0..100).all(|x| reach.iter().any(|y| x == y.total_cost)));

		assert!(
			reach
				.iter()
				.map(|x| x.total_cost)
				.tuple_windows()
				.all(|(a, b)| b >= a)
		);
	}

	#[test]
	fn graph() {
		let mut graph = HashMap::new();
		graph.insert("A", vec![("B", 2), ("C", 5)]);
		graph.insert("B", vec![("C", 2)]);
		graph.insert("C", vec![]);

		let reach = dijkstra_reach(&"A", |prev| graph[prev].clone()).collect_vec();

		assert_eq!(
			reach,
			[
				DijkstraReachableItem {
					node: "A",
					parent: None,
					total_cost: 0,
				},
				DijkstraReachableItem {
					node: "B",
					parent: Some("A"),
					total_cost: 2,
				},
				DijkstraReachableItem {
					node: "C",
					parent: Some("B"),
					total_cost: 4
				}
			]
		);
	}
}
