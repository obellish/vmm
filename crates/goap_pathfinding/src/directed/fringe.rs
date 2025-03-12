use std::{collections::VecDeque, hash::Hash, mem};

use indexmap::map::Entry::{Occupied, Vacant};

use super::reverse_path;
use crate::{
	FxIndexMap,
	num_traits::{Bounded, Zero},
};

pub fn fringe<N, C, IN>(
	start: &N,
	mut successors: impl FnMut(&N) -> IN,
	mut heuristic: impl FnMut(&N) -> C,
	mut success: impl FnMut(&N) -> bool,
) -> Option<(Vec<N>, C)>
where
	N: Clone + Eq + Hash,
	C: Bounded + Copy + Ord + Zero,
	IN: IntoIterator<Item = (N, C)>,
{
	let mut now = VecDeque::new();
	let mut later = VecDeque::new();
	let mut parents = FxIndexMap::default();
	let mut flimit = heuristic(start);
	now.push_back(0);
	parents.insert(start.clone(), (usize::MAX, C::zero()));

	'main: loop {
		if now.is_empty() {
			break 'main None;
		}

		let mut fmin = C::max_value();
		while let Some(i) = now.pop_front() {
			let (g, successors) = {
				let (node, &(_, g)) = parents.get_index(i)?;
				let f = g + heuristic(node);
				if f > flimit {
					if f < fmin {
						fmin = f;
					}

					later.push_back(i);
					continue;
				}
				if success(node) {
					let path = reverse_path(&parents, |&(p, ..)| p, i);
					break 'main Some((path, g));
				}
				(g, successors(node))
			};

			for (successor, cost) in successors {
				let g_successor = g + cost;
				let n;
				match parents.entry(successor) {
					Vacant(e) => {
						n = e.index();
						e.insert((i, g_successor));
					}
					Occupied(mut e) if e.get().1 > g_successor => {
						n = e.index();
						e.insert((i, g_successor));
					}
					Occupied(_) => continue,
				}

				if !remove(&mut later, &n) {
					remove(&mut now, &n);
				}
				now.push_front(n);
			}
		}

		mem::swap(&mut now, &mut later);
		flimit = fmin;
	}
}

fn remove<T: Eq>(v: &mut VecDeque<T>, e: &T) -> bool {
	v.iter().position(|x| x == e).is_some_and(|index| {
		v.remove(index);
		true
	})
}
