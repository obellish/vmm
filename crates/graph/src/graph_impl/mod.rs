#![expect(clippy::new_without_default)]

use alloc::vec::Vec;
use core::{
	cmp::max,
	error::Error as CoreError,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	hash::Hash,
	marker::PhantomData,
};

use super::{Directed, Direction, EdgeType, IntoWeightedEdge, Undirected};
use crate::iter_format::{DebugMap, IterFormatExt, NoPretty};

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

impl<N, E, Ty: EdgeType, Ix: IndexType> Graph<N, E, Ty, Ix> {
	#[must_use]
	pub fn with_capacity(nodes: usize, edges: usize) -> Self {
		Self {
			nodes: Vec::with_capacity(nodes),
			edges: Vec::with_capacity(edges),
			ty: PhantomData,
		}
	}

	#[must_use]
	pub fn node_count(&self) -> usize {
		self.nodes.len()
	}

	#[must_use]
	pub fn edge_count(&self) -> usize {
		self.edges.len()
	}

	#[allow(clippy::unused_self)]
	#[must_use]
	pub fn is_directed(&self) -> bool {
		Ty::is_directed()
	}

	#[track_caller]
	pub fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
		self.try_add_node(weight).unwrap()
	}

	pub fn try_add_node(&mut self, weight: N) -> Result<NodeIndex<Ix>, GraphError> {
		let node = Node {
			weight,
			next: [EdgeIndex::end(), EdgeIndex::end()],
		};

		let node_idx = NodeIndex::new(self.node_count());

		if <Ix as IndexType>::max().index() == !0 || NodeIndex::end() != node_idx {
			self.nodes.push(node);
			Ok(node_idx)
		} else {
			Err(GraphError::NodeIxLimit)
		}
	}

	pub fn node_weight(&self, a: NodeIndex<Ix>) -> Option<&N> {
		self.nodes.get(a.index()).map(|n| &n.weight)
	}

	pub fn node_weight_mut(&mut self, a: NodeIndex<Ix>) -> Option<&mut N> {
		self.nodes.get_mut(a.index()).map(|n| &mut n.weight)
	}

	#[track_caller]
	pub fn add_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> EdgeIndex<Ix> {
		let res = self.try_add_edge(a, b, weight);
		if matches!(res, Err(GraphError::NodeOutOfBounds)) {
			panic!("node indices out of bounds");
		}

		res.unwrap()
	}

	pub fn try_add_edge(
		&mut self,
		a: NodeIndex<Ix>,
		b: NodeIndex<Ix>,
		weight: E,
	) -> Result<EdgeIndex<Ix>, GraphError> {
		let edge_idx = EdgeIndex::new(self.edge_count());
		if !(<Ix as IndexType>::max().index() == !0 || EdgeIndex::end() != edge_idx) {
			return Err(GraphError::EdgeIxLimit);
		}

		let mut edge = Edge {
			weight,
			node: [a, b],
			next: [EdgeIndex::end(); 2],
		};

		match index_twice(&mut self.nodes, a.index(), b.index()) {
			Pair::None => return Err(GraphError::NodeOutOfBounds),
			Pair::One(an) => {
				edge.next = an.next;
				an.next[0] = edge_idx;
				an.next[1] = edge_idx;
			}
			Pair::Both(an, bn) => {
				edge.next = [an.next[0], bn.next[1]];
				an.next[0] = edge_idx;
				bn.next[1] = edge_idx;
			}
		}

		self.edges.push(edge);
		Ok(edge_idx)
	}

	#[track_caller]
	pub fn update_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> EdgeIndex<Ix> {
		self.try_update_edge(a, b, weight).unwrap()
	}

	pub fn try_update_edge(
		&mut self,
		a: NodeIndex<Ix>,
		b: NodeIndex<Ix>,
		weight: E,
	) -> Result<EdgeIndex<Ix>, GraphError> {
		if let Some(ix) = self.find_edge(a, b) {
			if let Some(ed) = self.edge_weight_mut(ix) {
				*ed = weight;
				return Ok(ix);
			}
		}

		self.try_add_edge(a, b, weight)
	}

	pub fn edge_weight(&self, e: EdgeIndex<Ix>) -> Option<&E> {
		self.edges.get(e.index()).map(|ed| &ed.weight)
	}

	pub fn edge_weight_mut(&mut self, e: EdgeIndex<Ix>) -> Option<&mut E> {
		self.edges.get_mut(e.index()).map(|ed| &mut ed.weight)
	}

	pub fn edge_endpoints(&self, e: EdgeIndex<Ix>) -> Option<(NodeIndex<Ix>, NodeIndex<Ix>)> {
		self.edges
			.get(e.index())
			.map(|ed| (ed.source(), ed.target()))
	}

	fn change_edge_links(
		&mut self,
		edge_node: [NodeIndex<Ix>; 2],
		e: EdgeIndex<Ix>,
		edge_next: [EdgeIndex<Ix>; 2],
	) {
		for &d in &DIRECTIONS {
			let k = d.index();
			let Some(node) = self.nodes.get_mut(edge_node[k].index()) else {
				debug_assert!(
					false,
					"edge's endpoint dir={:?} index={:?} not found",
					d, edge_node[k]
				);
				return;
			};
			let fst = node.next[k];
			if fst == e {
				node.next[k] = edge_next[k];
			} else {
				todo!()
			}
		}
	}

	pub fn find_edge(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> Option<EdgeIndex<Ix>> {
		if self.is_directed() {
			let node = self.nodes.get(a.index())?;

			self.find_edge_directed_from_node(node, b)
		} else {
			self.find_edge_undirected(a, b).map(|(ix, ..)| ix)
		}
	}

	fn find_edge_directed_from_node(
		&self,
		node: &Node<N, Ix>,
		b: NodeIndex<Ix>,
	) -> Option<EdgeIndex<Ix>> {
		let mut edix = node.next[0];
		while let Some(edge) = self.edges.get(edix.index()) {
			if edge.node[1] == b {
				return Some(edix);
			}

			edix = edge.next[0];
		}

		None
	}

	pub fn find_edge_undirected(
		&self,
		a: NodeIndex<Ix>,
		b: NodeIndex<Ix>,
	) -> Option<(EdgeIndex<Ix>, Direction)> {
		let node = self.nodes.get(a.index())?;

		self.find_edge_undirected_from_node(node, b)
	}

	fn find_edge_undirected_from_node(
		&self,
		node: &Node<N, Ix>,
		b: NodeIndex<Ix>,
	) -> Option<(EdgeIndex<Ix>, Direction)> {
		for &d in &DIRECTIONS {
			let k = d.index();
			let mut edix = node.next[k];
			while let Some(edge) = self.edges.get(edix.index()) {
				if edge.node[1 - k] == b {
					return Some((edix, d));
				}

				edix = edge.next[k];
			}
		}

		None
	}
}

