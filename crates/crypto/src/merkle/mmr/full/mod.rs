pub(crate) const fn high_bitmask(bit: u32) -> usize {
	if bit > usize::BITS - 1 {
		0
	} else {
		usize::MAX << bit
	}
}
