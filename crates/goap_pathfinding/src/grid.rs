use std::{
	collections::BTreeSet,
	fmt::{Debug, Formatter, Result as FmtResult, Write as _},
	iter::FusedIterator,
	ops::Sub,
};

use super::{
	FxIndexSet,
	directed::{bfs::bfs_reach, dfs::dfs_reach},
	matrix::Matrix,
	num_traits::ToPrimitive,
	utils::constrain,
};

#[derive(Clone)]
pub struct Grid {
	pub width: usize,
	pub height: usize,
	diagonal_mode: bool,
	dense: bool,
	exclusions: FxIndexSet<(usize, usize)>,
}

impl Grid {
	#[must_use]
	pub fn new(width: usize, height: usize) -> Self {
		Self {
			width,
			height,
			diagonal_mode: false,
			dense: false,
			exclusions: FxIndexSet::default(),
		}
	}

	#[must_use]
	pub const fn is_inside(&self, vertex: (usize, usize)) -> bool {
		vertex.0 < self.width && vertex.1 < self.height
	}

	#[must_use]
	pub const fn diagonal_mode(&self) -> bool {
		self.diagonal_mode
	}

	pub const fn diagonal_mode_mut(&mut self) -> &mut bool {
		&mut self.diagonal_mode
	}

	pub fn resize(&mut self, width: usize, height: usize) -> bool {
		let mut truncated = false;
		if width < self.width {
			truncated |=
				(width..self.width).any(|c| (0..self.height).any(|r| self.has_vertex((c, r))));
		}

		if height < self.height {
			truncated |=
				(0..self.width).any(|c| (height..self.height).any(|r| self.has_vertex((c, r))));
		}
		self.exclusions.retain(|&(x, y)| x < width && y < height);
		if self.dense {
			for c in self.width..width {
				for r in 0..height {
					self.exclusions.insert((c, r));
				}
			}

			for c in 0..self.width.min(width) {
				for r in self.height..height {
					self.exclusions.insert((c, r));
				}
			}
		}

		self.width = width;
		self.height = height;
		self.rebalance();

		truncated
	}

	#[must_use]
	pub const fn size(&self) -> usize {
		self.width * self.height
	}

	#[must_use]
	pub fn vertices_len(&self) -> usize {
		if self.dense {
			self.size() - self.exclusions.len()
		} else {
			self.exclusions.len()
		}
	}

	pub fn add_vertex(&mut self, vertex: (usize, usize)) -> bool {
		if !self.is_inside(vertex) {
			return false;
		}

		let r = if self.dense {
			self.exclusions.swap_remove(&vertex)
		} else {
			self.exclusions.insert(vertex)
		};

		self.rebalance();
		r
	}

	pub fn remove_vertex(&mut self, vertex: (usize, usize)) -> bool {
		if !self.is_inside(vertex) {
			return false;
		}

		let r = if self.dense {
			self.exclusions.insert(vertex)
		} else {
			self.exclusions.swap_remove(&vertex)
		};

		self.rebalance();
		r
	}

	fn borders(&self) -> impl Iterator<Item = (usize, usize)> + use<> {
		let width = self.width;
		let height = self.height;

		(0..width)
			.flat_map(move |x| vec![(x, 0), (x, height - 1)].into_iter())
			.chain((1..height - 1).flat_map(move |y| vec![(0, y), (width - 1, y)].into_iter()))
	}

	pub fn add_borders(&mut self) -> usize {
		if matches!(self.width, 0) || matches!(self.height, 0) {
			return 0;
		}

		let count = if self.dense {
			self.borders()
				.filter(|v| self.exclusions.swap_remove(v))
				.count()
		} else {
			self.borders()
				.filter(|v| self.exclusions.insert(*v))
				.count()
		};

		self.rebalance();
		count
	}

	pub fn remove_borders(&mut self) -> usize {
		if matches!(self.width, 0) || matches!(self.height, 0) {
			return 0;
		}

		let count = if self.dense {
			self.borders()
				.filter(|v| self.exclusions.insert(*v))
				.count()
		} else {
			self.borders()
				.filter(|v| self.exclusions.swap_remove(v))
				.count()
		};

		self.rebalance();
		count
	}

	fn rebalance(&mut self) {
		if self.exclusions.len() > self.width * self.height / 2 {
			self.exclusions = (0..self.width)
				.flat_map(|c| (0..self.height).map(move |r| (c, r)))
				.filter(|v| !self.exclusions.contains(v))
				.collect();
			self.invert();
		}
	}

