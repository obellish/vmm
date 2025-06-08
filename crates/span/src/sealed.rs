pub trait Sealed {}

impl<T: ?Sized> Sealed for super::Excluded<T> {}
impl<T: ?Sized> Sealed for super::Included<T> {}
impl<T: ?Sized> Sealed for super::Unbounded<T> {}
