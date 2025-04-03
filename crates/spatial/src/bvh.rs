use std::{
	mem,
	slice::{Iter, IterMut},
};

use approx::abs_diff_eq;
use rayon::{
	iter::{
		IntoParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
	},
	slice::{Iter as ParIter, IterMut as ParIterMut},
};
use vek::{Aabb, Vec3};

use super::{Bounded3D, RaycastHit, SpatialIndex, ray_box_intersect};

#[derive(Debug, Clone)]
pub struct Bvh<T> {
	internal_nodes: Vec<InternalNode>,
	leaf_nodes: Vec<T>,
	root: NodeIdx,
}

impl<T> Bvh<T>
where
	T: Bounded3D + Send + Sync,
{
	#[must_use]
	pub const fn new() -> Self {
		Self {
			internal_nodes: Vec::new(),
			leaf_nodes: Vec::new(),
			root: NodeIdx::MAX,
		}
	}

	pub fn rebuild(&mut self, leaves: impl IntoIterator<Item = T>) {
		self.internal_nodes.clear();
		self.leaf_nodes.clear();

		self.leaf_nodes.extend(leaves);

		let leaf_count = self.leaf_nodes.len();

		if matches!(leaf_count, 0) {
			return;
		}

		self.internal_nodes.reserve_exact(leaf_count - 1);
		self.internal_nodes.resize(
			leaf_count - 1,
			InternalNode {
				bb: Aabb::default(),
				left: NodeIdx::MAX,
				right: NodeIdx::MAX,
			},
		);

		if NodeIdx::try_from(leaf_count)
			.ok()
			.and_then(|count| count.checked_add(count - 1))
			.is_none()
		{
			panic!("too many elements in BVH");
		}

		let id = self.leaf_nodes[0].aabb();
		let scene_bounds = self
			.leaf_nodes
			.par_iter()
			.map(Bounded3D::aabb)
			.reduce(|| id, Aabb::union);

		self.root = rebuild_rec(
			0,
			scene_bounds,
			&mut self.internal_nodes,
			&mut self.leaf_nodes,
			leaf_count as NodeIdx,
		)
		.0;

		debug_assert_eq!(self.internal_nodes.len(), self.leaf_nodes.len() - 1);
	}

	#[must_use]
	pub fn traverse(&self) -> Option<Node<'_, T>> {
		if self.leaf_nodes.is_empty() {
			None
		} else {
			Some(Node::from_idx(self, self.root))
		}
	}

	#[allow(clippy::iter_without_into_iter)]
	pub fn iter(&self) -> Iter<'_, T> {
		self.leaf_nodes.iter()
	}

	#[allow(clippy::iter_without_into_iter)]
	pub fn iter_mut(&mut self) -> IterMut<'_, T> {
		self.leaf_nodes.iter_mut()
	}

	#[must_use]
	pub fn par_iter(&self) -> ParIter<'_, T> {
		self.leaf_nodes.par_iter()
	}

	pub fn par_iter_mut(&mut self) -> ParIterMut<'_, T> {
		self.leaf_nodes.par_iter_mut()
	}
}

impl<T> Default for Bvh<T>
where
	T: Bounded3D + Send + Sync,
{
	fn default() -> Self {
		Self::new()
	}
}

