use core::{
	cell::RefCell,
	fmt::{Debug, Formatter, Result as FmtResult},
};

#[repr(transparent)]
pub struct DebugMap<F>(pub F);

impl<F, I, K: Debug, V: Debug> Debug for DebugMap<F>
where
	F: Fn() -> I,
	I: IntoIterator<Item = (K, V)>,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_map().entries((self.0)()).finish()
	}
}

#[repr(transparent)]
pub struct NoPretty<T>(pub T);

impl<T: Debug> Debug for NoPretty<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		write!(f, "{:?}", self.0)
	}
}

#[derive(Clone)]
pub struct Format<'a, I> {
	sep: &'a str,
	inner: RefCell<Option<I>>,
}

impl<I: Iterator> Format<'_, I> {
	fn format(
		&self,
		f: &mut Formatter<'_>,
		mut cb: impl FnMut(&I::Item, &mut Formatter<'_>) -> FmtResult,
	) -> FmtResult {
		let Some(mut iter) = self.inner.borrow_mut().take() else {
			panic!("format: was already formatted once")
		};

		if let Some(fst) = iter.next() {
			cb(&fst, f)?;
			for elt in iter {
				if !self.sep.is_empty() {
					f.write_str(self.sep)?;
				}

				cb(&elt, f)?;
			}
		}

		Ok(())
	}
}

impl<I: Iterator> Debug for Format<'_, I>
where
	I::Item: Debug,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.format(f, Debug::fmt)
	}
}

pub trait IterFormatExt: Iterator {
	fn format(self, separator: &str) -> Format<'_, Self>
	where
		Self: Sized,
	{
		Format {
			sep: separator,
			inner: RefCell::new(Some(self)),
		}
	}
}

impl<I: Iterator> IterFormatExt for I {}
