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
		self.from.lerp(self.to, t)
	}

	pub fn x(&self, t: S) -> S {
		self.from.x * (S::ONE - t) + self.to.x * t
	}
}
