use super::BASIC_DATA as BASIC_OUTPUT_DATA;
use crate::io::*;

fn output_does_not_panic(mut output: impl Output) {
	_ = output.write_byte(0);
	_ = output.write_all(&[]);
	_ = output.write_all(&[1]);
	_ = output.write_all(&[1, 2, 3, 4, 5]);
}

fn basic_output_works(output: &mut impl Output) -> Result<()> {
	output.write_byte(0)?;
	output.write_byte(1)?;
	output.write_all(&[])?;
	output.write_all(&[2, 3, 4, 5])?;
	output.write_byte(6)?;
	output.write_all(&[7, 8, 9])
}

#[test]
fn slice() -> Result<()> {
	output_does_not_panic([1, 2].as_mut_slice());
	let mut buffer = [0; 10];
	let mut output = buffer.as_mut_slice();
	basic_output_works(&mut output)?;
	let expected: &mut [u8] = &mut [];
	assert_eq!(output, expected);
	assert_eq!(buffer, BASIC_OUTPUT_DATA);

	Ok(())
}

#[cfg(feature = "alloc")]
#[test]
fn vec() -> Result<()> {
	output_does_not_panic(alloc::vec::Vec::new());
	let mut output = alloc::vec::Vec::new();
	basic_output_works(&mut output)?;
	assert_eq!(output, BASIC_OUTPUT_DATA);

	Ok(())
}

#[cfg(feature = "std")]
#[test]
fn writer() -> Result<()> {
	output_does_not_panic(IoWriter::new(std::vec::Vec::new()));
	let mut output = IoWriter::new(std::vec::Vec::new());
	basic_output_works(&mut output)?;
	assert_eq!(output.writer, BASIC_OUTPUT_DATA);

	Ok(())
}
