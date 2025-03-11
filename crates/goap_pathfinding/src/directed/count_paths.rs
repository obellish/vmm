use std::hash::Hash;

use rustc_hash::FxHashMap;

pub fn count_paths<T, IN>(
	start: T,
	mut successors: impl FnMut(&T) -> IN,
	mut success: impl FnMut(&T) -> bool,
) -> usize
where
	T: Eq + Hash,
	IN: IntoIterator<Item = T>,
{
	cached_count_paths(
		start,
		&mut successors,
		&mut success,
		&mut FxHashMap::default(),
	)
}

fn cached_count_paths<T, IN>(
	start: T,
	successors: &mut impl FnMut(&T) -> IN,
	success: &mut impl FnMut(&T) -> bool,
	cache: &mut FxHashMap<T, usize>,
) -> usize
where
	T: Eq + Hash,
	IN: IntoIterator<Item = T>,
{
	if let Some(&n) = cache.get(&start) {
		return n;
	}

	let count = if success(&start) {
		1
	} else {
		successors(&start)
			.into_iter()
			.map(|successor| cached_count_paths(successor, successors, success, cache))
			.sum()
	};

	cache.insert(start, count);

	count
}

#[cfg(test)]
mod tests {
	use super::count_paths;

	#[test]
	fn grid() {
		let n = count_paths(
			(0, 0),
			|&(x, y)| {
				[(x + 1, y), (x, y + 1)]
					.into_iter()
					.filter(|&(x, y)| x < 8 && y < 8)
			},
			|&c| matches!(c, (7, 7)),
		);

		assert_eq!(n, 3432);
	}
}
