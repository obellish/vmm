use alloc::string::String;
use core::fmt::{Display, Formatter, LowerHex, Result as FmtResult};

#[repr(transparent)]
pub struct DisplayHex<'a>(pub &'a [u8]);

impl<'a> DisplayHex<'a> {
	pub fn new<'b: 'a, T>(item: &'b T) -> Self
	where
		T: AsRef<[u8]>,
	{
		Self(item.as_ref())
	}
}

impl Display for DisplayHex<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		LowerHex::fmt(self, f)
	}
}

impl LowerHex for DisplayHex<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		if f.alternate() {
			f.write_str("0x")?;
		}

		for byte in self.0 {
			write!(f, "{byte:02x}")?;
		}

		Ok(())
	}
}

impl ToHex for DisplayHex<'_> {
	fn to_hex(&self) -> String {
		format!("{self:x}")
	}

	fn to_hex_with_prefix(&self) -> String {
		format!("{self:#x}")
	}
}

impl crate::prettier::PrettyPrint for DisplayHex<'_> {
	fn render(&self) -> crate::prettier::Document {
		crate::prettier::text(format!("{self:#x}"))
	}
}

pub trait ToHex {
	fn to_hex(&self) -> String;

	fn to_hex_with_prefix(&self) -> String;
}

impl ToHex for [u8] {
	fn to_hex(&self) -> String {
		DisplayHex(self).to_hex()
	}

	fn to_hex_with_prefix(&self) -> String {
		DisplayHex(self).to_hex_with_prefix()
	}
}
