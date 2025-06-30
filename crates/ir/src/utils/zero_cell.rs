pub trait IsZeroingCell: super::sealed::Sealed {
	fn is_zeroing_cell(&self) -> bool;
}
