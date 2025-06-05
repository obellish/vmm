use std::hash::Hash;

pub trait Sealed {}

impl Sealed for () {}
impl<T> Sealed for crate::tree::Ord<T> where T: ?Sized + Ord {}
impl<T> Sealed for crate::hash::Hash<T> where T: ?Sized + Eq + Hash {}
