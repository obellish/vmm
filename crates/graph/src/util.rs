use core::iter;

pub fn enumerate<I: IntoIterator>(iterable: I) -> iter::Enumerate<I::IntoIter> {
	iterable.into_iter().enumerate()
}

pub fn zip<I: IntoIterator, J: IntoIterator>(i: I, j: J) -> iter::Zip<I::IntoIter, J::IntoIter> {
	i.into_iter().zip(j)
}
