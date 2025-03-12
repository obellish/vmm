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

impl<C> EdmondsKarp<C> for SparseCapacity<C>
where
	C: Bounded + Copy + Eq + Ord + Signed + Zero,
{
	fn new(size: usize, source: usize, sink: usize) -> Self
	where
		Self: Sized,
	{
		assert!(source < size, "source is greater than or equal to size");
		assert!(sink < size, "sink is greater than or equal to size");

		Self {
			common: Common {
				sink,
				size,
				source,
				details: true,
				total_capacity: C::zero(),
			},
			flows: BTreeMap::new(),
			residuals: BTreeMap::new(),
		}
	}

	fn from_matrix(source: usize, sink: usize, capacities: Matrix<C>) -> Self
	where
		Self: Sized,
	{
		assert!(capacities.is_square(), "capacities matrix is not square");
		let size = capacities.rows;
		assert!(
			source < size,
			"source is greater than or equal to matrix size"
		);
		assert!(sink < size, "sink is greater than or equal to matrix size");
		let mut result = Self::new(size, source, sink);
		for from in 0..size {
			for to in 0..size {
				let capacity = capacities[(from, to)];
				if capacity > C::zero() {
					result.set_capacity(from, to, capacity);
				}
			}
		}

		result
	}

	fn common(&self) -> &Common<C> {
		&self.common
	}

	fn common_mut(&mut self) -> &mut Common<C> {
		&mut self.common
	}

	fn residual_successors(&self, from: usize) -> Vec<(usize, C)> {
		self.residuals.get(&from).map_or_else(Vec::new, |ns| {
			ns.iter()
				.filter_map(|(&n, &c)| (c > C::zero()).then_some((n, c)))
				.collect()
		})
	}

	fn residual_capacity(&self, from: usize, to: usize) -> C {
		Self::get_value(&self.residuals, from, to)
	}

	fn flow(&self, from: usize, to: usize) -> C {
		Self::get_value(&self.flows, from, to)
	}

	fn flows(&self) -> Vec<((usize, usize), C)> {
		self.flows
			.clone()
			.into_iter()
			.flat_map(|(k, vs)| {
				vs.into_iter()
					.filter_map(move |(v, c)| (c > C::zero()).then_some(((k, v), c)))
			})
			.collect()
	}

	fn add_flow(&mut self, from: usize, to: usize, capacity: C) {
		let direct = self.flow(from, to) + capacity;
		Self::set_value(&mut self.flows, from, to, direct);
		Self::set_value(&mut self.flows, to, from, -direct);
		self.add_residual_capacity(from, to, -capacity);
		self.add_residual_capacity(to, from, capacity);
	}

	fn add_residual_capacity(&mut self, from: usize, to: usize, capacity: C) {
		let new_capacity = self.residual_capacity(from, to) + capacity;
		Self::set_value(&mut self.residuals, from, to, new_capacity);
	}

	fn flows_from(&self, from: usize) -> Vec<usize> {
		self.flows.get(&from).map_or_else(Vec::new, |ns| {
			ns.iter()
				.filter_map(|(&o, &c)| (c > C::zero()).then_some(o))
				.collect()
		})
	}
}

unsafe impl<C: Send> Send for SparseCapacity<C> {}

#[derive(Debug, Clone)]
pub struct DenseCapacity<C> {
	common: Common<C>,
	residuals: Matrix<C>,
	flows: Matrix<C>,
}

