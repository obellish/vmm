use std::collections::HashSet;

use criterion::{Criterion, criterion_group, criterion_main};
use goap_pathfinding::prelude::*;
use itertools::Itertools;
use rand::{Rng as _, RngCore as _, SeedableRng as _, seq::SliceRandom};
use rand_xorshift::XorShiftRng;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Pt {
	x: u16,
	y: u16,
}

impl Pt {
	const fn new(x: u16, y: u16) -> Self {
		Self { x, y }
	}

	#[inline]
	const fn heuristic(p: &Self) -> usize {
		(128 - p.x - p.y) as usize
	}

	fn successors(pt: &Self) -> Vec<Self> {
		let mut ret = Vec::with_capacity(4);
		if 0 < pt.x {
			ret.push(Self::new(pt.x - 1, pt.y));
		}

		if pt.x < 64 {
			ret.push(Self::new(pt.x + 1, pt.y));
		}

		if 0 < pt.y {
			ret.push(Self::new(pt.x, pt.y - 1));
		}

		if pt.y < 64 {
			ret.push(Self::new(pt.x, pt.y + 1));
		}

		ret
	}

	const fn correct(n: &Self) -> bool {
		matches!(n, Self { x: 64, y: 64 })
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct BigPt {
	x: u16,
	y: u16,
	_fill: [u64; 32],
}

impl BigPt {
	const fn new(x: u16, y: u16) -> Self {
		Self {
			x,
			y,
			_fill: [0; 32],
		}
	}

	const fn heuristic(p: &Self) -> usize {
		(64 - p.x - p.y) as usize
	}

	fn successors(pt: &Self) -> Vec<Self> {
		let mut ret = Vec::with_capacity(4);
		if 0 < pt.x {
			ret.push(Self::new(pt.x - 1, pt.y));
		}
		if pt.x < 32 {
			ret.push(Self::new(pt.x + 1, pt.y));
		}
		if 0 < pt.y {
			ret.push(Self::new(pt.x, pt.y - 1));
		}
		if pt.y < 32 {
			ret.push(Self::new(pt.x, pt.y + 1));
		}

		ret
	}

	const fn correct(n: &Self) -> bool {
		matches!(n, Self { x: 32, y: 32, .. })
	}
}

fn corner_to_corner_astar(c: &mut Criterion) {
	let mut group: criterion::BenchmarkGroup<'_, criterion::measurement::WallTime> =
		c.benchmark_group(stringify!(corner_to_corner_astar));
	group.bench_function("small struct", |b| {
		b.iter(|| {
			assert_ne!(
				astar(
					&Pt::new(0, 0),
					|n| Pt::successors(n).into_iter().map(|n| (n, 1)),
					Pt::heuristic,
					Pt::correct
				),
				None
			);
		});
	});
	group.bench_function("large struct", |b| {
		b.iter(|| {
			assert_ne!(
				astar(
					&BigPt::new(0, 0),
					|n| BigPt::successors(n).into_iter().map(|n| (n, 1)),
					BigPt::heuristic,
					BigPt::correct
				),
				None
			);
		});
	});
}

fn corner_to_corner_bfs(c: &mut Criterion) {
	let mut group = c.benchmark_group(stringify!(corner_to_corner_bfs));
	group.bench_function("small struct", |b| {
		b.iter(|| {
			assert_ne!(bfs(&Pt::new(0, 0), Pt::successors, Pt::correct), None);
		});
	});
	group.bench_function("large struct", |b| {
		b.iter(|| {
			assert_ne!(
				bfs(&BigPt::new(0, 0), BigPt::successors, BigPt::correct),
				None
			);
		});
	});
}

fn corner_to_corner_bfs_bidirectional(c: &mut Criterion) {
	let mut group = c.benchmark_group(stringify!(corner_to_corner_bfs_bidirectional));
	group.bench_function("small struct", |b| {
		b.iter(|| {
			assert_ne!(
				bfs_bidirectional(
					&Pt::new(0, 0),
					&Pt::new(64, 64),
					Pt::successors,
					Pt::successors
				),
				None
			);
		});
	});
	group.bench_function("large struct", |b| {
		b.iter(|| {
			assert_ne!(
				bfs_bidirectional(
					&BigPt::new(0, 0),
					&BigPt::new(32, 32),
					BigPt::successors,
					BigPt::successors
				),
				None
			);
		});
	});
}

fn corner_to_corner_dfs(c: &mut Criterion) {
	let mut group = c.benchmark_group(stringify!(corner_to_corner_dfs));
	group.bench_function("small struct", |b| {
		b.iter(|| {
			assert_ne!(dfs(Pt::new(0, 0), Pt::successors, Pt::correct), None);
		});
	});
	group.bench_function("large struct", |b| {
		b.iter(|| {
			assert_ne!(
				dfs(BigPt::new(0, 0), BigPt::successors, BigPt::correct),
				None
			);
		});
	});
}

fn corner_to_corner_dijkstra(c: &mut Criterion) {
	let mut group = c.benchmark_group(stringify!(corner_to_corner_dijkstra));
	group.bench_function("small struct", |b| {
		b.iter(|| {
			assert_ne!(
				dijkstra(
					&Pt::new(0, 0),
					|n| Pt::successors(n).into_iter().map(|n| (n, 1)),
					Pt::correct
				),
				None
			);
		});
	});
	group.bench_function("large struct", |b| {
		b.iter(|| {
			assert_ne!(
				dijkstra(
					&BigPt::new(0, 0),
					|n| BigPt::successors(n).into_iter().map(|n| (n, 1)),
					BigPt::correct
				),
				None
			);
		});
	});
}

fn no_path_astar(c: &mut Criterion) {
	let mut group = c.benchmark_group(stringify!(no_path_astar));
	group.bench_function("small struct", |b| {
		b.iter(|| {
			assert!(
				astar(
					&Pt::new(2, 3),
					|n| Pt::successors(n).into_iter().map(|n| (n, 1)),
					|_| 1,
					|_| false
				)
				.is_none()
			);
		});
	});
	group.bench_function("large struct", |b| {
		b.iter(|| {
			assert!(
				astar(
					&BigPt::new(2, 3),
					|n| BigPt::successors(n).into_iter().map(|n| (n, 1)),
					|_| 1,
					|_| false
				)
				.is_none()
			);
		});
	});
}

fn no_path_bfs(c: &mut Criterion) {
	let mut group = c.benchmark_group(stringify!(no_path_bfs));
	group.bench_function("small struct", |b| {
		b.iter(|| {
			assert!(bfs(&Pt::new(2, 3), Pt::successors, |_| false).is_none());
		});
	});
	group.bench_function("large struct", |b| {
		b.iter(|| {
			assert!(bfs(&BigPt::new(2, 3), BigPt::successors, |_| false).is_none());
		});
	});
}

criterion_group!(
	benches,
	corner_to_corner_astar,
	corner_to_corner_bfs,
	corner_to_corner_bfs_bidirectional,
	corner_to_corner_dfs,
	corner_to_corner_dijkstra,
	no_path_astar,
	no_path_bfs
);
criterion_main!(benches);
