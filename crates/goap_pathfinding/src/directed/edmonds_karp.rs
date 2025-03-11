use std::{
	collections::{BTreeMap, BTreeSet, VecDeque},
	hash::Hash,
};

use super::bfs::bfs;
use crate::{
	FxIndexSet,
	matrix::Matrix,
	num_traits::{Bounded, Signed, Zero},
};

#[derive(Debug, Clone)]
pub struct Common<C> {
	size: usize,
	source: usize,
	sink: usize,
	total_capacity: C,
	details: bool,
}

pub struct SparseCapacity<C> {
	common: Common<C>,
	flows: BTreeMap<usize, BTreeMap<usize, C>>,
	residuals: BTreeMap<usize, BTreeMap<usize, C>>,
}

impl<C> SparseCapacity<C>
where
	C: Bounded + Copy + Eq + Ord + Signed + Zero,
{
	fn set_value(data: &mut BTreeMap<usize, BTreeMap<usize, C>>, from: usize, to: usize, value: C) {
		let to_remove = {
			let sub = data.entry(from).or_default();
			if value == C::zero() {
				sub.remove(&to);
			} else {
				sub.insert(to, value);
			}

			sub.is_empty()
		};

		if to_remove {
			data.remove(&from);
		}
	}

	fn get_value(data: &BTreeMap<usize, BTreeMap<usize, C>>, from: usize, to: usize) -> C {
		data.get(&from)
			.and_then(|ns| ns.get(&to))
			.copied()
			.unwrap_or_else(C::zero)
	}
}

impl<C> EdmondsKarp<C> for SparseCapacity<C> where C: Bounded + Copy + Eq + Ord + Signed + Zero {}

unsafe impl<C: Send> Send for SparseCapacity<C> {}

pub trait EdmondsKarp<C>
where
	C: Bounded + Copy + Ord + Signed + Zero,
{
	fn new(size: usize, source: usize, sink: usize) -> Self
	where
		Self: Sized;

	fn from_matrix(source: usize, sink: usize, capacities: Matrix<C>) -> Self
	where
		Self: Sized;

	#[must_use]
	fn from_iter(source: usize, sink: usize, capacities: impl IntoIterator<Item = C>) -> Self
	where
		Self: Sized,
	{
		Self::from_matrix(
			source,
			sink,
			Matrix::try_square_from_iter(capacities).unwrap(),
		)
	}

	fn common(&self) -> &Common<C>;

	fn common_mut(&mut self) -> &mut Common<C>;

	fn size(&self) -> usize {
		self.common().size
	}

	fn source(&self) -> usize {
		self.common().source
	}

	fn sink(&self) -> usize {
		self.common().sink
	}

	fn residual_successors(&self, from: usize) -> Vec<(usize, C)>;

	fn residual_capacity(&self, from: usize, to: usize) -> C;

	fn flow(&self, from: usize, to: usize) -> C;

	fn flows_from(&self, from: usize) -> Vec<usize>;

	fn flows(&self) -> Vec<((usize, usize), C)>;

	fn set_capacity(&mut self, from: usize, to: usize, capacity: C) {
		let flow = self.flow(from, to);
		let delta = capacity - (self.residual_capacity(from, to) + flow);
		if capacity < flow {
			let to_cancel = flow - capacity;
			self.add_flow(to, from, to_cancel);
			let source = self.source();
			self.cancel_flow(source, from, to_cancel);
			let sink = self.sink();
			self.cancel_flow(to, sink, to_cancel);
			self.common_mut().total_capacity = self.total_capacity() - to_cancel;
		}

		self.add_residual_capacity(from, to, delta);
	}

	fn add_flow(&mut self, from: usize, to: usize, capacity: C);

	fn total_capacity(&self) -> C {
		self.common().total_capacity
	}

	fn add_residual_capacity(&mut self, from: usize, to: usize, capacity: C);

	fn set_total_capacity(&mut self, capacity: C) {
		self.common_mut().total_capacity = capacity;
	}

	fn omit_details(&mut self) {
		self.common_mut().details = false;
	}

	fn has_details(&self) -> bool {
		self.common().details
	}

	fn augment(&mut self) -> EKFlows<usize, C> {
		let source_nodes = self.update_flows();
		if self.has_details() {
			let cuts = self
				.flows()
				.iter()
				.filter(|((from, to), _)| source_nodes.contains(from) && !source_nodes.contains(to))
				.copied()
				.collect::<Vec<_>>();
			(self.flows(), self.total_capacity(), cuts)
		} else {
			(Vec::new(), self.total_capacity(), Vec::new())
		}
	}
}

trait EdmondsKarpInternal<C> {
	fn update_flows(&mut self) -> BTreeSet<usize>;
	fn cancel_flow(&mut self, from: usize, to: usize, capacity: C);
}

impl<C, T> EdmondsKarpInternal<C> for T
where
	C: Bounded + Copy + Ord + Signed + Zero,
	T: EdmondsKarp<C> + ?Sized,
{
	fn update_flows(&mut self) -> BTreeSet<usize> {
		let size = self.size();
		let source = self.source();
		let sink = self.sink();
		let mut parents = vec![None; size];
		let mut path_capacity = vec![C::max_value(); size];
		let mut to_see = VecDeque::new();
		let mut seen = BTreeSet::new();
		'augment: loop {
			to_see.clear();
			to_see.push_back(source);
			seen.clear();
			while let Some(node) = to_see.pop_front() {
				seen.insert(node);
				let capacity_so_far = path_capacity[node];
				for (successor, residual) in self.residual_successors(node).iter().copied() {
					if successor == source || parents[successor].is_some() {
						continue;
					}

					parents[successor] = Some(node);
					path_capacity[successor] = if capacity_so_far < residual {
						capacity_so_far
					} else {
						residual
					};

					if successor == sink {
						let mut n = sink;
						while n != source {
							let p = parents[n].unwrap();
							self.add_flow(p, n, path_capacity[sink]);
							n = p;
						}

						let total = self.total_capacity();
						self.set_total_capacity(total + path_capacity[sink]);
						parents.fill(None);
						path_capacity.fill(C::max_value());
						continue 'augment;
					}

					to_see.push_back(successor);
				}
			}
			break;
		}

		seen
	}

	fn cancel_flow(&mut self, from: usize, to: usize, mut capacity: C) {
		if from == to {
			return;
		}

		while capacity > C::zero() {
			let Some(path) = bfs(&from, |&n| self.flows_from(n).into_iter(), |&n| n == to) else {
				unreachable!("no flow to cancel");
			};
			let path = path
				.clone()
				.into_iter()
				.zip(path.into_iter().skip(1))
				.collect::<Vec<_>>();
			let mut max_cancelable = path
				.iter()
				.map(|&(src, dst)| self.flow(src, dst))
				.min()
				.unwrap();

			if max_cancelable > capacity {
				max_cancelable = capacity;
			}

			for (src, dst) in path {
				self.add_flow(dst, src, max_cancelable);
			}

			capacity = capacity - max_cancelable;
		}
	}
}

pub type Edge<N, C> = ((N, N), C);

pub type EKFlows<N, C> = (Vec<Edge<N, C>>, C, Vec<Edge<N, C>>);
