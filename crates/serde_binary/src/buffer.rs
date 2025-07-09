use super::{Error, Result};

pub trait Buffer {
	fn clear(&mut self);

	fn as_slice(&self) -> &[u8];

	fn push(&mut self, byte: u8) -> Result<()>;

	fn reserve_slice(&mut self, len: usize) -> Result<&mut [u8]>;

	fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<()> {
		bytes.iter().try_for_each(|b| self.push(*b))
	}
}

impl Buffer for () {
	fn clear(&mut self) {}

	fn as_slice(&self) -> &[u8] {
		&[]
	}

	fn push(&mut self, _: u8) -> Result<()> {
		Err(Error::BufferTooSmall)
	}

	fn reserve_slice(&mut self, len: usize) -> Result<&mut [u8]> {
		if matches!(len, 0) {
			Ok(&mut [])
		} else {
			Err(Error::BufferTooSmall)
		}
	}
}

#[cfg(feature = "alloc")]
impl Buffer for alloc::vec::Vec<u8> {
	fn clear(&mut self) {
		self.clear();
	}

	fn as_slice(&self) -> &[u8] {
		self
	}

	fn push(&mut self, byte: u8) -> Result<()> {
		self.try_reserve(1)?;
		self.push(byte);
		Ok(())
	}

	fn reserve_slice(&mut self, len: usize) -> Result<&mut [u8]> {
		self.try_reserve(len)?;
		let prev = self.len();
		self.resize(prev.checked_add(len).ok_or(Error::Overflow)?, 0);
		Ok(self.as_mut_slice().split_at_mut(prev).1)
	}

	fn extend_from_slice(&mut self, bytes: &[u8]) -> Result<()> {
		self.try_reserve(bytes.len())?;
		self.extend_from_slice(bytes);
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn does_not_panic(mut buffer: impl Buffer) {
		buffer.clear();
		_ = buffer.as_slice();
		_ = buffer.push(0);
		_ = buffer.extend_from_slice(&[]);
		_ = buffer.extend_from_slice(&[5]);
		_ = buffer.extend_from_slice(&[1, 2, 3, 4, 5]);
		_ = buffer.reserve_slice(0);
		_ = buffer.reserve_slice(1);
		_ = buffer.reserve_slice(usize::MAX / 2);
		_ = buffer.reserve_slice(usize::MAX);
	}

	#[cfg(feature = "alloc")]
	fn basic_buffer_works(mut buffer: impl Buffer) -> Result<()> {
		buffer.clear();
		assert_eq!(buffer.as_slice(), [0u8; 0]);
		buffer.push(1)?;
		assert_eq!(buffer.as_slice(), [1]);
		buffer.push(2)?;
		assert_eq!(buffer.as_slice(), [1, 2]);
		buffer.extend_from_slice(&[3, 4])?;
		assert_eq!(buffer.as_slice(), [1, 2, 3, 4]);
		buffer.push(5)?;
		assert_eq!(buffer.as_slice(), [1, 2, 3, 4, 5]);
		buffer.clear();
		assert_eq!(buffer.as_slice(), [0u8; 0]);
		buffer.extend_from_slice(&[])?;
		assert_eq!(buffer.as_slice(), [0u8; 0]);
		buffer.extend_from_slice(&[1])?;
		assert_eq!(buffer.as_slice(), [1]);
		buffer.extend_from_slice(&[2, 3, 4, 5])?;
		assert_eq!(buffer.as_slice(), [1, 2, 3, 4, 5]);

		Ok(())
	}

	#[cfg(feature = "alloc")]
	fn reserve_slice_works(mut buffer: impl Buffer) -> Result<()> {
		buffer.clear();
		let slice = buffer.reserve_slice(0)?;
		assert_eq!(slice, [0u8; 0]);
		let slice = buffer.reserve_slice(10)?;
		assert_eq!(slice.len(), 10);
		assert_eq!(slice, [0; 10]);
		for (i, target) in slice.iter_mut().enumerate() {
			*target = i as u8;
		}

		assert_eq!(slice, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
		let slice = buffer.as_slice();
		assert_eq!(slice, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

		let slice = buffer.reserve_slice(1)?;
		slice[0] = 10;
		assert_eq!(slice, [10]);
		let slice = buffer.as_slice();
		assert_eq!(slice, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

		Ok(())
	}

	#[test]
	fn unit() {
		does_not_panic(());
	}

	#[cfg(feature = "alloc")]
	#[test]
	fn vec() -> Result<()> {
		does_not_panic(alloc::vec::Vec::new());
		basic_buffer_works(alloc::vec::Vec::new())?;
		reserve_slice_works(alloc::vec::Vec::new())
	}
}
