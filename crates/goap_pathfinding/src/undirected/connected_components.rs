use std::{
	collections::{
		HashMap, HashSet,
		hash_map::Entry::{Occupied, Vacant},
	},
	hash::Hash,
	iter::once,
	marker::PhantomData,
};

use rustc_hash::{FxHashMap, FxHashSet};

pub struct ConnectedComponents<
	N,
	It = Vec<N>,
	It2 = HashSet<N>,
	C1 = HashSet<N>,
	C2 = Vec<C1>,
	C3 = HashMap<N, usize>,
> {
	marker: PhantomData<(N, It, It2, C1, C2, C3)>,
}

impl<N, It, It2, C1, C2, C3> ConnectedComponents<N, It, It2, C1, C2, C3>
where
	N: Clone + Eq + Hash,
	It: Clone + IntoIterator<Item = N>,
	for<'it> &'it It: IntoIterator<Item = &'it N>,
	for<'it> &'it It2: IntoIterator<Item = &'it N>,
	C1: FromIterator<N>,
	C2: FromIterator<C1>,
	C3: FromIterator<(N, usize)>,
{
	pub fn separate_components(groups: &[It]) -> (HashMap<&N, usize>, Vec<usize>) {
		let mut table = (0..groups.len()).collect::<Vec<_>>();
		let mut indices = HashMap::new();
		for (mut group_index, group) in groups.iter().enumerate() {
			let mut is_empty = true;
			for element in group {
				is_empty = false;
				match indices.entry(element) {
					Vacant(e) => {
						e.insert(group_index);
					}
					Occupied(e) => {
						table[group_index] = find(&mut table, *e.get());
						group_index = table[group_index];
					}
				}
			}

			if is_empty {
				table[group_index] = usize::MAX;
			}
		}

		#[expect(unused_mut)]
		for mut group_index in indices.values_mut() {
			*group_index = find(&mut table, *group_index);
		}
		for group_index in 0..groups.len() {
			if table[group_index] != usize::MAX {
				let target = find(&mut table, group_index);
				table[group_index] = target;
			}
		}

		(indices, table)
	}

	pub fn components(groups: &[It]) -> C2 {
		let (_, gindices) = Self::separate_components(groups);
		let mut gb = FxHashMap::<usize, FxHashSet<N>>::default();
		for (i, n) in gindices
			.into_iter()
			.enumerate()
			.filter(|&(_, n)| n != usize::MAX)
		{
			let set = gb.entry(n).or_default();
			for e in groups[i].clone() {
				set.insert(e);
			}
		}

		gb.into_values().map(|v| v.into_iter().collect()).collect()
	}

	pub fn connected_components<IN>(starts: &[N], mut neighbors: impl FnMut(&N) -> IN) -> C2
	where
		IN: IntoIterator<Item = N>,
	{
		ConnectedComponents::<N, Vec<N>, It2, C1, C2, C3>::components(
			&starts
				.iter()
				.map(|s| {
					neighbors(s)
						.into_iter()
						.chain(once(s.clone()))
						.collect::<Vec<_>>()
				})
				.collect::<Vec<_>>(),
		)
	}

	pub fn component_index(components: &[It2]) -> C3 {
		components
			.iter()
			.enumerate()
			.flat_map(|(i, c)| c.into_iter().map(move |n| (n.clone(), i)))
			.collect()
	}
}

#[must_use]
pub fn separate_components<N>(groups: &[Vec<N>]) -> (HashMap<&N, usize>, Vec<usize>)
where
	N: Clone + Eq + Hash,
{
	ConnectedComponents::<N>::separate_components(groups)
}

#[must_use]
pub fn components<N>(groups: &[Vec<N>]) -> Vec<HashSet<N>>
where
	N: Clone + Eq + Hash,
{
	ConnectedComponents::<N>::components(groups)
}

pub fn connected_components<N, IN>(starts: &[N], neighbors: impl FnMut(&N) -> IN) -> Vec<HashSet<N>>
where
	N: Clone + Eq + Hash,
	IN: IntoIterator<Item = N>,
{
	ConnectedComponents::<N>::connected_components(starts, neighbors)
}

#[must_use]
#[expect(clippy::implicit_hasher)]
pub fn component_index<N>(components: &[HashSet<N>]) -> HashMap<N, usize>
where
	N: Clone + Eq + Hash,
{
	ConnectedComponents::<N>::component_index(components)
}

fn find(table: &mut [usize], mut x: usize) -> usize {
	while table[x] != x {
		let t = table[x];
		table[x] = table[table[x]];
		x = t;
	}

	x
}
