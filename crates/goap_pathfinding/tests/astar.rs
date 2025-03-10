use goap_pathfinding::directed::astar::astar;

static GOAL: Pos = Pos(4,6);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Pos(i32, i32);

impl Pos {
	const fn distance(&self, other: &Self) -> u32 {
		self.0.abs_diff(other.0) + self.1.abs_diff(other.1)
	}

	fn successors(&self) -> Vec<(Self, u32)> {
		let &Self(x, y) = self;
		[
			Self(x + 1, y + 1),
			Self(x + 1, y - 2),
			Self(x - 1, y + 2),
			Self(x - 1, y - 2),
			Self(x + 2, y + 1),
			Self(x + 2, y - 1),
			Self(x - 2, y + 1),
			Self(x - 2, y - 1),
		]
		.into_iter()
		.map(|p| (p, 1))
		.collect()
	}
}

#[test]
fn astar_docs() {
    let result = astar(&Pos(1, 1), Pos::successors, |p| p.distance(&GOAL) / 3, |p| *p == GOAL);

    assert_eq!(result.unwrap().1, 4);
}
