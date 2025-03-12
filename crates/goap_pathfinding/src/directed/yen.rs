use std::{
	cmp::{Ordering, Reverse},
	collections::{BinaryHeap, HashSet},
	hash::Hash,
};

use super::dijkstra::dijkstra_internal;
use crate::num_traits::Zero;

#[derive(Debug, PartialEq, Eq)]
struct Path<N, C>
where
	N: Clone + Eq + Hash,
	C: Copy + Ord + Zero,
{
	nodes: Vec<N>,
	cost: C,
}

impl<N, C> Ord for Path<N, C>
where
	N: Clone + Eq + Hash,
	C: Copy + Ord + Zero,
{
	fn cmp(&self, other: &Self) -> Ordering {
		self.cost
			.cmp(&other.cost)
			.then(self.nodes.len().cmp(&other.nodes.len()))
	}
}

impl<N, C> PartialOrd for Path<N, C>
where
	N: Clone + Eq + Hash,
	C: Copy + Ord + Zero,
{
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

pub fn yen<N, C, IN>(
	start: &N,
	mut successors: impl FnMut(&N) -> IN,
	mut success: impl FnMut(&N) -> bool,
	k: usize,
) -> Vec<(Vec<N>, C)>
where
	N: Clone + Eq + Hash,
	C: Copy + Ord + Zero,
	IN: IntoIterator<Item = (N, C)>,
{
	let Some((n, c)) = dijkstra_internal(start, &mut successors, &mut success) else {
		return Vec::new();
	};

	let mut visited = HashSet::new();
	let mut routes = vec![Path { nodes: n, cost: c }];
	let mut k_routes = BinaryHeap::new();
	for ki in 0..(k - 1) {
		if routes.len() <= ki || routes.len() == k {
			break;
		}

		let previous = &routes[ki].nodes;
		for i in 0..(previous.len() - 1) {
			let spur_node = &previous[i];
			let root_path = &previous[0..i];

			let mut filtered_edges = HashSet::new();
			for path in &routes {
				if path.nodes.len() > i + 1
					&& &path.nodes[0..i] == root_path
					&& &path.nodes[i] == spur_node
				{
					filtered_edges.insert((&path.nodes[i], &path.nodes[i + 1]));
				}
			}
			let filtered_nodes = HashSet::<&N>::from_iter(root_path);
			let mut filtered_successor = |n: &N| {
				successors(n)
					.into_iter()
					.filter(|(n2, ..)| {
						!filtered_nodes.contains(&n2) && !filtered_edges.contains(&(n, n2))
					})
					.collect::<Vec<_>>()
			};

			if let Some((spur_path, ..)) =
				dijkstra_internal(spur_node, &mut filtered_successor, &mut success)
			{
				let nodes = root_path
					.iter()
					.cloned()
					.chain(spur_path)
					.collect::<Vec<_>>();
				if !visited.contains(&nodes) {
					let cost = make_cost(&nodes, &mut successors);
					let path = Path { nodes, cost };
					visited.insert(path.nodes.clone());
					k_routes.push(Reverse(path));
				}
			}
		}

		if let Some(k_route) = k_routes.pop() {
			let route = k_route.0;
			let cost = route.cost;
			routes.push(route);

			while routes.len() < k {
				let Some(k_route) = k_routes.peek() else {
					break;
				};
				if k_route.0.cost == cost {
					let Some(k_route) = k_routes.pop() else {
						break;
					};
					routes.push(k_route.0);
				} else {
					break;
				}
			}
		}
	}

	routes.sort_unstable();
	routes
		.into_iter()
		.map(|Path { nodes, cost }| (nodes, cost))
		.collect()
}

fn make_cost<N, IN, C>(nodes: &[N], successors: &mut impl FnMut(&N) -> IN) -> C
where
	N: Eq,
	C: Zero,
	IN: IntoIterator<Item = (N, C)>,
{
	let mut cost = C::zero();
	for edge in nodes.windows(2) {
		for (n, c) in successors(&edge[0]) {
			if n == edge[1] {
				cost = cost + c;
			}
		}
	}

	cost
}
