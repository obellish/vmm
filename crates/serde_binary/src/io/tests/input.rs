use super::BASIC_DATA as BASIC_INPUT_DATA;
use crate::io::*;

const PANIC_INPUT_DATA: &[u8] = &[0];

const READ_BYTES_INPUT_DATA: &[u8] = &[5; 20];

fn input_does_not_panic<'de>(mut input: impl Input<'de>) {
	_ = input.peek_byte();
	_ = input.read_byte();
	_ = input.read_exact(&mut [0, 1, 2, 3, 4]);
	_ = input.read_bytes::<()>(10, None);
	_ = input.read_bytes(10, Some(&mut ()));
	_ = input.read_bytes::<()>(usize::MAX / 2, None);
	_ = input.read_bytes::<()>(usize::MAX, None);
	_ = input.skip_bytes(10);
	_ = input.skip_bytes(usize::MAX / 2);
	_ = input.skip_bytes(usize::MAX);
}

fn basic_input_works<'de>(mut input: impl Input<'de>) -> Result<()> {
	let byte = input.peek_byte()?;
	assert_eq!(byte, 0);
	_ = input.peek_byte()?;
	let byte = input.peek_byte()?;
	assert_eq!(byte, 0);

	let byte = input.read_byte()?;
	assert_eq!(byte, 0);
	let byte = input.peek_byte()?;
	assert_eq!(byte, 1);
	_ = input.read_byte()?;
	let byte = input.read_byte()?;
	assert_eq!(byte, 2);

	let mut target = [0; 0];
	input.read_exact(&mut target)?;
	let byte = input.peek_byte()?;
	assert_eq!(byte, 3);
	let mut target = [0; 2];
	input.read_exact(&mut target)?;
	assert_eq!(target, [3, 4]);
	let byte = input.peek_byte()?;
	assert_eq!(byte, 5);

	input.skip_bytes(0)?;
	let byte = input.peek_byte()?;
	assert_eq!(byte, 5);
	input.skip_bytes(1)?;
	input.skip_bytes(2)?;
	let byte = input.peek_byte()?;
	assert_eq!(byte, 8);

	input.skip_bytes(2)?;

	assert!(input.peek_byte().is_err());
	assert!(input.read_byte().is_err());
	assert!(input.read_exact(&mut [0]).is_err());
	assert!(input.skip_bytes(1).is_err());

	Ok(())
}

fn read_bytes_works<'de, B: Buffer>(
	mut input: impl Input<'de>,
	mut buffer: Option<B>,
) -> Result<()> {
	if let Some(b) = buffer.as_mut() {
		b.clear();
	}

	let borrowed = input.read_bytes(10, buffer.as_mut())?;
	let slice = borrowed.unwrap_or_else(|| buffer.as_ref().map_or(&[], |b| b.as_slice()));
	assert_eq!(slice.len(), 10);
	assert_eq!(slice, [5; 10]);

	if let Some(b) = buffer.as_mut() {
		b.clear();
	}

	let borrowed = input.read_bytes(5, buffer.as_mut())?;
	let slice = borrowed.unwrap_or_else(|| buffer.as_ref().map_or(&[], |b| b.as_slice()));
	assert_eq!(slice.len(), 5);
	assert_eq!(slice, [5; 5]);

	if let Some(b) = buffer.as_mut() {
		b.clear();
	}

	assert!(input.read_bytes(10, buffer.as_mut()).is_err());

	Ok(())
}

#[test]
fn slice() -> Result<()> {
	input_does_not_panic(PANIC_INPUT_DATA);
	basic_input_works(BASIC_INPUT_DATA)?;
	read_bytes_works(READ_BYTES_INPUT_DATA, None::<()>)?;

	let mut input = READ_BYTES_INPUT_DATA;
	let mut buffer = None::<()>;
	let borrowed = input.read_bytes(10, buffer.as_mut())?;
	assert!(borrowed.is_some());

	Ok(())
}

#[cfg(feature = "std")]
#[test]
fn reader() -> Result<()> {
	input_does_not_panic(IoReader::new(PANIC_INPUT_DATA));
	basic_input_works(IoReader::new(BASIC_INPUT_DATA))?;
	read_bytes_works(
		IoReader::new(READ_BYTES_INPUT_DATA),
		Some(std::vec::Vec::new()),
	)?;

	Ok(())
}
