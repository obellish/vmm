use std::{
	cmp::Reverse,
	collections::{BinaryHeap, HashSet},
	hash::Hash,
};

pub fn prim<N, C>(edges: &[(N, N, C)]) -> Vec<(&N, &N, C)>
where
	N: Eq + Hash + Ord,
	C: Clone + Ord,
{
	let Some((start, ..)) = edges.first() else {
		return Vec::new();
	};

	let mut priority_queue = edges
		.iter()
		.filter_map(|(n, n1, c)| (n == start).then_some(Reverse((c, n, n1))))
		.collect::<BinaryHeap<_>>();

	let (mut mst, mut visited) = (Vec::new(), HashSet::new());
	visited.insert(start);
	while let Some(Reverse((c, n, n1))) = priority_queue.pop() {
		if visited.contains(n1) {
			continue;
		}

		mst.push((n, n1, c.clone()));

		for (n2, n3, c) in edges {
			if n1 == n2 && !visited.contains(n3) {
				priority_queue.push(Reverse((c, n1, n3)));
			} else if n1 == n3 && !visited.contains(n2) {
				priority_queue.push(Reverse((c, n1, n2)));
			}
		}

		visited.insert(n1);
	}

	mst
}
