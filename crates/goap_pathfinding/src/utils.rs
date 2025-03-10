use integer_sqrt::IntegerSquareRoot;

use super::num_traits::{PrimInt, Unsigned};

pub fn uint_sqrt<T>(n: T) -> Option<T>
where
	T: PrimInt + Unsigned,
{
	let root = n.integer_sqrt();
	(n == root * root).then_some(root)
}

#[must_use]
pub fn move_in_direction(
	(row, col): (usize, usize),
	direction: (isize, isize),
	dimensions: (usize, usize),
) -> Option<(usize, usize)> {
	if row >= dimensions.0 || col >= dimensions.1 || matches!(direction, (0, 0)) {
		return None;
	}

	let (new_row, new_col) = (row as isize + direction.0, col as isize + direction.1);
	(new_row >= 0
		&& (new_row as usize) < dimensions.0
		&& new_col >= 0
		&& (new_col as usize) < dimensions.1)
		.then_some((new_row as usize, new_col as usize))
}

pub fn in_direction(
	start: (usize, usize),
	direction: (isize, isize),
	dimensions: (usize, usize),
) -> impl Iterator<Item = (usize, usize)> {
	std::iter::successors(Some(start), move |current| {
		move_in_direction(*current, direction, dimensions)
	})
	.skip(1)
}

#[must_use]
pub const fn constrain(value: isize, upper: usize) -> usize {
	if value > 0 {
		value as usize % upper
	} else {
		(upper - (-value) as usize % upper) % upper
	}
}
