use std::fmt::{Debug, Formatter, Result as FmtResult};

#[repr(transparent)]
pub struct IfRunner<P>(pub P);

impl<P: Debug> Debug for IfRunner<P> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.0, f)
	}
}
