#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod prelude;

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Bundle)]
pub struct TransformBundle<T: Bundle> {
	pub transform: Transform,
	pub value: T,
}

impl<T: Bundle> TransformBundle<T> {
	pub const fn new(transform: Transform, value: T) -> Self {
		Self { transform, value }
	}
}

pub trait WithTransform: Bundle + Sized {
	fn with_transform(self, transform: Transform) -> TransformBundle<Self> {
		TransformBundle::new(transform, self)
	}

	fn with_xyz(self, x: f32, y: f32, z: f32) -> TransformBundle<Self> {
		self.with_transform(Transform::from_xyz(x, y, z))
	}
}

impl<T> WithTransform for T where T: Bundle + Sized {}
