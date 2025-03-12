#[derive(Debug)]
enum Path {
	FoundOptimum,
	Impossible,
	NoneAtThisDepth,
}

impl Path {
	fn step<N, IN>(
		path: &mut Vec<N>,
		successors: &mut impl FnMut(&N) -> IN,
		success: &mut impl FnMut(&N) -> bool,
		depth: usize,
	) -> Self
	where
		N: Eq,
		IN: IntoIterator<Item = N>,
	{
		if matches!(depth, 0) {
			Self::NoneAtThisDepth
		} else if success(path.last().unwrap()) {
			Self::FoundOptimum
		} else {
			let successors_it = successors(path.last().unwrap());

			let mut best_result = Self::Impossible;

			for n in successors_it {
				if !path.contains(&n) {
					path.push(n);
					match Self::step(path, successors, success, depth - 1) {
						Self::FoundOptimum => return Self::FoundOptimum,
						Self::NoneAtThisDepth => best_result = Self::NoneAtThisDepth,
						Self::Impossible => {}
					}
					path.pop();
				}
			}

			best_result
		}
	}
}

pub fn iddfs<N, IN>(
	start: N,
	mut successors: impl FnMut(&N) -> IN,
	mut success: impl FnMut(&N) -> bool,
) -> Option<Vec<N>>
where
	N: Eq,
	IN: IntoIterator<Item = N>,
{
	let mut path = vec![start];

	let mut current_max_depth = 1;

	loop {
		match Path::step(&mut path, &mut successors, &mut success, current_max_depth) {
			Path::FoundOptimum => break Some(path),
			Path::NoneAtThisDepth => current_max_depth += 1,
			Path::Impossible => break None,
		}
	}
}
