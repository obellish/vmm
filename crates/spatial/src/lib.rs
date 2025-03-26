#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod bvh;

use vek::{Aabb, Vec3};

#[derive(Debug, PartialEq, Eq)]
pub struct RaycastHit<'a, O, N = f64> {
	pub object: &'a O,
	pub near: N,
	pub far: N,
}

impl<O, N: Clone> Clone for RaycastHit<'_, O, N> {
	fn clone(&self) -> Self {
		Self {
			object: self.object,
			near: self.near.clone(),
			far: self.far.clone(),
		}
	}
}

impl<O, N: Copy> Copy for RaycastHit<'_, O, N> {}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WithAabb<O, N = f64> {
	pub object: O,
	pub aabb: Aabb<N>,
}

impl<O, N> WithAabb<O, N> {
	pub const fn new(object: O, aabb: Aabb<N>) -> Self {
		Self { object, aabb }
	}
}

impl<O, N: Clone> Bounded3D<N> for WithAabb<O, N> {
	fn aabb(&self) -> Aabb<N> {
		self.aabb.clone()
	}
}

pub trait SpatialIndex<N = f64> {
	type Object: Bounded3D<N>;

	fn query<T>(
		&self,
		collides: impl FnMut(Aabb<N>) -> bool,
		f: impl FnMut(&Self::Object) -> Option<T>,
	) -> Option<T>;

	fn raycast(
		&self,
		origin: Vec3<f64>,
		direction: Vec3<f64>,
		f: impl FnMut(RaycastHit<'_, Self::Object, N>) -> bool,
	) -> Option<RaycastHit<'_, Self::Object, N>>;
}

pub trait Bounded3D<N = f64> {
	fn aabb(&self) -> Aabb<N>;
}

impl<N: Clone> Bounded3D<N> for Aabb<N> {
	fn aabb(&self) -> Self {
		self.clone()
	}
}

#[must_use]
pub fn ray_box_intersect(ro: Vec3<f64>, rd: Vec3<f64>, bb: Aabb<f64>) -> Option<(f64, f64)> {
	let mut near = -f64::INFINITY;
	let mut far = f64::INFINITY;

	for i in 0..3 {
		let t0 = (bb.min[i] - ro[i]) / rd[i];
		let t1 = (bb.max[i] - ro[i]) / rd[i];

		near = near.max(t0.min(t1));
		far = far.min(t0.max(t1));
	}

	if near <= far && far >= 0.0 {
		Some((near.max(0.0), far))
	} else {
		None
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn ray_box_edge_cases() {
		let bb = Aabb {
			min: Vec3::new(0.0, 0.0, 0.0),
			max: Vec3::new(1.0, 1.0, 1.0),
		};

		let ros = [
			// On a corner
			Vec3::new(0.0, 0.0, 0.0),
			// Outside
			Vec3::new(-0.5, 0.5, -0.5),
			// In the center
			Vec3::new(0.5, 0.5, 0.5),
			// On an edge
			Vec3::new(0.0, 0.5, 0.0),
			// On a face
			Vec3::new(0.0, 0.5, 0.5),
			// Outside slabs
			Vec3::new(-2.0, -2.0, -2.0),
		];

		let rds = [
			Vec3::new(1.0, 0.0, 0.0),
			Vec3::new(-1.0, 0.0, 0.0),
			Vec3::new(0.0, 1.0, 0.0),
			Vec3::new(0.0, -1.0, 0.0),
			Vec3::new(0.0, 0.0, 1.0),
			Vec3::new(0.0, 0.0, -1.0),
		];

		assert!(rds.iter().all(|d| d.is_normalized()));

		for ro in ros {
			for rd in rds {
				if let Some((near, far)) = ray_box_intersect(ro, rd, bb) {
					assert!(near.is_finite());
					assert!(far.is_finite());
					assert!(near <= far);
					assert!(near >= 0.0);
					assert!(far >= 0.0);
				}
			}
		}
	}
}
