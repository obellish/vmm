#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

#[cfg(any(test, feature = "std"))]
extern crate std;

mod line;
mod scalar;
#[macro_use]
mod segment;
pub mod traits;
pub mod utils;

#[doc(inline)]
pub use {arrayvec, euclid};

pub use self::{
	euclid::{
		Angle,
		default::{Box2D, Point2D as Point, Scale, Size2D as Size, Vector2D as Vector},
	},
	line::LineSegment,
	scalar::Scalar,
	segment::{BoundingBox, Segment},
};

pub const fn vector<S>(x: S, y: S) -> Vector<S> {
	Vector::new(x, y)
}

pub const fn point<S>(x: S, y: S) -> Point<S> {
	Point::new(x, y)
}

pub const fn size<S>(w: S, h: S) -> Size<S> {
	Size::new(w, h)
}

pub type Transform<S> = self::euclid::default::Transform2D<S>;

pub type Rotation<S> = self::euclid::default::Rotation2D<S>;

pub type Translation<S> = self::euclid::default::Translation2D<S>;