impl<C> EdmondsKarp<C> for DenseCapacity<C>
where
	C: Bounded + Copy + Ord + Signed + Zero,
{
	fn new(size: usize, source: usize, sink: usize) -> Self
	where
		Self: Sized,
	{
		assert!(source < size, "source is greater than or equal to size");
		assert!(sink < size, "sink is greater than or equal to size");
		Self {
			common: Common {
				size,
				sink,
				source,
				total_capacity: C::zero(),
				details: true,
			},
			residuals: Matrix::new(size, size, C::zero()),
			flows: Matrix::new(size, size, C::zero()),
		}
	}

	fn from_matrix(source: usize, sink: usize, capacities: Matrix<C>) -> Self
	where
		Self: Sized,
	{
		assert!(capacities.is_square(), "capacities matrix is not square");
		let size = capacities.rows;
		assert!(
			source < size,
			"source is greater than or equal to matrix size"
		);
		assert!(sink < size, "sink is greater than or equal to matrix size");
		Self {
			common: Common {
				size,
				sink,
				source,
				total_capacity: C::zero(),
				details: true,
			},
			residuals: capacities,
			flows: Matrix::new(size, size, C::zero()),
		}
	}

	fn common(&self) -> &Common<C> {
		&self.common
	}

	fn common_mut(&mut self) -> &mut Common<C> {
		&mut self.common
	}

	fn residual_successors(&self, from: usize) -> Vec<(usize, C)> {
		(0..self.size())
			.filter_map(|n| {
				let residual = self.residual_capacity(from, n);
				(residual > C::zero()).then_some((n, residual))
			})
			.collect()
	}

	fn residual_capacity(&self, from: usize, to: usize) -> C {
		self.residuals[(from, to)]
	}

	fn flow(&self, from: usize, to: usize) -> C {
		self.flows[(from, to)]
	}

	fn flows(&self) -> Vec<((usize, usize), C)> {
		(0..self.size())
			.flat_map(|from| (0..self.size()).map(move |to| (from, to)))
			.filter_map(|(from, to)| {
				let flow = self.flow(from, to);
				(flow > C::zero()).then_some(((from, to), flow))
			})
			.collect()
	}

	fn add_flow(&mut self, from: usize, to: usize, capacity: C) {
		self.flows[(from, to)] = self.flows[(from, to)] + capacity;
		self.flows[(to, from)] = self.flows[(to, from)] - capacity;
		self.residuals[(from, to)] = self.residuals[(from, to)] - capacity;
		self.residuals[(to, from)] = self.residuals[(to, from)] + capacity;
	}

	fn add_residual_capacity(&mut self, from: usize, to: usize, capacity: C) {
		self.residuals[(from, to)] = self.residual_capacity(from, to) + capacity;
	}

	fn flows_from(&self, from: usize) -> Vec<usize> {
		(0..self.size())
			.filter(|to| self.flow(from, *to) > C::zero())
			.collect()
	}
}

unsafe impl<C: Send> Send for DenseCapacity<C> {}

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

pub fn edmonds_karp<N, C, IC, EK>(vertices: &[N], source: &N, sink: &N, caps: IC) -> EKFlows<N, C>
where
	N: Copy + Eq + Hash,
	C: Bounded + Copy + Ord + Signed + Zero,
	IC: IntoIterator<Item = Edge<N, C>>,
	EK: EdmondsKarp<C>,
{
	let reverse = vertices.iter().collect::<FxIndexSet<_>>();
	let mut capacities = EK::new(
		vertices.len(),
		reverse
			.get_index_of(source)
			.unwrap_or_else(|| panic!("source not found in vertices")),
		reverse
			.get_index_of(sink)
			.unwrap_or_else(|| panic!("sink not found in vertices")),
	);
	for ((from, to), capacity) in caps {
		capacities.set_capacity(
			reverse.get_index_of(&from).unwrap(),
			reverse.get_index_of(&to).unwrap(),
			capacity,
		);
	}

	let (paths, max, cut) = capacities.augment();
	(
		paths
			.into_iter()
			.map(|((a, b), c)| ((vertices[a], vertices[b]), c))
			.collect(),
		max,
		cut.into_iter()
			.map(|((a, b), c)| ((vertices[a], vertices[b]), c))
			.collect(),
	)
}

pub fn edmonds_karp_dense<N, C, IC>(vertices: &[N], source: &N, sink: &N, caps: IC) -> EKFlows<N, C>
where
	N: Copy + Eq + Hash,
	C: Bounded + Copy + Ord + Signed + Zero,
	IC: IntoIterator<Item = Edge<N, C>>,
{
	edmonds_karp::<N, C, IC, DenseCapacity<C>>(vertices, source, sink, caps)
}

pub fn edmonds_karp_sparse<N, C, IC>(
	vertices: &[N],
	source: &N,
	sink: &N,
	caps: IC,
) -> EKFlows<N, C>
where
	N: Copy + Eq + Hash,
	C: Bounded + Copy + Ord + Signed + Zero,
	IC: IntoIterator<Item = Edge<N, C>>,
{
	edmonds_karp::<N, C, IC, SparseCapacity<C>>(vertices, source, sink, caps)
}
