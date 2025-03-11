pub mod directions;

use std::{
	collections::BTreeSet,
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	iter::FusedIterator,
	ops::{Deref, DerefMut, Index, IndexMut, Neg, Range},
	slice::{Iter, IterMut},
};

use super::{
	directed::{bfs::bfs_reach, dfs::dfs_reach},
	num_traits::Signed,
	utils::{constrain, in_direction, move_in_direction, uint_sqrt},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Matrix<C> {
	pub rows: usize,
	pub columns: usize,
	data: Vec<C>,
}

impl<C> Matrix<C> {
	pub fn try_from_iter(
		rows: usize,
		columns: usize,
		values: impl IntoIterator<Item = C>,
	) -> Result<Self, MatrixFormatError> {
		let values = values.into_iter().collect::<Vec<_>>();

		if rows * columns != values.len() {
			return Err(MatrixFormatError::WrongLength);
		}

		if !matches!(rows, 0) && matches!(columns, 0) {
			return Err(MatrixFormatError::EmptyRow);
		}

		Ok(Self {
			rows,
			columns,
			data: values,
		})
	}

	pub fn try_square_from_iter(
		values: impl IntoIterator<Item = C>,
	) -> Result<Self, MatrixFormatError> {
		let values = values.into_iter().collect::<Vec<_>>();
		let Some(size) = uint_sqrt(values.len()) else {
			return Err(MatrixFormatError::WrongLength);
		};

		Self::try_from_iter(size, size, values)
	}

	#[must_use]
	pub const fn empty(columns: usize) -> Self {
		Self {
			rows: 0,
			columns,
			data: Vec::new(),
		}
	}

	#[must_use]
	pub const fn is_empty(&self) -> bool {
		matches!(self.rows, 0)
	}

	pub fn try_from_rows<IC>(rows: impl IntoIterator<Item = IC>) -> Result<Self, MatrixFormatError>
	where
		IC: IntoIterator<Item = C>,
	{
		let mut rows = rows.into_iter();
		if let Some(first_row) = rows.next() {
			let mut data = first_row.into_iter().collect::<Vec<C>>();
			let number_of_columns = data.len();
			let mut number_of_rows = 1;
			for row in rows {
				number_of_rows += 1;
				data.extend(row);
				if number_of_rows * number_of_columns != data.len() {
					return Err(MatrixFormatError::WrongLength);
				}
			}

			Self::try_from_iter(number_of_rows, number_of_columns, data)
		} else {
			Ok(Self::empty(0))
		}
	}

	#[must_use]
	pub const fn is_square(&self) -> bool {
		self.rows == self.columns
	}

	#[must_use]
	pub const unsafe fn idx_unchecked(&self, i: (usize, usize)) -> usize {
		i.0 * self.columns + i.1
	}

	#[must_use]
	pub fn idx(&self, i: (usize, usize)) -> usize {
		assert!(
			i.0 < self.rows,
			"trying to access row {} (max {})",
			i.0,
			self.rows - 1
		);
		assert!(
			i.1 < self.columns,
			"trying to access column {} (max {})",
			i.1,
			self.columns - 1
		);

		unsafe { self.idx_unchecked(i) }
	}

	#[must_use]
	pub const fn constrain(&self, (row, column): (isize, isize)) -> (usize, usize) {
		(constrain(row, self.rows), constrain(column, self.columns))
	}

	#[must_use]
	pub const fn within_bounds(&self, (row, column): (usize, usize)) -> bool {
		row < self.rows && column < self.columns
	}

	#[must_use]
	pub fn get(&self, i: (usize, usize)) -> Option<&C> {
		self.within_bounds(i)
			.then(|| &self.data[unsafe { self.idx_unchecked(i) }])
	}

	pub fn get_mut(&mut self, i: (usize, usize)) -> Option<&mut C> {
		self.within_bounds(i).then(|| {
			let idx = unsafe { self.idx_unchecked(i) };
			&mut self.data[idx]
		})
	}

	pub fn flip_lr(&mut self) {
		for r in 0..self.rows {
			self.data[r * self.columns..(r + 1) * self.columns].reverse();
		}
	}

	pub fn flip_ud(&mut self) {
		for r in 0..self.rows / 2 {
			for c in 0..self.columns {
				self.data
					.swap(r * self.columns + c, (self.rows - 1 - r) * self.columns + c);
			}
		}
	}

	pub fn rotate_cw(&mut self, times: usize) {
		assert!(self.is_square(), "attempt to rotate a non-square matrix");

		match times % 4 {
			0 => (),
			2 => self.data.reverse(),
			n => {
				for r in 0..self.rows / 2 {
					for c in 0..self.columns.div_ceil(2) {
						let i1 = r * self.columns + c;
						let i2 = c * self.columns + self.columns - 1 - r;
						let i3 = (self.rows - 1 - r) * self.columns + self.columns - 1 - c;
						let i4 = (self.rows - 1 - c) * self.columns + r;
						if matches!(n, 1) {
							self.data.swap(i1, i2);
							self.data.swap(i1, i4);
							self.data.swap(i3, i4);
						} else {
							self.data.swap(i3, i4);
							self.data.swap(i1, i4);
							self.data.swap(i1, i2);
						}
					}
				}
			}
		}
	}

	pub fn rotate_ccw(&mut self, times: usize) {
		self.rotate_cw(4 - (times % 4));
	}

	pub fn neighbors(
		&self,
		(r, c): (usize, usize),
		diagonals: bool,
	) -> impl Iterator<Item = (usize, usize)> + use<C> {
		let (row_range, column_range) = if r < self.rows && c < self.columns {
			(
				r.saturating_sub(1)..(self.rows).min(r + 2),
				c.saturating_sub(1)..(self.columns).min(c + 2),
			)
		} else {
			(0..0, 0..0)
		};

		row_range
			.flat_map(move |r| column_range.clone().map(move |c| (r, c)))
			.filter(move |&(rr, cc)| (rr != r || cc != c) && (diagonals || rr == r || cc == c))
	}

	#[must_use]
	pub fn move_in_direction(
		&self,
		start: (usize, usize),
		direction: (isize, isize),
	) -> Option<(usize, usize)> {
		move_in_direction(start, direction, (self.rows, self.columns))
	}

	pub fn in_direction(
		&self,
		start: (usize, usize),
		direction: (isize, isize),
	) -> impl Iterator<Item = (usize, usize)> + use<C> {
		in_direction(start, direction, (self.rows, self.columns))
	}

	#[must_use]
	pub fn iter(&self) -> RowIterator<'_, C> {
		self.into_iter()
	}

	#[must_use]
	pub const fn columns_iter(&self) -> ColumnIterator<'_, C> {
		ColumnIterator {
			matrix: self,
			column: 0,
		}
	}

	pub fn keys(&self) -> impl Iterator<Item = (usize, usize)> + use<C> {
		let columns = self.columns;
		(0..self.rows).flat_map(move |r| (0..columns).map(move |c| (r, c)))
	}

	pub fn values(&self) -> Iter<'_, C> {
		self.data.iter()
	}

	pub fn values_mut(&mut self) -> IterMut<'_, C> {
		self.data.iter_mut()
	}

	pub fn items(&self) -> impl Iterator<Item = ((usize, usize), &C)> {
		self.keys().zip(self.values())
	}

	pub fn items_mut(&mut self) -> impl Iterator<Item = ((usize, usize), &mut C)> {
		self.keys().zip(self.values_mut())
	}

	pub fn bfs_reachable(
		&self,
		start: (usize, usize),
		diagonals: bool,
		mut predicate: impl FnMut((usize, usize)) -> bool,
	) -> BTreeSet<(usize, usize)> {
		bfs_reach(start, |&n| {
			self.neighbors(n, diagonals)
				.filter(|&n| predicate(n))
				.collect::<Vec<_>>()
		})
		.collect()
	}

	pub fn dfs_reachable(
		&self,
		start: (usize, usize),
		diagonals: bool,
		mut predicate: impl FnMut((usize, usize)) -> bool,
	) -> BTreeSet<(usize, usize)> {
		dfs_reach(start, |&n| {
			self.neighbors(n, diagonals)
				.filter(|&n| predicate(n))
				.collect::<Vec<_>>()
		})
		.collect()
	}

	fn transpose_in_place_non_square(&mut self) {
		let m = self.columns;
		let n = self.rows;

		let mn1 = m * n - 1;

		let mut visited = vec![0u8; (m * n + 7).div_ceil(8)];

		for s in 1..self.data.len() {
			if !matches!(visited[s / 8] & (1 << (s % 8)), 0) {
				continue;
			}

			let mut x = s;
			loop {
				if x != mn1 {
					x = (n * x) % mn1;
				}

				self.data.swap(x, s);
				visited[x / 8] |= 1 << (x % 8);

				if x == s {
					break;
				}
			}
		}

		self.rows = m;
		self.columns = n;
	}

	pub fn transpose(&mut self) {
		if self.is_square() {
			for r in 0..self.rows {
				for c in r + 1..self.columns {
					self.data.swap(r * self.columns + c, c * self.columns + r);
				}
			}
		} else {
			self.transpose_in_place_non_square();
		}
	}

	pub fn set_slice(&mut self, (row, column): (usize, usize), slice: &Self)
	where
		C: Copy,
	{
		let height = (self.rows - row).min(slice.rows);
		let width = (self.columns - column).min(slice.columns);
		for r in 0..height {
			self.data[(row + r) * self.columns + column..(row + r) * self.columns + column + width]
				.copy_from_slice(&slice.data[r * slice.columns..r * slice.columns + width]);
		}
	}

	pub fn map<O>(self, transform: impl FnMut(C) -> O) -> Matrix<O> {
		Matrix {
			rows: self.rows,
			columns: self.columns,
			data: self.data.into_iter().map(transform).collect(),
		}
	}

	pub fn swap(&mut self, a: (usize, usize), b: (usize, usize)) {
		let (a, b) = (self.idx(a), self.idx(b));
		self.data.swap(a, b);
	}

	pub fn from_fn(rows: usize, columns: usize, cb: impl FnMut((usize, usize)) -> C) -> Self {
		assert!(
			matches!(rows, 0) || columns > 0,
			"unable to create a matrix with empty rows"
		);

		Self {
			rows,
			columns,
			data: (0..rows)
				.flat_map(move |row| (0..columns).map(move |column| (row, column)))
				.map(cb)
				.collect(),
		}
	}
}