impl<'a, T> IntoIterator for &'a Bvh<T>
where
	T: Bounded3D + Send + Sync,
{
	type IntoIter = Iter<'a, T>;
	type Item = &'a T;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, T> IntoIterator for &'a mut Bvh<T>
where
	T: Bounded3D + Send + Sync,
{
	type IntoIter = IterMut<'a, T>;
	type Item = &'a mut T;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

impl<'a, T> IntoParallelIterator for &'a Bvh<T>
where
	T: Bounded3D + Send + Sync,
{
	type Item = &'a T;
	type Iter = ParIter<'a, T>;

	fn into_par_iter(self) -> Self::Iter {
		self.par_iter()
	}
}

impl<'a, T> IntoParallelIterator for &'a mut Bvh<T>
where
	T: Bounded3D + Send + Sync,
{
	type Item = &'a mut T;
	type Iter = ParIterMut<'a, T>;

	fn into_par_iter(self) -> Self::Iter {
		self.par_iter_mut()
	}
}

impl<O> SpatialIndex for Bvh<O>
where
	O: Bounded3D + Send + Sync,
{
	type Object = O;

	fn query<T>(
		&self,
		mut collides: impl FnMut(Aabb<f64>) -> bool,
		mut f: impl FnMut(&Self::Object) -> Option<T>,
	) -> Option<T> {
		query_rec(self.traverse()?, &mut collides, &mut f)
	}

	fn raycast(
		&self,
		origin: Vec3<f64>,
		direction: Vec3<f64>,
		mut f: impl FnMut(RaycastHit<'_, Self::Object, f64>) -> bool,
	) -> Option<RaycastHit<'_, Self::Object, f64>> {
		debug_assert!(
			direction.is_normalized(),
			"the ray direction must be normalized"
		);

		let root = self.traverse()?;
		let (near, far) = ray_box_intersect(origin, direction, root.aabb())?;

		let mut hit = None;
		raycast_rec(root, &mut hit, near, far, origin, direction, &mut f);
		hit
	}
}

#[derive(Debug)]
pub struct Internal<'a, T> {
	bvh: &'a Bvh<T>,
	idx: NodeIdx,
}

impl<'a, T> Internal<'a, T> {
	#[must_use]
	pub fn split(self) -> (Aabb<f64>, Node<'a, T>, Node<'a, T>) {
		let internal = &self.bvh.internal_nodes[self.idx as usize];

		let bb = internal.bb;
		let left = Node::from_idx(self.bvh, internal.left);
		let right = Node::from_idx(self.bvh, internal.right);

		(bb, left, right)
	}
}

impl<T> Bounded3D for Internal<'_, T> {
	fn aabb(&self) -> Aabb<f64> {
		self.bvh.internal_nodes[self.idx as usize].bb
	}
}

impl<T> Clone for Internal<'_, T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T> Copy for Internal<'_, T> {}

#[derive(Debug, Clone)]
struct InternalNode {
	bb: Aabb<f64>,
	left: NodeIdx,
	right: NodeIdx,
}

#[derive(Debug)]
pub enum Node<'a, T> {
	Internal(Internal<'a, T>),
	Leaf(&'a T),
}

impl<'a, T> Node<'a, T> {
	fn from_idx(bvh: &'a Bvh<T>, idx: NodeIdx) -> Self {
		if idx < bvh.internal_nodes.len() as NodeIdx {
			Self::Internal(Internal { bvh, idx })
		} else {
			Self::Leaf(&bvh.leaf_nodes[(idx - bvh.internal_nodes.len() as NodeIdx) as usize])
		}
	}
}

impl<T: Bounded3D> Bounded3D for Node<'_, T> {
	fn aabb(&self) -> Aabb<f64> {
		match self {
			Self::Internal(i) => i.aabb(),
			Self::Leaf(l) => l.aabb(),
		}
	}
}

impl<T> Clone for Node<'_, T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T> Copy for Node<'_, T> {}

type NodeIdx = u32;

