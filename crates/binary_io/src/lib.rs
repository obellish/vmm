#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod impls;

#[cfg(not(feature = "std"))]
pub use self::impls::*;

pub trait Read {
	type Error;

	fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Self::Error>;
}

pub trait Write {
	type Error;

	fn write_all(&mut self, data: &[u8]) -> Result<(), Self::Error>;

	fn flush(&mut self) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn read_eof() {
		let mut reader = &[1u8; 0][..];
		let mut buffer = [0u8; 1];

		assert!(reader.read_exact(&mut buffer[..]).is_err());
	}

	#[test]
	fn read_one<'a>() -> Result<(), <&'a [u8] as crate::Read>::Error> {
		let mut reader = &[1u8; 1][..];
		let mut buffer = [0u8; 1];

		reader.read_exact(&mut buffer[..])?;
		assert_eq!(buffer, [1]);

		assert!(reader.read_exact(&mut buffer[..]).is_err());

		Ok(())
	}

	#[test]
	fn read_two<'a>() -> Result<(), <&'a [u8] as crate::Read>::Error> {
		let mut reader = &[1u8; 2][..];
		let mut buffer = [0u8; 1];

		reader.read_exact(&mut buffer[..])?;
		assert_eq!(buffer[0], 1);

		reader.read_exact(&mut buffer[..])?;
		assert_eq!(buffer[0], 1);

		assert!(reader.read_exact(&mut buffer).is_err());

		Ok(())
	}

	#[test]
	#[cfg(feature = "std")]
	fn read_std() -> std::io::Result<()> {
		let mut reader = std::io::repeat(1);
		let mut buffer = [0u8; 2];

		reader.read_exact(&mut buffer[..])?;
		assert_eq!(buffer, [1; 2]);

		Ok(())
	}

	#[test]
	fn write_oos() {
		let mut writer = &mut [0u8; 0][..];

		assert!(writer.write_all(&[1u8; 1][..]).is_err());
	}

	#[test]
	fn write_one<'a>() -> Result<(), <&'a mut [u8] as crate::Write>::Error> {
		let mut buffer = [0u8; 1];
		let mut writer = &mut buffer[..];

		writer.write_all(&[1u8; 1][..])?;

		assert!(writer.write_all(&[1u8; 1][..]).is_err());
		assert_eq!(buffer, [1]);

		Ok(())
	}

	#[test]
	fn write_two<'a>() -> Result<(), <&'a mut [u8] as crate::Write>::Error> {
		let mut buffer = [0u8; 2];
		let mut writer = &mut buffer[..];

		writer.write_all(&[1u8; 1][..])?;
		writer.write_all(&[1u8; 1][..])?;
		assert!(writer.write_all(&[1u8; 1][..]).is_err());

		assert_eq!(buffer, [1; 2]);

		Ok(())
	}

	#[test]
	#[cfg(feature = "alloc")]
	fn write_vec() -> Result<(), <alloc::vec::Vec<u8> as crate::Write>::Error> {
		let mut buffer = alloc::vec::Vec::new();

		buffer.write_all(&[1u8; 1][..])?;
		buffer.write_all(&[1u8; 1][..])?;

		assert_eq!(buffer.len(), 2);
		assert_eq!(buffer, [1; 2]);

		Ok(())
	}

	#[test]
	#[cfg(feature = "std")]
	fn write_std() -> std::io::Result<()> {
		let mut writer =std::io::sink();

		writer.write_all(&[1u8; 1][..])?;
		writer.write_all(&[1u8; 1][..])?;

		Ok(())
	}
}
