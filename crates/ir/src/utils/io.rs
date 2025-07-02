pub trait HasIo: super::sealed::Sealed {
	fn has_read(&self) -> bool;

	fn has_write(&self) -> bool;

	fn has_io(&self) -> bool {
		self.has_read() || self.has_write()
	}
}

impl<T: HasIo> HasIo for [T] {
	fn has_read(&self) -> bool {
		self.iter().any(HasIo::has_read)
	}

	fn has_write(&self) -> bool {
		self.iter().any(HasIo::has_write)
	}

	fn has_io(&self) -> bool {
		self.iter().any(HasIo::has_io)
	}
}