impl<C: Clone> Matrix<C> {
	pub fn extend(&mut self, row: &[C]) -> Result<(), MatrixFormatError> {
		if row.is_empty() {
			return Err(MatrixFormatError::EmptyRow);
		}

		if self.columns != row.len() {
			return Err(MatrixFormatError::WrongLength);
		}

		self.rows += 1;
		for e in row {
			self.data.push(e.clone());
		}

		Ok(())
	}

	#[must_use]
	pub fn transposed(&self) -> Self {
		assert!(
			!matches!(self.rows, 0) || matches!(self.columns, 0),
			"this operation would create a matrix with empty rows"
		);

		Self {
			rows: self.columns,
			columns: self.rows,
			data: (0..self.columns)
				.flat_map(|c| (0..self.rows).map(move |r| self.data[r * self.columns + c].clone()))
				.collect(),
		}
	}

	#[must_use]
	pub fn flipped_ud(&self) -> Self {
		let mut copy = self.clone();
		copy.flip_ud();
		copy
	}

	#[must_use]
	pub fn flipped_lr(&self) -> Self {
		let mut copy = self.clone();
		copy.flip_lr();
		copy
	}

	#[must_use]
	pub fn rotated_cw(&self, times: usize) -> Self {
		if self.is_square() {
			let mut copy = self.clone();
			copy.rotate_cw(times);
			copy
		} else {
			match times % 4 {
				0 => self.clone(),
				1 => {
					let mut copy = self.transposed();
					copy.flip_lr();
					copy
				}
				2 => {
					let mut copy = self.clone();
					copy.data.reverse();
					copy
				}
				_ => {
					let mut copy = self.transposed();
					copy.flip_ud();
					copy
				}
			}
		}
	}

