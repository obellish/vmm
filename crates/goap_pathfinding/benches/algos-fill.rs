use criterion::{Criterion, criterion_group, criterion_main};
use goap_pathfinding::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Pt {
	x: u16,
	y: u16,
	_fill: [u64; 32],
}

impl Pt {
	const fn new(x: u16, y: u16) -> Self {
		Self {
			x,
			y,
			_fill: [0; 32],
		}
	}

	#[inline]
	const fn heuristic(p: &Self) -> usize {
		(64 - p.x - p.y) as usize
	}
}

#[inline]
fn successors(pt: &Pt) -> Vec<Pt> {
	let mut ret = Vec::with_capacity(4);
	if 0 < pt.x {
		ret.push(Pt::new(pt.x - 1, pt.y));
	}
	if pt.x < 32 {
		ret.push(Pt::new(pt.x + 1, pt.y));
	}
	if 0 < pt.y {
		ret.push(Pt::new(pt.x, pt.y - 1));
	}
	if pt.y < 32 {
		ret.push(Pt::new(pt.x, pt.y + 1));
	}
	ret
}

fn corner_to_corner_astar(c: &mut Criterion) {
	c.bench_function(
		&format!("{}-fill", stringify!(corner_to_corner_astar)),
		|b| {
			b.iter(|| {
				assert_ne!(
					astar(
						&Pt::new(0, 0),
						|n| successors(n).into_iter().map(|n| (n, 1)),
						Pt::heuristic,
						|n| matches!(n, Pt { x: 32, y: 32, .. })
					),
					None
				);
			});
		},
	);
}

fn corner_to_corner_bfs(c: &mut Criterion) {
	c.bench_function(&format!("{}-fill", stringify!(corner_to_corner_bfs)), |b| {
		b.iter(|| {
			assert_ne!(
				bfs(&Pt::new(0, 0), successors, |n| matches!(
					n,
					Pt { x: 32, y: 32, .. }
				)),
				None
			);
		});
	});
}

criterion_group!(benches, corner_to_corner_astar, corner_to_corner_bfs);
criterion_main!(benches);
