use std::{
	collections::{BTreeMap, BTreeSet, VecDeque},
	hash::Hash,
};

use super::bfs::bfs;
use crate::{
	FxIndexSet,
	matrix::Matrix,
	num_traits::{Bounded, Signed, Zero},
};

#[derive(Debug, Clone)]
pub struct Common<C> {
	size: usize,
	source: usize,
	sink: usize,
	total_capacity: C,
	details: bool,
}

pub trait EdmondsKarp<C>
where
	C: Bounded + Copy + Ord + Signed + Zero,
{
	fn new(size: usize, source: usize, sink: usize) -> Self
	where
		Self: Sized;

	fn from_matrix(source: usize, sink: usize, capacities: Matrix<C>) -> Self
	where
		Self: Sized;

	#[must_use]
	fn from_iter(source: usize, sink: usize, capacities: impl IntoIterator<Item = C>) -> Self
	where
		Self: Sized,
	{
		Self::from_matrix(
			source,
			sink,
			Matrix::try_square_from_iter(capacities).unwrap(),
		)
	}
}

pub type Edge<N, C> = ((N, N), C);

pub type EKFlows<N, C> = (Vec<Edge<N, C>>, C, Vec<Edge<N, C>>);