	#[must_use]
	pub fn rotated_ccw(&self, times: usize) -> Self {
		self.rotated_cw(4 - (times % 4))
	}

	pub fn try_slice(
		&self,
		rows: Range<usize>,
		columns: Range<usize>,
	) -> Result<Self, MatrixFormatError> {
		if rows.end > self.rows || columns.end > self.columns {
			return Err(MatrixFormatError::WrongIndex);
		}

		let height = rows.end - rows.start;
		let width = columns.end - columns.start;
		let mut v = Vec::with_capacity(height * width);
		for r in rows {
			v.extend(
				self.data[r * self.columns + columns.start..r * self.columns + columns.end]
					.iter()
					.cloned(),
			);
		}

		Self::try_from_iter(height, width, v)
	}

	pub fn fill(&mut self, value: C) {
		self.data.fill(value);
	}

	pub fn square(size: usize, value: C) -> Self {
		Self::new(size, size, value)
	}

	pub fn new(rows: usize, columns: usize, value: C) -> Self {
		assert!(
			matches!(rows, 0) || columns > 0,
			"unable to create a matrix with empty rows"
		);
		Self {
			rows,
			columns,
			data: vec![value; rows * columns],
		}
	}
}

impl<C> Deref for Matrix<C> {
	type Target = [C];

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl<C> DerefMut for Matrix<C> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.data
	}
}

