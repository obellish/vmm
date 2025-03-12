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

#[cfg(test)]
mod tests {
	use std::collections::HashSet;

	use itertools::Itertools;
	use rand::prelude::*;
	use rand_xorshift::XorShiftRng;

	use super::{components, connected_components, separate_components};

	#[test]
	fn basic_separate_components() {
		let groups = [vec![1, 2], vec![3, 4], vec![5, 6], vec![1, 4]];
		let (h, g) = separate_components(&groups);
		assert!([1, 2, 3, 4].iter().map(|n| h[n]).all_equal());
		assert_eq!(h[&5], h[&6]);
		assert_ne!(h[&1], h[&5]);
		assert_eq!(h.len(), 6);
		assert_eq!(g[0], g[1]);
		assert_eq!(g[0], g[3]);
		assert_ne!(g[0], g[2]);
		assert_eq!(g.len(), 4);
	}

	#[test]
	fn empty_separate_components() {
		let groups = [vec![1, 2], vec![3, 4], vec![], vec![1, 4]];
		let (h, g) = separate_components(&groups);
		assert!([1, 2, 3, 4].iter().map(|n| h[n]).all_equal());
		assert_eq!(h.len(), 4);
		assert_eq!(g[0], g[1]);
		assert_eq!(g[0], g[3]);
		assert_ne!(g[0], g[2]);
		assert_eq!(g[2], usize::MAX);
		assert_eq!(g.len(), 4);
	}

	#[test]
	fn basic_components() {
		let mut c = components(&[vec![1, 2], vec![3, 4], vec![5, 6], vec![1, 4, 7]]);
		c.sort_unstable_by_key(|v| *v.iter().min().unwrap());
		assert_eq!(c.len(), 2);
		assert_eq!(
			c[0].clone().into_iter().sorted().collect_vec(),
			[1, 2, 3, 4, 7]
		);

		assert_eq!(c[1].clone().into_iter().sorted().collect_vec(), [5, 6]);
	}

	#[test]
	fn empty_components() {
		let mut c = components(&[vec![1, 2], vec![3, 4], vec![], vec![1, 4, 7]]);
		c.sort_unstable_by_key(|v| *v.iter().min().unwrap());
		assert_eq!(c.len(), 1);
		assert_eq!(
			c[0].clone().into_iter().sorted().collect_vec(),
			[1, 2, 3, 4, 7]
		);
	}

	#[test]
	fn basic_connected_components() {
		let mut counter = 0;
		let mut c = connected_components(&[1, 4], |&n| {
			counter += 1;
			if matches!(n % 2, 0) {
				[2, 4, 6, 8]
			} else {
				[1, 3, 5, 7]
			}
		});
		c.sort_unstable_by_key(|v| *v.iter().min().unwrap());
		assert_eq!(c.len(), 2);
		assert_eq!(
			c[0].clone().into_iter().sorted().collect_vec(),
			[1, 3, 5, 7]
		);
		assert_eq!(
			c[1].clone().into_iter().sorted().collect_vec(),
			[2, 4, 6, 8]
		);
		assert_eq!(counter, 2);
	}

	#[test]
	#[expect(clippy::set_contains_or_insert)]
	#[cfg_attr(miri, ignore)]
	fn larger_separate_components() {
		let mut rng = XorShiftRng::from_seed([
			3, 42, 93, 129, 1, 85, 72, 42, 84, 23, 95, 212, 253, 10, 4, 2,
		]);
		let mut seen = HashSet::new();
		let mut components = (0..100)
			.map(|_| {
				let mut component = Vec::new();
				for _ in 0..100 {
					let node = rng.next_u64();
					if !seen.contains(&node) {
						seen.insert(node);
						component.push(node);
					}
				}

				component.sort_unstable();
				assert!(
					!component.is_empty(),
					"component is empty, rng seed needs changed"
				);
				component
			})
			.collect_vec();

		components.sort_unstable_by_key(|v| *v.iter().min().unwrap());
		let mut groups = components
			.iter()
			.flat_map(|component| {
				let mut component = component.clone();
				component.shuffle(&mut rng);
				let mut subcomponents = Vec::new();
				while !component.is_empty() {
					let cut = rng.random_range(0..component.len());
					let mut subcomponent = component.drain(cut..).collect_vec();
					if !component.is_empty() {
						subcomponent.push(component[0]);
					}

					subcomponent.shuffle(&mut rng);
					subcomponents.push(subcomponent);
				}
				subcomponents
			})
			.collect_vec();

		groups.shuffle(&mut rng);
		let (_, group_mappings) = separate_components(&groups);
		let mut out_groups = vec![HashSet::new(); groups.len()];
		for (i, n) in group_mappings.into_iter().enumerate() {
			assert!(
				n < groups.len(),
				"group index is greater than expected: {n}/{}",
				groups.len()
			);
			for &e in &groups[i] {
				out_groups[n].insert(e);
			}
		}

		let out_groups = out_groups
			.into_iter()
			.map(|g| g.into_iter().collect_vec())
			.collect_vec();
		let mut out_groups = out_groups
			.into_iter()
			.filter_map(|mut group| {
				if group.is_empty() {
					None
				} else {
					group.sort_unstable();
					Some(group)
				}
			})
			.collect_vec();

		out_groups.sort_by_key(|c| c[0]);
		assert_eq!(components, out_groups);
	}
}