	pub fn clear(&mut self) -> bool {
		let r = !self.is_empty();
		self.dense = false;
		self.exclusions.clear();
		r
	}

	pub fn fill(&mut self) -> bool {
		let r = !self.is_full();
		self.clear();
		self.invert();
		r
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		if self.dense {
			self.exclusions.len() == self.size()
		} else {
			self.exclusions.is_empty()
		}
	}

	#[must_use]
	pub fn is_full(&self) -> bool {
		if self.dense {
			self.exclusions.is_empty()
		} else {
			self.exclusions.len() == self.size()
		}
	}

	pub const fn invert(&mut self) {
		self.dense = !self.dense;
	}

	#[must_use]
	pub fn has_vertex(&self, vertex: (usize, usize)) -> bool {
		self.is_inside(vertex) && (self.exclusions.contains(&vertex) ^ self.dense)
	}

	#[must_use]
	pub fn has_edge(&self, v1: (usize, usize), v2: (usize, usize)) -> bool {
		if !self.has_vertex(v1) || !self.has_vertex(v2) {
			return false;
		}

		let x = v1.0.abs_diff(v2.0);
		let y = v1.1.abs_diff(v2.1);
		matches!(x + y, 1) || (matches!((x, y), (1, 1)) && self.diagonal_mode())
	}

	#[must_use]
	pub fn neighbors(&self, vertex: (usize, usize)) -> Vec<(usize, usize)> {
		if !self.has_vertex(vertex) {
			return Vec::new();
		}

		let (x, y) = vertex;
		let mut candidates = Vec::with_capacity(8);
		if x > 0 {
			candidates.push((x - 1, y));
			if self.diagonal_mode() {
				if y > 0 {
					candidates.push((x - 1, y - 1));
				}

				if y + 1 < self.height {
					candidates.push((x - 1, y + 1));
				}
			}
		}

		if x + 1 < self.width {
			candidates.push((x + 1, y));
			if self.diagonal_mode() {
				if y > 0 {
					candidates.push((x + 1, y - 1));
				}

				if y + 1 < self.height {
					candidates.push((x + 1, y + 1));
				}
			}
		}

		if y > 0 {
			candidates.push((x, y - 1));
		}

		if y + 1 < self.height {
			candidates.push((x, y + 1));
		}

		candidates.retain(|&v| self.has_vertex(v));

		candidates
	}

	pub fn bfs_reachable(
		&self,
		start: (usize, usize),
		mut predicate: impl FnMut((usize, usize)) -> bool,
	) -> BTreeSet<(usize, usize)> {
		bfs_reach(start, |&n| {
			self.neighbors(n)
				.into_iter()
				.filter(|&n| predicate(n))
				.collect::<Vec<_>>()
		})
		.collect()
	}

	pub fn dfs_reachable(
		&self,
		start: (usize, usize),
		mut predicate: impl FnMut((usize, usize)) -> bool,
	) -> BTreeSet<(usize, usize)> {
		dfs_reach(start, |&n| {
			self.neighbors(n)
				.into_iter()
				.filter(|&n| predicate(n))
				.collect::<Vec<_>>()
		})
		.collect()
	}

	#[must_use]
	pub fn distance(&self, a: (usize, usize), b: (usize, usize)) -> usize {
		let (dx, dy) = (a.0.abs_diff(b.0), a.1.abs_diff(b.1));
		if self.diagonal_mode() {
			dx.max(dy)
		} else {
			dx + dy
		}
	}

	pub fn from_coordinates<T>(points: &[(T, T)]) -> Option<Self>
	where
		T: Copy + Default + Ord + Sub<Output = T> + ToPrimitive,
	{
		let (min_x, min_y) = (
			points
				.iter()
				.map(|(x, ..)| x)
				.min()
				.copied()
				.unwrap_or_default(),
			points
				.iter()
				.map(|(.., y)| y)
				.min()
				.copied()
				.unwrap_or_default(),
		);

		points
			.iter()
			.map(|(x, y)| Some(((*x - min_x).to_usize()?, (*y - min_y).to_usize()?)))
			.collect()
	}

	#[must_use]
	pub const fn constrain(&self, vertex: (isize, isize)) -> (usize, usize) {
		(
			constrain(vertex.0, self.width),
			constrain(vertex.1, self.height),
		)
	}

	#[must_use]
	pub const fn iter(&self) -> Iter<'_> {
		Iter {
			grid: self,
			x: 0,
			y: 0,
		}
	}

	#[must_use]
	pub const fn edges(&self) -> Edges<'_> {
		Edges {
			grid: self,
			x: 0,
			y: 0,
			i: 0,
		}
	}
}

