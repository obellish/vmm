use core::{
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	ops::{Deref, DerefMut},
};

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
	any(
		target_arch = "x86_64",
		target_arch = "aarch64",
		target_arch = "arm64ec",
		target_arch = "powerpc64",
	),
	repr(align(128))
)]
#[cfg_attr(
	any(
		target_arch = "arm",
		target_arch = "mips",
		target_arch = "mips32r6",
		target_arch = "mips64",
		target_arch = "mips64r6",
		target_arch = "sparc",
		target_arch = "hexagon",
	),
	repr(align(32))
)]
#[cfg_attr(target_arch = "m68k", repr(align(16)))]
#[cfg_attr(target_arch = "s390x", repr(align(256)))]
#[cfg_attr(
	not(any(
		target_arch = "x86_64",
		target_arch = "aarch64",
		target_arch = "arm64ec",
		target_arch = "powerpc64",
		target_arch = "arm",
		target_arch = "mips",
		target_arch = "mips32r6",
		target_arch = "mips64",
		target_arch = "mips64r6",
		target_arch = "sparc",
		target_arch = "hexagon",
		target_arch = "m68k",
		target_arch = "s390x",
	)),
	repr(align(64))
)]
pub struct CachePadded<T> {
	value: T,
}

impl<T> CachePadded<T> {
	pub const fn new(value: T) -> Self {
		Self { value }
	}

	pub fn into_inner(self) -> T {
		self.value
	}
}

impl<T: Debug> Debug for CachePadded<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("CachePadded")
			.field("value", &self.value)
			.finish()
	}
}

impl<T> Deref for CachePadded<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl<T> DerefMut for CachePadded<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.value
	}
}

impl<T: Display> Display for CachePadded<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.value, f)
	}
}

impl<T> From<T> for CachePadded<T> {
	fn from(value: T) -> Self {
		Self::new(value)
	}
}

unsafe impl<T: Send> Send for CachePadded<T> {}
unsafe impl<T: Sync> Sync for CachePadded<T> {}
