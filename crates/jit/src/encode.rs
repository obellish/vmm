use core::iter;

use super::{imms::*, regs::*};

pub trait Encode {
	const WIDTH: u8;

	fn encode<E>(&self, sink: &mut E)
	where
		E: Extend<u8>;
}

impl Encode for u8 {
	const WIDTH: Self = 1;

	fn encode<E>(&self, sink: &mut E)
	where
		E: Extend<Self>,
	{
		sink.extend(iter::once(*self));
	}
}

impl Encode for u16 {
	const WIDTH: u8 = 2;

	fn encode<E>(&self, sink: &mut E)
	where
		E: Extend<u8>,
	{
		sink.extend(self.to_le_bytes());
	}
}

impl Encode for u32 {
	const WIDTH: u8 = 4;

	fn encode<E>(&self, sink: &mut E)
	where
		E: Extend<u8>,
	{
		sink.extend(self.to_le_bytes());
	}
}

impl Encode for u64 {
	const WIDTH: u8 = 8;

	fn encode<E>(&self, sink: &mut E)
	where
		E: Extend<u8>,
	{
		sink.extend(self.to_le_bytes());
	}
}

impl Encode for u128 {
	const WIDTH: u8 = 16;

	fn encode<E>(&self, sink: &mut E)
	where
		E: Extend<u8>,
	{
		sink.extend(self.to_le_bytes());
	}
}

impl Encode for i8 {
	const WIDTH: u8 = 1;

	fn encode<E>(&self, sink: &mut E)
	where
		E: Extend<u8>,
	{
		sink.extend(iter::once(*self as u8));
	}
}

impl Encode for i16 {
	const WIDTH: u8 = 2;

	fn encode<E>(&self, sink: &mut E)
	where
		E: Extend<u8>,
	{
		sink.extend(self.to_le_bytes());
	}
}

impl Encode for i32 {
	const WIDTH: u8 = 4;

	fn encode<E>(&self, sink: &mut E)
	where
		E: Extend<u8>,
	{
		sink.extend(self.to_le_bytes());
	}
}

impl Encode for i64 {
	const WIDTH: u8 = 8;

	fn encode<E>(&self, sink: &mut E)
	where
		E: Extend<u8>,
	{
		sink.extend(self.to_le_bytes());
	}
}

impl Encode for i128 {
	const WIDTH: u8 = 16;

	fn encode<E>(&self, sink: &mut E)
	where
		E: Extend<u8>,
	{
		sink.extend(self.to_le_bytes());
	}
}

impl Encode for XReg {
	const WIDTH: u8 = 1;

	fn encode<E>(&self, sink: &mut E)
	where
		E: Extend<u8>,
	{
		sink.extend(iter::once(self.to_u8()));
	}
}

impl Encode for FReg {
	const WIDTH: u8 = 1;

	fn encode<E>(&self, sink: &mut E)
	where
		E: Extend<u8>,
	{
		sink.extend(iter::once(self.to_u8()));
	}
}

impl Encode for VReg {
	const WIDTH: u8 = 1;

	fn encode<E>(&self, sink: &mut E)
	where
		E: Extend<u8>,
	{
		sink.extend(iter::once(self.to_u8()));
	}
}

impl Encode for PcRelOffset {
	const WIDTH: u8 = 4;

	fn encode<E>(&self, sink: &mut E)
	where
		E: Extend<u8>,
	{
		i32::from(*self).encode(sink);
	}
}