impl Debug for Grid {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let (present, absent) = if f.alternate() {
			('▓', '░')
		} else {
			('#', '.')
		};
		let lines = if f.sign_minus() {
			(0..self.height).rev().collect::<Vec<_>>()
		} else {
			(0..self.height).collect()
		};

		let last = lines.last().copied().unwrap();
		for y in lines {
			for x in 0..self.width {
				f.write_char(if self.has_vertex((x, y)) {
					present
				} else {
					absent
				})?;
			}

			if y != last {
				f.write_char('\n')?;
			}
		}

		Ok(())
	}
}

impl Eq for Grid {}

impl From<Matrix<bool>> for Grid {
	fn from(value: Matrix<bool>) -> Self {
		Self::from(&value)
	}
}

impl From<&Matrix<bool>> for Grid {
	fn from(value: &Matrix<bool>) -> Self {
		let mut grid = Self::new(value.columns, value.rows);
		for ((r, c), &v) in value.items() {
			if v {
				grid.add_vertex((c, r));
			}
		}

		grid
	}
}

impl FromIterator<(usize, usize)> for Grid {
	fn from_iter<T>(iter: T) -> Self
	where
		T: IntoIterator<Item = (usize, usize)>,
	{
		let vertices = iter.into_iter().collect();
		let mut width = 0;
		let mut height = 0;
		for &(x, y) in &vertices {
			if x + 1 > width {
				width = x + 1;
			}

			if y + 1 > height {
				height = y + 1;
			}
		}

		let mut grid = Self {
			width,
			height,
			diagonal_mode: false,
			dense: false,
			exclusions: vertices,
		};

		grid.rebalance();
		grid
	}
}

impl IntoIterator for Grid {
	type IntoIter = IntoIter;
	type Item = (usize, usize);

	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			grid: self,
			x: 0,
			y: 0,
		}
	}
}

impl<'a> IntoIterator for &'a Grid {
	type IntoIter = Iter<'a>;
	type Item = (usize, usize);

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl PartialEq for Grid {
	fn eq(&self, other: &Self) -> bool {
		self.vertices_len() == other.vertices_len()
			&& self.iter().zip(other.iter()).all(|(a, b)| a == b)
	}
}

pub struct Edges<'a> {
	grid: &'a Grid,
	x: usize,
	y: usize,
	i: usize,
}

impl FusedIterator for Edges<'_> {}

impl Iterator for Edges<'_> {
	type Item = ((usize, usize), (usize, usize));

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			if self.y == self.grid.height {
				return None;
			}

			let x = self.x;
			let y = self.y;
			let other = match self.i {
				0 => (x + 1, y),
				1 => (x, y + 1),
				2 => (x + 1, y + 1),
				_ => (x - 1, y + 1),
			};
			self.i += 1;
			if matches!((x, self.i), (0, 3)) || matches!(self.i, 4) {
				self.i = 0;
				self.x += 1;
				if self.x == self.grid.width {
					self.x = 0;
					self.y += 1;
				}
			}

			if self.grid.has_edge((x, y), other) {
				return Some(((x, y), other));
			}
		}
	}
}

pub struct IntoIter {
	grid: Grid,
	x: usize,
	y: usize,
}

impl FusedIterator for IntoIter {}

impl Iterator for IntoIter {
	type Item = (usize, usize);

	fn next(&mut self) -> Option<Self::Item> {
		if self.grid.dense {
			loop {
				if self.y == self.grid.height {
					return None;
				}

				let r = (self.grid.has_vertex((self.x, self.y))).then_some((self.x, self.y));
				self.x += 1;
				if self.x == self.grid.width {
					self.x = 0;
					self.y += 1;
				}

				if r.is_some() {
					return r;
				}
			}
		} else {
			self.grid.exclusions.pop()
		}
	}
}

pub struct Iter<'a> {
	grid: &'a Grid,
	x: usize,
	y: usize,
}

impl FusedIterator for Iter<'_> {}

impl Iterator for Iter<'_> {
	type Item = (usize, usize);

	fn next(&mut self) -> Option<Self::Item> {
		if self.grid.dense {
			loop {
				if self.y == self.grid.height {
					return None;
				}

				let r = (self.grid.has_vertex((self.x, self.y))).then_some((self.x, self.y));
				self.x += 1;
				if self.x == self.grid.width {
					self.x = 0;
					self.y += 1;
				}

				if r.is_some() {
					return r;
				}
			}
		} else {
			self.grid
				.exclusions
				.get_index(self.x)
				.inspect(|_| {
					self.x += 1;
				})
				.copied()
		}
	}
}