impl<C, IC> FromIterator<IC> for Matrix<C>
where
	IC: IntoIterator<Item = C>,
{
	fn from_iter<T: IntoIterator<Item = IC>>(iter: T) -> Self {
		match Self::try_from_rows(iter) {
			Ok(m) => m,
			Err(e) => panic!("{e}"),
		}
	}
}

impl<C> Index<(usize, usize)> for Matrix<C> {
	type Output = C;

	fn index(&self, index: (usize, usize)) -> &Self::Output {
		&self.data[self.idx(index)]
	}
}

impl<C> Index<&(usize, usize)> for Matrix<C> {
	type Output = C;

	fn index(&self, index: &(usize, usize)) -> &Self::Output {
		&self[*index]
	}
}

impl<C> IndexMut<(usize, usize)> for Matrix<C> {
	fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
		let i = self.idx(index);
		&mut self.data[i]
	}
}

impl<C> IndexMut<&(usize, usize)> for Matrix<C> {
	fn index_mut(&mut self, index: &(usize, usize)) -> &mut Self::Output {
		&mut self[*index]
	}
}

impl<'a, C> IntoIterator for &'a Matrix<C> {
	type IntoIter = RowIterator<'a, C>;
	type Item = &'a [C];

	fn into_iter(self) -> Self::IntoIter {
		RowIterator {
			matrix: self,
			row: 0,
		}
	}
}

impl<C> Neg for Matrix<C>
where
	C: Clone + Signed,
{
	type Output = Self;

	fn neg(self) -> Self::Output {
		Self {
			rows: self.rows,
			columns: self.columns,
			data: self.data.iter().map(|x| -x.clone()).collect(),
		}
	}
}

pub struct RowIterator<'a, C> {
	matrix: &'a Matrix<C>,
	row: usize,
}

impl<C> DoubleEndedIterator for RowIterator<'_, C> {
	fn next_back(&mut self) -> Option<Self::Item> {
		(self.row < self.matrix.rows).then(|| {
			let row = self.matrix.rows - self.row;
			self.row += 1;
			&self.matrix.data[(row - 1) * self.matrix.columns..row * self.matrix.columns]
		})
	}
}

impl<C> FusedIterator for RowIterator<'_, C> {}

impl<'a, C> Iterator for RowIterator<'a, C> {
	type Item = &'a [C];

	fn next(&mut self) -> Option<Self::Item> {
		(self.row < self.matrix.rows).then(|| {
			self.row += 1;
			&self.matrix.data[(self.row - 1) * self.matrix.columns..self.row * self.matrix.columns]
		})
	}
}

pub struct ColumnIterator<'a, C> {
	matrix: &'a Matrix<C>,
	column: usize,
}

impl<C> DoubleEndedIterator for ColumnIterator<'_, C> {
	fn next_back(&mut self) -> Option<Self::Item> {
		(self.column < self.matrix.columns).then(|| {
			self.column += 1;
			let column = self.matrix.columns - self.column;
			(0..self.matrix.rows)
				.map(|r| &self.matrix[(r, column)])
				.collect()
		})
	}
}

impl<C> FusedIterator for ColumnIterator<'_, C> {}

impl<'a, C> Iterator for ColumnIterator<'a, C> {
	type Item = Vec<&'a C>;

	fn next(&mut self) -> Option<Self::Item> {
		(self.column < self.matrix.columns).then(|| {
			self.column += 1;
			(0..self.matrix.rows)
				.map(|r| &self.matrix[(r, self.column - 1)])
				.collect()
		})
	}
}

#[derive(Debug)]
pub enum MatrixFormatError {
	EmptyRow,
	WrongIndex,
	WrongLength,
}

impl Display for MatrixFormatError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::EmptyRow => "matrix rows cannot be empty",
			Self::WrongIndex => "index does not point to data inside the matrix",
			Self::WrongLength => "provided data does not correspond to the expected length",
		})
	}
}

impl StdError for MatrixFormatError {}
