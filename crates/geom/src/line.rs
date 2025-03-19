use core::{mem::swap, ops::Range};

use super::{
	BoundingBox, Box2D, Point, Scalar, Segment, Vector, point, traits::Transformation,
	utils::min_max, vector,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LineSegment<S> {
	pub from: Point<S>,
	pub to: Point<S>,
}

impl<S: Scalar> LineSegment<S> {
	pub fn sample(&self, t: S) -> Point<S> {
		self.from().lerp(self.to(), t)
	}

	pub fn x(&self, t: S) -> S {
		self.from().x * (S::ONE - t) + self.to().x * t
	}

	pub fn y(&self, t: S) -> S {
		self.from().y * (S::ONE - t) + self.to().y * t
	}

	pub const fn from(&self) -> Point<S> {
		self.from
	}

	pub const fn to(&self) -> Point<S> {
		self.to
	}

	pub fn solve_t_for_x(&self, x: S) -> S {
		let dx = self.to().x - self.from().x;
		if dx == S::ZERO {
			return S::ZERO;
		}

		(x - self.from().x) / dx
	}

	pub fn solve_t_for_y(&self, y: S) -> S {
		let dy = self.to().y - self.from().y;
		if dy == S::ZERO {
			return S::ZERO;
		}

		(y - self.from().y) / dy
	}

	pub fn solve_y_for_x(&self, x: S) -> S {
		self.y(self.solve_t_for_x(x))
	}

	pub fn solve_x_for_y(&self, y: S) -> S {
		self.x(self.solve_t_for_y(y))
	}

	#[must_use]
	pub const fn flip(&self) -> Self {
		Self {
			from: self.to(),
			to: self.from(),
		}
	}

	#[must_use]
	pub fn split_range(&self, t_range: Range<S>) -> Self {
		Self {
			from: self.sample(t_range.start),
			to: self.sample(t_range.end),
		}
	}

	pub fn split(&self, t: S) -> (Self, Self) {
		let split_point = self.sample(t);

		(
			Self {
				from: self.from(),
				to: split_point,
			},
			Self {
				from: split_point,
				to: self.to(),
			},
		)
	}

	#[must_use]
	pub fn before_split(&self, t: S) -> Self {
		Self {
			from: self.from(),
			to: self.sample(t),
		}
	}

	#[must_use]
	pub fn after_split(&self, t: S) -> Self {
		Self {
			from: self.sample(t),
			to: self.to(),
		}
	}

	pub fn split_at_x(&self, x: S) -> (Self, Self) {
		self.split(self.solve_t_for_x(x))
	}
}

impl<S: Scalar> BoundingBox for LineSegment<S> {
	type Scalar = S;

	fn bounding_range_x(&self) -> (Self::Scalar, Self::Scalar) {
		min_max(self.from().x, self.to().x)
	}

	fn bounding_range_y(&self) -> (Self::Scalar, Self::Scalar) {
		min_max(self.from().y, self.to().y)
	}

	fn fast_bounding_range_x(&self) -> (Self::Scalar, Self::Scalar) {
		self.bounding_range_x()
	}

	fn fast_bounding_range_y(&self) -> (Self::Scalar, Self::Scalar) {
		self.bounding_range_y()
	}
}
