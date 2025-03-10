use std::{
	cmp::Ordering,
	collections::{BinaryHeap, HashSet},
	hash::Hash,
	iter::FusedIterator,
};

use indexmap::map::Entry::{Occupied, Vacant};

use super::reverse_path;
use crate::{FxIndexMap, num_traits::Zero};

#[derive(Clone)]
pub struct AstarSolution<N> {
	sinks: Vec<usize>,
	parents: Vec<(N, Vec<usize>)>,
	current: Vec<Vec<usize>>,
	terminated: bool,
}

impl<N> AstarSolution<N>
where
	N: Clone + Eq + Hash,
{
	fn complete(&mut self) {
		loop {
			let ps = match self.current.last() {
				None => self.sinks.clone(),
				Some(last) => self.parents(*last.last().unwrap()).clone(),
			};

			if ps.is_empty() {
				break;
			}

			self.current.push(ps);
		}
	}

	fn next_vec(&mut self) {
		while matches!(self.current.last().map(Vec::len), Some(1)) {
			self.current.pop();
		}

		self.current.last_mut().map(Vec::pop);
	}

	fn node(&self, i: usize) -> &N {
		&self.parents[i].0
	}

	fn parents(&self, i: usize) -> &Vec<usize> {
		&self.parents[i].1
	}
}

impl<N> Iterator for AstarSolution<N>
where
	N: Clone + Eq + Hash,
{
	type Item = Vec<N>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.terminated {
			return None;
		}

		let path = self
			.current
			.iter()
			.rev()
			.filter_map(|v| v.last().copied())
			.map(|i| self.node(i).clone())
			.collect::<Vec<_>>();
		self.next_vec();
		self.terminated = self.current.is_empty();
		Some(path)
	}
}

impl<N> FusedIterator for AstarSolution<N> where N: Clone + Eq + Hash {}

struct SmallestCostHolder<K> {
	estimated_cost: K,
	cost: K,
	index: usize,
}

impl<K: PartialEq> Eq for SmallestCostHolder<K> {}

impl<K: Ord> Ord for SmallestCostHolder<K> {
	fn cmp(&self, other: &Self) -> Ordering {
		other
			.estimated_cost
			.cmp(&self.estimated_cost)
			.then(self.cost.cmp(&other.cost))
	}
}

impl<K: PartialEq> PartialEq for SmallestCostHolder<K> {
	fn eq(&self, other: &Self) -> bool {
		self.estimated_cost.eq(&other.estimated_cost) && self.cost.eq(&other.cost)
	}
}

impl<K: Ord> PartialOrd for SmallestCostHolder<K> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

pub fn astar<N, C, IN>(
	start: &N,
	mut successors: impl FnMut(&N) -> IN,
	mut heuristic: impl FnMut(&N) -> C,
	mut success: impl FnMut(&N) -> bool,
) -> Option<(Vec<N>, C)>
where
	N: Clone + Eq + Hash,
	C: Copy + Ord + Zero,
	IN: IntoIterator<Item = (N, C)>,
{
	let mut to_see = BinaryHeap::new();
	to_see.push(SmallestCostHolder {
		estimated_cost: C::zero(),
		cost: C::zero(),
		index: 0,
	});

	let mut parents = FxIndexMap::default();
	parents.insert(start.clone(), (usize::MAX, C::zero()));
	while let Some(SmallestCostHolder { cost, index, .. }) = to_see.pop() {
		let successors = {
			let (node, &(_, c)) = parents.get_index(index)?;
			if success(node) {
				let path = reverse_path(&parents, |&(p, _)| p, index);
				return Some((path, cost));
			}

			if cost > c {
				continue;
			}

			successors(node)
		};

		for (successor, move_cost) in successors {
			let new_cost = cost + move_cost;
			let h;
			let n;
			match parents.entry(successor) {
				Vacant(e) => {
					h = heuristic(e.key());
					n = e.index();
					e.insert((index, new_cost));
				}
				Occupied(mut e) if e.get().1 > new_cost => {
					h = heuristic(e.key());
					n = e.index();
					e.insert((index, new_cost));
				}
				Occupied(..) => continue,
			}

			to_see.push(SmallestCostHolder {
				estimated_cost: new_cost + h,
				cost: new_cost,
				index: n,
			});
		}
	}

	None
}

pub fn astar_bag<N, C, IN>(
	start: &N,
	mut successors: impl FnMut(&N) -> IN,
	mut heuristic: impl FnMut(&N) -> C,
	mut success: impl FnMut(&N) -> bool,
) -> Option<(AstarSolution<N>, C)>
where
	N: Clone + Eq + Hash,
	C: Copy + Ord + Zero,
	IN: IntoIterator<Item = (N, C)>,
{
	let mut to_see = BinaryHeap::new();
	let mut min_cost = None;
	let mut sinks = HashSet::new();
	to_see.push(SmallestCostHolder {
		estimated_cost: C::zero(),
		cost: C::zero(),
		index: 0,
	});
	let mut parents = FxIndexMap::default();
	parents.insert(start.clone(), (HashSet::<usize>::new(), C::zero()));
	while let Some(SmallestCostHolder {
		estimated_cost,
		cost,
		index,
	}) = to_see.pop()
	{
		if matches!(min_cost, Some(min_cost) if estimated_cost > min_cost) {
			break;
		}

		let successors = {
			let (node, &(_, c)) = parents.get_index(index)?;
			if success(node) {
				min_cost = Some(cost);
				sinks.insert(index);
			}

			if cost > c {
				continue;
			}
			successors(node)
		};

		for (successor, move_cost) in successors {
			let new_cost = cost + move_cost;
			let h;
			let n;
			match parents.entry(successor) {
				Vacant(e) => {
					h = heuristic(e.key());
					n = e.index();
					let mut p = HashSet::new();
					p.insert(index);
					e.insert((p, new_cost));
				}
				Occupied(mut e) => {
					if e.get().1 > new_cost {
						h = heuristic(e.key());
						n = e.index();
						let s = e.get_mut();
						s.0.clear();
						s.0.insert(index);
						s.1 = new_cost;
					} else {
						if e.get().1 == new_cost {
							e.get_mut().0.insert(index);
						}
						continue;
					}
				}
			}

			to_see.push(SmallestCostHolder {
				estimated_cost: new_cost + h,
				cost: new_cost,
				index: n,
			});
		}
	}

	min_cost.map(|cost| {
		let parents = parents
			.into_iter()
			.map(|(k, (ps, ..))| (k, ps.into_iter().collect()))
			.collect();
		(
			AstarSolution {
				sinks: sinks.into_iter().collect(),
				parents,
				current: Vec::new(),
				terminated: false,
			},
			cost,
		)
	})
}

pub fn astar_bag_collect<N, C, IN>(
	start: &N,
	successors: impl FnMut(&N) -> IN,
	heuristic: impl FnMut(&N) -> C,
	success: impl FnMut(&N) -> bool,
) -> Option<(Vec<Vec<N>>, C)>
where
	N: Clone + Eq + Hash,
	C: Copy + Ord + Zero,
	IN: IntoIterator<Item = (N, C)>,
{
	astar_bag(start, successors, heuristic, success)
		.map(|(solutions, cost)| (solutions.collect(), cost))
}
