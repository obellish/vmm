#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

extern crate alloc;

#[cfg(any(feature = "std", test))]
extern crate std;

pub mod graph;
mod graph_impl;
mod iter_format;
mod iter_utils;
pub mod unionfind;
mod util;
#[macro_use]
pub mod visit;

use core::ops::Not;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
	feature = "serde",
	derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)
)]
#[repr(usize)]
pub enum Direction {
	Outgoing = 0,
	Incoming,
}

impl Direction {
	#[must_use]
	pub const fn opposite(self) -> Self {
		match self {
			Self::Outgoing => Self::Incoming,
			Self::Incoming => Self::Outgoing,
		}
	}

	#[must_use]
	pub const fn index(self) -> usize {
		(self as usize) & 0x1
	}
}

impl Not for Direction {
	type Output = Self;

	fn not(self) -> Self::Output {
		self.opposite()
	}
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Directed {}

impl EdgeType for Directed {
	fn is_directed() -> bool {
		true
	}
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Undirected {}

impl EdgeType for Undirected {
	fn is_directed() -> bool {
		false
	}
}

pub trait EdgeType {
	fn is_directed() -> bool;
}

pub trait IntoWeightedEdge<E> {
	type NodeId;

	fn into_weighted_edge(self) -> (Self::NodeId, Self::NodeId, E);
}

impl<Ix, E: Default> IntoWeightedEdge<E> for (Ix, Ix) {
	type NodeId = Ix;

	fn into_weighted_edge(self) -> (Self::NodeId, Self::NodeId, E) {
		let (s, t) = self;
		(s, t, E::default())
	}
}

impl<Ix, E> IntoWeightedEdge<E> for (Ix, Ix, E) {
	type NodeId = Ix;

	fn into_weighted_edge(self) -> (Self::NodeId, Self::NodeId, E) {
		self
	}
}

impl<Ix, E: Clone> IntoWeightedEdge<E> for (Ix, Ix, &E) {
	type NodeId = Ix;

	fn into_weighted_edge(self) -> (Self::NodeId, Self::NodeId, E) {
		let (a, b, c) = self;
		(a, b, c.clone())
	}
}

impl<Ix: Copy, E: Default> IntoWeightedEdge<E> for &(Ix, Ix) {
	type NodeId = Ix;

	fn into_weighted_edge(self) -> (Self::NodeId, Self::NodeId, E) {
		let (s, t) = *self;
		(s, t, E::default())
	}
}

impl<Ix: Copy, E: Clone> IntoWeightedEdge<E> for &(Ix, Ix, E) {
	type NodeId = Ix;

	fn into_weighted_edge(self) -> (Self::NodeId, Self::NodeId, E) {
		self.clone()
	}
}
