#[cfg_attr(not(feature = "stable_graph"), expect(dead_code))]
pub trait IterUtilsExt: Iterator {
	fn ex_find_map<R>(&mut self, mut f: impl FnMut(Self::Item) -> Option<R>) -> Option<R> {
		for elt in self {
			if let result @ Some(_) = f(elt) {
				return result;
			}
		}

		None
	}

	fn ex_rfind_map<R>(&mut self, mut f: impl FnMut(Self::Item) -> Option<R>) -> Option<R>
	where
		Self: DoubleEndedIterator,
	{
		while let Some(elt) = self.next_back() {
			if let result @ Some(_) = f(elt) {
				return result;
			}
		}

		None
	}
}

impl<I: Iterator> IterUtilsExt for I {}
