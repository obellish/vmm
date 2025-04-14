use alloc::vec::Vec;
use core::{
	cmp::max,
	error::Error as CoreError,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	hash::Hash,
	marker::PhantomData,
};

use super::{Directed, Direction, EdgeType, IntoWeightedEdge, Undirected};

const DIRECTIONS: [Direction; 2] = [Direction::Outgoing, Direction::Incoming];

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeIndex<Ix = DefaultIx>(Ix);

impl<Ix: IndexType> EdgeIndex<Ix> {
	#[must_use]
	pub fn new(x: usize) -> Self {
		Self(IndexType::new(x))
	}

	pub fn index(self) -> usize {
		self.0.index()
	}

	#[must_use]
	pub fn end() -> Self {
		Self(IndexType::max())
	}

	const fn into_node(self) -> NodeIndex<Ix> {
		NodeIndex(self.0)
	}
}

impl<Ix: Debug> Debug for EdgeIndex<Ix> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_tuple("EdgeIndex").field(&self.0).finish()
	}
}

impl<Ix: IndexType> From<Ix> for EdgeIndex<Ix> {
	fn from(value: Ix) -> Self {
		Self(value)
	}
}

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct NodeIndex<Ix = DefaultIx>(Ix);

impl<Ix: IndexType> NodeIndex<Ix> {
	#[must_use]
	pub fn new(x: usize) -> Self {
		Self(IndexType::new(x))
	}

	pub fn index(self) -> usize {
		self.0.index()
	}

	#[must_use]
	pub fn end() -> Self {
		Self(IndexType::max())
	}

	const fn into_edge(self) -> EdgeIndex<Ix> {
		EdgeIndex(self.0)
	}
}

impl<Ix: Debug> Debug for NodeIndex<Ix> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_tuple("NodeIndex").field(&self.0).finish()
	}
}

impl<Ix: IndexType> From<Ix> for NodeIndex<Ix> {
	fn from(value: Ix) -> Self {
		Self(value)
	}
}

unsafe impl<Ix: IndexType> IndexType for NodeIndex<Ix> {
	fn index(&self) -> usize {
		self.0.index()
	}

	fn new(x: usize) -> Self {
		Self::new(x)
	}

	fn max() -> Self {
		Self(<Ix as IndexType>::max())
	}
}

#[derive(Default)]
pub struct Node<N, Ix = DefaultIx> {
	pub weight: N,
	next: [EdgeIndex<Ix>; 2],
}

impl<N, Ix: IndexType> Node<N, Ix> {
	pub const fn next_edge(&self, dir: Direction) -> EdgeIndex<Ix> {
		self.next[dir.index()]
	}
}

impl<E: Clone, Ix: Copy> Clone for Node<E, Ix> {
	fn clone(&self) -> Self {
		Self {
			weight: self.weight.clone(),
			next: self.next,
		}
	}
}

#[derive(Debug)]
pub struct Edge<E, Ix = DefaultIx> {
	pub weight: E,
	next: [EdgeIndex<Ix>; 2],
	node: [NodeIndex<Ix>; 2],
}

impl<E, Ix: IndexType> Edge<E, Ix> {
	pub const fn next_edge(&self, dir: Direction) -> EdgeIndex<Ix> {
		self.next[dir.index()]
	}

	pub const fn source(&self) -> NodeIndex<Ix> {
		self.node[0]
	}

	pub const fn target(&self) -> NodeIndex<Ix> {
		self.node[1]
	}
}

impl<E: Clone, Ix: Copy> Clone for Edge<E, Ix> {
	fn clone(&self) -> Self {
		Self {
			weight: self.weight.clone(),
			next: self.next,
			node: self.node,
		}
	}
}

pub struct Graph<N, E, Ty = Directed, Ix = DefaultIx> {
	nodes: Vec<Node<N, Ix>>,
	edges: Vec<Edge<E, Ix>>,
	ty: PhantomData<Ty>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphError {
	NodeIxLimit,
	EdgeIxLimit,
	NodeMissed(usize),
	NodeOutOfBounds,
}

impl Display for GraphError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::NodeIxLimit => f.write_str("graph at the maximum number of nodes for index"),
			Self::EdgeIxLimit => f.write_str("graph at the maximum number of edges for index"),
			Self::NodeMissed(i) => {
				f.write_str("node with index ")?;
				Display::fmt(&i, f)?;
				f.write_str(" is missing from graph")
			}
			Self::NodeOutOfBounds => f.write_str("node indices out of bounds"),
		}
	}
}

impl CoreError for GraphError {}

enum Pair<T> {
	Both(T, T),
	One(T),
	None,
}

pub unsafe trait IndexType: Copy + Debug + Default + Hash + Ord + 'static {
	fn new(x: usize) -> Self;

	fn index(&self) -> usize;

	fn max() -> Self;
}

unsafe impl IndexType for usize {
	fn new(x: usize) -> Self {
		x
	}

	fn index(&self) -> usize {
		*self
	}

	fn max() -> Self {
		Self::MAX
	}
}

unsafe impl IndexType for u32 {
	fn new(x: usize) -> Self {
		x as Self
	}

	fn index(&self) -> usize {
		*self as usize
	}

	fn max() -> Self {
		Self::MAX
	}
}

unsafe impl IndexType for u16 {
	fn new(x: usize) -> Self {
		x as Self
	}

	fn index(&self) -> usize {
		*self as usize
	}

	fn max() -> Self {
		Self::MAX
	}
}

unsafe impl IndexType for u8 {
	fn new(x: usize) -> Self {
		x as Self
	}

	fn index(&self) -> usize {
		*self as usize
	}

	fn max() -> Self {
		Self::MAX
	}
}

#[must_use]
pub fn node_index<Ix: IndexType>(index: usize) -> NodeIndex<Ix> {
	NodeIndex::new(index)
}

#[must_use]
pub fn edge_index<Ix: IndexType>(index: usize) -> EdgeIndex<Ix> {
	EdgeIndex::new(index)
}

fn index_twice<T>(slice: &mut [T], a: usize, b: usize) -> Pair<&mut T> {
	if max(a, b) >= slice.len() {
		Pair::None
	} else if a == b {
		Pair::One(&mut slice[max(a, b)])
	} else {
		unsafe {
			let ptr = slice.as_mut_ptr();
			let ar = &mut *ptr.add(a);
			let br = &mut *ptr.add(b);
			Pair::Both(ar, br)
		}
	}
}

pub type DefaultIx = u32;

pub type DiGraph<N, E, Ix = DefaultIx> = Graph<N, E, Directed, Ix>;

pub type UnGraph<N, E, Ix = DefaultIx> = Graph<N, E, Undirected, Ix>;
