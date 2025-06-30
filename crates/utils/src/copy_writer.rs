use std::{
	borrow::ToOwned,
	io::{Result as IoResult, prelude::*},
};

pub struct CopyWriter<T, U> {
	first: T,
	second: U,
}

impl<T, U> CopyWriter<T, U> {
	pub const fn new(first: T, second: U) -> Self {
		Self { first, second }
	}

	pub fn into_inner(self) -> (T, U) {
		(self.first, self.second)
	}

	pub const fn as_ref(&self) -> CopyWriter<&T, &U> {
		CopyWriter {
			first: &self.first,
			second: &self.second,
		}
	}
}

impl<T: Clone, U: Clone> Clone for CopyWriter<T, U> {
	fn clone(&self) -> Self {
		Self {
			first: self.first.clone(),
			second: self.second.clone(),
		}
	}
}

impl<T: Write, U: Write> Write for CopyWriter<T, U> {
	fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
		let buf = buf.to_owned();

		let first_write_count = self.first.write(&buf.clone())?;

		let second_write_count = self.second.write(&buf)?;

		assert_eq!(first_write_count, second_write_count);
		Ok(first_write_count)
	}

	fn flush(&mut self) -> IoResult<()> {
		self.first.flush()?;
		self.second.flush()?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use std::{
		io::{IoSlice, Result as IoResult, prelude::*},
		vec::Vec as StdVec,
	};

	use super::CopyWriter;

	type Vec = StdVec<u8>;

	struct TestWriter {
		n_bufs: usize,
		per_call: usize,
		written: Vec,
	}

	impl TestWriter {
		const fn new(n_bufs: usize, per_call: usize) -> Self {
			Self {
				n_bufs,
				per_call,
				written: Vec::new(),
			}
		}
	}

	impl Write for TestWriter {
		fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
			self.write_vectored(&[IoSlice::new(buf)])
		}

		fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> IoResult<usize> {
			let mut left = self.per_call;
			let mut written = 0;
			for buf in bufs.iter().take(self.n_bufs) {
				let n = std::cmp::min(left, buf.len());
				self.written.extend_from_slice(&buf[0..n]);
				left -= n;
				written += n;
			}

			Ok(written)
		}

		fn flush(&mut self) -> IoResult<()> {
			Ok(())
		}
	}

	#[test]
	fn double_write_works() -> IoResult<()> {
		let mut first = Vec::new();
		let mut second = Vec::new();

		let bytes = {
			let mut writer = CopyWriter::new(&mut first, &mut second);

			writer.write(&[0, 1, 2, 3, 4, 5])?
		};

		assert_eq!(first.len(), bytes);
		assert_eq!(second.len(), bytes);

		assert_eq!(first, [0, 1, 2, 3, 4, 5]);
		assert_eq!(second, [0, 1, 2, 3, 4, 5]);

		Ok(())
	}

	#[test]
	fn writer_read_from_one_buf() -> IoResult<()> {
		let mut first = TestWriter::new(1, 2);
		let mut second = TestWriter::new(1, 2);

		{
			let mut writer = CopyWriter::new(&mut first, &mut second);

			assert_eq!(writer.write(&[])?, 0);
			assert_eq!(writer.write_vectored(&[])?, 0);
		}

		{
			let mut writer = CopyWriter::new(&mut first, &mut second);

			assert_eq!(writer.write(&[1; 3])?, 2);
			let bufs = &[IoSlice::new(&[2; 3])];
			assert_eq!(writer.write_vectored(bufs)?, 2);
		}

		{
			let mut writer = CopyWriter::new(&mut first, &mut second);
			let bufs = &[IoSlice::new(&[3]), IoSlice::new(&[4; 2])];
			assert_eq!(writer.write_vectored(bufs)?, 1);
		}

		assert_eq!(first.written, [1, 1, 2, 2, 3]);
		assert_eq!(first.written, second.written);

		Ok(())
	}
}
