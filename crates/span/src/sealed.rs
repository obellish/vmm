pub trait Sealed {}

impl<T> Sealed for super::Excluded<T> {}
impl<T> Sealed for super::Included<T> {}
impl<T: ?Sized> Sealed for super::Unbounded<T> {}
