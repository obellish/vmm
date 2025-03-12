use std::{hash::Hash, ops::ControlFlow};

use indexmap::IndexSet;

use crate::num_traits::Zero;

pub fn idastar<N, C, IN>(
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
	let mut path = IndexSet::from([start.clone()]);

	std::iter::repeat(())
		.try_fold(heuristic(start), |bound, ()| {
			search(
				&mut path,
				C::zero(),
				bound,
				&mut successors,
				&mut heuristic,
				&mut success,
			)
			.map_break(Some)?
			.map_or(ControlFlow::Break(None), ControlFlow::Continue)
		})
		.break_value()
		.flatten()
}

fn search<N, C, IN>(
	path: &mut IndexSet<N>,
	cost: C,
	bound: C,
	successors: &mut impl FnMut(&N) -> IN,
	heuristic: &mut impl FnMut(&N) -> C,
	success: &mut impl FnMut(&N) -> bool,
) -> ControlFlow<(Vec<N>, C), Option<C>>
where
	N: Clone + Eq + Hash,
	C: Copy + Ord + Zero,
	IN: IntoIterator<Item = (N, C)>,
{
	let neighbors = {
		let start = &path[path.len() - 1];
		let f = cost + heuristic(start);
		if f > bound {
			return ControlFlow::Continue(Some(f));
		}
		if success(start) {
			return ControlFlow::Break((path.iter().cloned().collect(), f));
		}

		let mut neighbors = successors(start)
			.into_iter()
			.filter_map(|(n, c)| {
				(!path.contains(&n)).then(|| {
					let h = heuristic(&n);
					(n, c, c + h)
				})
			})
			.collect::<Vec<_>>();
		neighbors.sort_unstable_by(|(.., c1), (.., c2)| c1.cmp(c2));
		neighbors
	};
	let mut min = None;
	for (node, extra, ..) in neighbors {
		let (idx, _) = path.insert_full(node);
		match search(path, cost + extra, bound, successors, heuristic, success)? {
			Some(m) if min.is_none_or(|n| n >= m) => min = Some(m),
			_ => {}
		}
		path.swap_remove_index(idx);
	}
	ControlFlow::Continue(min)
}
