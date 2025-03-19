use super::{Point, Rotation, Scalar, Scale, Transform, Translation, Vector};

pub trait Transformation<S> {
	fn transform_point(&self, p: Point<S>) -> Point<S>;

	fn transform_vector(&self, v: Vector<S>) -> Vector<S>;
}

impl<S: Scalar> Transformation<S> for Transform<S> {
	fn transform_point(&self, p: Point<S>) -> Point<S> {
		self.transform_point(p)
	}

	fn transform_vector(&self, v: Vector<S>) -> Vector<S> {
		self.transform_vector(v)
	}
}

impl<S: Scalar> Transformation<S> for Rotation<S> {
	fn transform_point(&self, p: Point<S>) -> Point<S> {
		self.transform_point(p)
	}

	fn transform_vector(&self, v: Vector<S>) -> Vector<S> {
		self.transform_vector(v)
	}
}

impl<S: Scalar> Transformation<S> for Translation<S> {
	fn transform_point(&self, p: Point<S>) -> Point<S> {
		self.transform_point(p)
	}

	fn transform_vector(&self, v: Vector<S>) -> Vector<S> {
		v
	}
}

impl<S: Scalar> Transformation<S> for Scale<S> {
	fn transform_point(&self, p: Point<S>) -> Point<S> {
		(*self).transform_point(p)
	}

	fn transform_vector(&self, v: Vector<S>) -> Vector<S> {
		(*self).transform_vector(v)
	}
}

impl<S: Scalar, T> Transformation<S> for &T
where
	T: Transformation<S>,
{
	fn transform_point(&self, p: Point<S>) -> Point<S> {
		(*self).transform_point(p)
	}

	fn transform_vector(&self, v: Vector<S>) -> Vector<S> {
		(*self).transform_vector(v)
	}
}