impl<N, E> Graph<N, E, Directed> {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			nodes: Vec::new(),
			edges: Vec::new(),
			ty: PhantomData,
		}
	}
}

impl<N, E> Graph<N, E, Undirected> {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			nodes: Vec::new(),
			edges: Vec::new(),
			ty: PhantomData,
		}
	}
}

impl<N: Clone, E: Clone, Ty, Ix: IndexType> Clone for Graph<N, E, Ty, Ix> {
	fn clone(&self) -> Self {
		Self {
			nodes: self.nodes.clone(),
			edges: self.edges.clone(),
			ty: self.ty,
		}
	}

	fn clone_from(&mut self, source: &Self) {
		self.nodes.clone_from(&source.nodes);
		self.edges.clone_from(&source.edges);
		self.ty = source.ty;
	}
}

impl<N: Debug, E: Debug, Ty: EdgeType, Ix: IndexType> Debug for Graph<N, E, Ty, Ix> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let etype = if self.is_directed() {
			"Directed"
		} else {
			"Undirected"
		};

		let mut fmt_struct = f.debug_struct("Graph");
		fmt_struct
			.field("ty", &etype)
			.field("node_count", &self.node_count())
			.field("edge_count", &self.edge_count());
		if self.edge_count() > 0 {
			fmt_struct.field(
				"edges",
				&self
					.edges
					.iter()
					.map(|e| NoPretty((e.source().index(), e.target().index())))
					.format(", "),
			);
		}

		if !matches!(size_of::<N>(), 0) {
			fmt_struct.field(
				"node_weights",
				&DebugMap(|| self.nodes.iter().map(|n| &n.weight).enumerate()),
			);
		}

		if !matches!(size_of::<E>(), 0) {
			fmt_struct.field(
				"edge_weights",
				&DebugMap(|| self.edges.iter().map(|n| &n.weight).enumerate()),
			);
		}

		fmt_struct.finish()
	}
}

struct EdgesWalkerMut<'a, E: 'a, Ix: IndexType = DefaultIx> {
	edges: &'a mut [Edge<E, Ix>],
	next: EdgeIndex<Ix>,
	dir: Direction,
}

impl<'a, E: 'a, Ix: IndexType> EdgesWalkerMut<'a, E, Ix> {
	const fn new(edges: &'a mut [Edge<E, Ix>], next: EdgeIndex<Ix>, dir: Direction) -> Self {
		Self { edges, next, dir }
	}

	fn next_edge(&mut self) -> Option<&mut Edge<E, Ix>> {
		None
	}
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
