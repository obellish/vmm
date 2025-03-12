use std::iter::Sum;

use super::{
	FxIndexSet,
	matrix::Matrix,
	num_traits::{Bounded, Signed, Zero},
};

pub trait Weights<C> {
	fn rows(&self) -> usize;

	fn columns(&self) -> usize;

	fn at(&self, row: usize, column: usize) -> C;

	#[must_use]
	fn neg(&self) -> Self
	where
		Self: Sized,
		C: Signed;
}

impl<C: Copy> Weights<C> for Matrix<C> {
	fn rows(&self) -> usize {
		self.rows
	}

	fn columns(&self) -> usize {
		self.columns
	}

	fn at(&self, row: usize, column: usize) -> C {
		self[(row, column)]
	}

	fn neg(&self) -> Self
	where
		Self: Sized,
		C: Signed,
	{
		-self.clone()
	}
}

pub fn kuhn_munkres<C>(weights: &impl Weights<C>) -> (C, Vec<usize>)
where
	C: Bounded + Copy + Ord + Signed + Sum<C> + Zero,
{
	let nx = weights.rows();
	let ny = weights.columns();
	assert!(
		nx <= ny,
		"number of rows must be larger than number of columns"
	);

	let mut xy = vec![None::<usize>; nx];
	let mut yx = vec![None::<usize>; ny];
	let mut lx = (0..nx)
		.map(|row| (0..ny).map(|col| weights.at(row, col)).max().unwrap())
		.collect::<Vec<_>>();
	let mut ly = vec![C::zero(); ny];
	let mut s = FxIndexSet::default();
	let mut alternating = Vec::with_capacity(ny);
	let mut slack = vec![C::zero(); ny];
	let mut slackx = Vec::with_capacity(ny);
	for root in 0..nx {
		alternating.clear();
		alternating.resize(ny, None);
		let mut y = {
			s.clear();
			s.insert(root);
			for y in 0..ny {
				slack[y] = lx[root] + ly[y] - weights.at(root, y);
			}
			slackx.clear();
			slackx.resize(ny, root);
			Some(loop {
				let mut delta = C::max_value();
				let mut x = 0;
				let mut y = 0;
				for yy in 0..ny {
					if alternating[yy].is_none() && slack[yy] < delta {
						delta = slack[yy];
						x = slackx[yy];
						y = yy;
					}
				}

				if delta > C::zero() {
					for &x in &s {
						lx[x] = lx[x] - delta;
					}

					for y in 0..ny {
						if alternating[y].is_some() {
							ly[y] = ly[y] + delta;
						} else {
							slack[y] = slack[y] - delta;
						}
					}
				}

				alternating[y] = Some(x);
				if yx[y].is_none() {
					break y;
				}

				let x = yx[y].unwrap();
				s.insert(x);

				for y in 0..ny {
					if alternating[y].is_none() {
						let alternate_slack = lx[x] + ly[y] - weights.at(x, y);
						if slack[y] > alternate_slack {
							slack[y] = alternate_slack;
							slackx[y] = x;
						}
					}
				}
			})
		};

		while y.is_some() {
			let x = alternating[y.unwrap()].unwrap();
			let prec = xy[x];
			yx[y.unwrap()] = Some(x);
			xy[x] = y;
			y = prec;
		}
	}

	(
		lx.into_iter().sum::<C>() + ly.into_iter().sum(),
		xy.into_iter().map(Option::unwrap).collect(),
	)
}

pub fn kuhn_munkres_min<C>(weights: &impl Weights<C>) -> (C, Vec<usize>)
where
	C: Bounded + Copy + Ord + Signed + Sum<C> + Zero,
{
	let (total, assignments) = kuhn_munkres(&weights.neg());
	(-total, assignments)
}
