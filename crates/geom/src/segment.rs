use core::ops::Range;

use super::{Box2D, LineSegment, Point, Scalar, Vector, point};

#[expect(clippy::return_self_not_must_use)]
pub trait Segment: Copy + Sized {
	type Scalar: Scalar;

	fn from(&self) -> Point<Self::Scalar>;

	fn to(&self) -> Point<Self::Scalar>;

	fn sample(&self, t: Self::Scalar) -> Point<Self::Scalar>;

	fn derivative(&self, t: Self::Scalar) -> Vector<Self::Scalar>;

	fn split(&self, t: Self::Scalar) -> (Self, Self);

	fn before_split(&self, t: Self::Scalar) -> Self;

	fn after_split(&self, t: Self::Scalar) -> Self;

	fn split_range(&self, t_range: Range<Self::Scalar>) -> Self;

	fn flip(&self) -> Self;

	fn approximate_length(&self, tolerance: Self::Scalar) -> Self::Scalar;

	fn for_each_flattened_with_t(
		&self,
		tolerance: Self::Scalar,
		callback: &mut impl FnMut(&LineSegment<Self::Scalar>, Range<Self::Scalar>),
	);

	fn x(&self, t: Self::Scalar) -> Self::Scalar {
		self.sample(t).x
	}

	fn y(&self, t: Self::Scalar) -> Self::Scalar {
		self.sample(t).y
	}

	fn dx(&self, t: Self::Scalar) -> Self::Scalar {
		self.derivative(t).x
	}

	fn dy(&self, t: Self::Scalar) -> Self::Scalar {
		self.derivative(t).y
	}
}

pub trait BoundingBox {
	type Scalar: Scalar;

	fn bounding_range_x(&self) -> (Self::Scalar, Self::Scalar);

	fn bounding_range_y(&self) -> (Self::Scalar, Self::Scalar);

	fn fast_bounding_range_x(&self) -> (Self::Scalar, Self::Scalar);

	fn fast_bounding_range_y(&self) -> (Self::Scalar, Self::Scalar);

	fn bounding_box(&self) -> Box2D<Self::Scalar> {
		let (min_x, max_x) = self.bounding_range_x();
		let (min_y, max_y) = self.bounding_range_y();

		Box2D {
			min: point(min_x, min_y),
			max: point(max_x, max_y),
		}
	}

	fn fast_bounding_box(&self) -> Box2D<Self::Scalar> {
		let (min_x, max_x) = self.fast_bounding_range_x();
		let (min_y, max_y) = self.fast_bounding_range_y();

		Box2D {
			min: point(min_x, min_y),
			max: point(max_x, max_y),
		}
	}
}
