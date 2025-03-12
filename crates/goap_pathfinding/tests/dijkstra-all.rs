use goap_pathfinding::prelude::*;
use rand::{Rng as _, rngs};

fn build_network(size: usize) -> Matrix<usize> {
	let mut network = Matrix::new(size, size, 0);
	let mut rng = rngs::ThreadRng::default();
	for a in 0..size {
		for b in 0..size {
			if rng.random_ratio(2, 3) {
				network[(a, b)] = rng.random::<u16>() as usize;
			}
		}
	}

	network
}

fn neighbors(network: Matrix<usize>) -> impl FnMut(&usize) -> Vec<(usize, usize)> {
	move |&a| {
		(0..network.rows)
			.filter_map(|b| match network[(a, b)] {
				0 => None,
				p => Some((b, p)),
			})
			.collect()
	}
}

#[test]
fn all_paths() {
	#[cfg(not(miri))]
	const SIZE: usize = 30;
	#[cfg(miri)]
	const SIZE: usize = 8;
	let network = build_network(SIZE);
	for start in 0..SIZE {
		let paths = dijkstra_all(&start, neighbors(network.clone()));
		for target in 0..SIZE {
			if let Some((path, cost)) =
				dijkstra(&start, neighbors(network.clone()), |&n| n == target)
			{
				if start == target {
					assert!(
						!paths.contains_key(&target),
						"path {start} -> {target} is present in {network:?}"
					);
				} else {
					assert!(
						paths.contains_key(&target),
						"path {start} -> {target} is not found in {network:?}"
					);

					assert_eq!(
						cost, paths[&target].1,
						"costs differ in path {start} -> {target} in {network:?}"
					);

					let other_path = build_path(&target, &paths);

					assert_eq!(
						path, other_path,
						"path {start} -> {target} differs in {network:?}: {path:?} vs {other_path:?}"
					);
				}
			} else {
				assert!(
					!paths.contains_key(&target),
					"path {start} -> {target} is present in {network:?}"
				);
			}
		}
	}
}

#[test]
fn partial_paths() {
	#[cfg(not(miri))]
	const SIZE: usize = 100;
	#[cfg(miri)]
	const SIZE: usize = 10;
	let network = build_network(SIZE);
	for start in 0..SIZE {
		let (paths, reached) = dijkstra_partial(&start, neighbors(network.clone()), |&n| {
			!matches!(start, 0) && !matches!(n, 0) && start != n && matches!(n % start, 0)
		});

		if let Some(target) = reached {
			assert_eq!(target % start, 0, "bad stop condition");

			let cost = paths[&target].1;
			let (path, dijkstra_cost) =
				dijkstra(&start, neighbors(network.clone()), |&n| n == target).unwrap();
			assert_eq!(
				cost, dijkstra_cost,
				"costs {start} -> {target} differ in {network:?}"
			);
			let other_path = build_path(&target, &paths);

			assert_eq!(
				path, other_path,
				"path {start} -> {target} differin {network:?}: {path:?} vs {other_path:?}"
			);
		} else if !matches!(start, 0) && start <= (SIZE - 1) / 2 {
			for target in 1..(SIZE / start) {
				assert!(
					dijkstra(&start, neighbors(network.clone()), |&n| n == target).is_none(),
					"path {start} -> {target} found in {network:?}"
				);
			}
		}
	}
}
