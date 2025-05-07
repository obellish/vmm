use crate::{TAPE_SIZE, Tape, TapePointer};

#[test]
fn wrapping_pointer() {
	let mut ptr = TapePointer::new(108).unwrap();

	ptr += 10usize;

	assert_eq!(ptr.value(), 118);

	ptr += TAPE_SIZE - 4;

	assert_eq!(ptr.value(), 114);
}
