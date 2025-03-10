pub mod astar;
pub mod bfs;

use std::hash::Hash;

use super::FxIndexMap;

#[expect(clippy::needless_collect)]
fn reverse_path<N, V>(
	parents: &FxIndexMap<N, V>,
	mut parent: impl FnMut(&V) -> usize,
	start: usize,
) -> Vec<N>
where
	N: Clone + Eq + Hash,
{
	let mut i = start;
	let path = std::iter::from_fn(|| {
		parents.get_index(i).map(|(node, value)| {
			i = parent(value);
			node
		})
	})
	.collect::<Vec<_>>();

	path.into_iter().rev().cloned().collect()
}