fn rebuild_rec<T>(
	idx: NodeIdx,
	mut bounds: Aabb<f64>,
	internal_nodes: &mut [InternalNode],
	leaf_nodes: &mut [T],
	total_leaf_count: NodeIdx,
) -> (NodeIdx, Aabb<f64>)
where
	T: Bounded3D + Send,
{
	debug_assert_eq!(leaf_nodes.len() - 1, internal_nodes.len());

	if matches!(leaf_nodes.len(), 1) {
		return (total_leaf_count - 1 + idx, leaf_nodes[0].aabb());
	}

	loop {
		debug_assert!(bounds.is_valid());

		let dims = bounds.max - bounds.min;

		let (mut split, bounds_left, bounds_right) = if dims.x >= dims.y && dims.x >= dims.z {
			let mid = middle(bounds.min.x, bounds.max.x);
			let [bounds_left, bounds_right] = bounds.split_at_x(mid);

			let p = partition(leaf_nodes, |l| {
				middle(l.aabb().min.x, l.aabb().max.x) <= mid
			});

			(p, bounds_left, bounds_right)
		} else if dims.y >= dims.x && dims.y >= dims.z {
			let mid = middle(bounds.min.y, bounds.max.y);
			let [bounds_left, bounds_right] = bounds.split_at_y(mid);

			let p = partition(leaf_nodes, |l| {
				middle(l.aabb().min.y, l.aabb().max.y) <= mid
			});

			(p, bounds_left, bounds_right)
		} else {
			let mid = middle(bounds.min.z, bounds.max.z);
			let [bounds_left, bounds_right] = bounds.split_at_z(mid);

			let p = partition(leaf_nodes, |l| {
				middle(l.aabb().min.z, l.aabb().max.z) <= mid
			});

			(p, bounds_left, bounds_right)
		};

		if matches!(split, 0) {
			if abs_diff_eq!(
				bounds_right.min,
				bounds_right.max,
				epsilon = f64::EPSILON * 100.0
			) {
				split += 1;
			} else {
				bounds = bounds_right;
				continue;
			}
		} else if split == leaf_nodes.len() {
			if abs_diff_eq!(
				bounds_left.min,
				bounds_left.max,
				epsilon = f64::EPSILON * 100.0
			) {
				split -= 1;
			} else {
				bounds = bounds_left;
				continue;
			}
		}

		let (leaves_left, leaves_right) = leaf_nodes.split_at_mut(split);

		let (internal_left, internal_right) = internal_nodes.split_at_mut(split);
		let (internal, internal_left) = internal_left.split_last_mut().unwrap();

		let ((left, bounds_left), (right, bounds_right)) = rayon::join(
			|| {
				rebuild_rec(
					idx,
					bounds_left,
					internal_left,
					leaves_left,
					total_leaf_count,
				)
			},
			|| {
				rebuild_rec(
					idx + split as NodeIdx,
					bounds_right,
					internal_right,
					leaves_right,
					total_leaf_count,
				)
			},
		);

		internal.bb = bounds_left.union(bounds_right);
		internal.left = left;
		internal.right = right;

		break (idx + split as NodeIdx - 1, internal.bb);
	}
}

fn partition<T>(s: &mut [T], mut pred: impl FnMut(&T) -> bool) -> usize {
	let mut it = s.iter_mut();
	let mut true_count = 0;

	while let Some(head) = it.find(|x| {
		if pred(x) {
			true_count += 1;
			false
		} else {
			true
		}
	}) {
		if let Some(tail) = it.rfind(|x| pred(x)) {
			mem::swap(head, tail);
			true_count += 1;
		} else {
			break;
		}
	}

	true_count
}

fn middle(a: f64, b: f64) -> f64 {
	(a + b) / 2.0
}

fn query_rec<O: Bounded3D, T>(
	node: Node<'_, O>,
	collides: &mut impl FnMut(Aabb<f64>) -> bool,
	f: &mut impl FnMut(&O) -> Option<T>,
) -> Option<T> {
	match node {
		Node::Internal(int) => {
			let (bb, left, right) = int.split();

			if collides(bb) {
				query_rec(left, collides, f).or_else(|| query_rec(right, collides, f))
			} else {
				None
			}
		}
		Node::Leaf(leaf) => {
			if collides(leaf.aabb()) {
				f(leaf)
			} else {
				None
			}
		}
	}
}

fn raycast_rec<'a, O: Bounded3D>(
	node: Node<'a, O>,
	hit: &mut Option<RaycastHit<'a, O>>,
	near: f64,
	far: f64,
	origin: Vec3<f64>,
	direction: Vec3<f64>,
	f: &mut impl FnMut(RaycastHit<'a, O>) -> bool,
) {
	if let Some(hit) = hit {
		if hit.near <= near {
			return;
		}
	}

	match node {
		Node::Internal(int) => {
			let (.., left, right) = int.split();

			let int_left = ray_box_intersect(origin, direction, left.aabb());
			let int_right = ray_box_intersect(origin, direction, right.aabb());

			match (int_left, int_right) {
				(Some((near_left, far_left)), Some((near_right, far_right))) => {
					if near_left < near_right {
						raycast_rec(left, hit, near_left, far_left, origin, direction, f);
						raycast_rec(right, hit, near_right, far_right, origin, direction, f);
					} else {
						raycast_rec(right, hit, near_right, far_right, origin, direction, f);
						raycast_rec(left, hit, near_left, far_left, origin, direction, f);
					}
				}
				(Some((near, far)), None) => {
					raycast_rec(left, hit, near, far, origin, direction, f);
				}
				(None, Some((near, far))) => {
					raycast_rec(right, hit, near, far, origin, direction, f);
				}
				(None, None) => {}
			}
		}
		Node::Leaf(leaf) => {
			let this_hit = RaycastHit {
				object: leaf,
				near,
				far,
			};

			if f(this_hit) {
				*hit = Some(this_hit);
			}
		}
	}
}
