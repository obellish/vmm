use std::fmt::{Debug, Display, Formatter, Result as FmtResult, Write as _};

use super::{Decode, Encode};

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub struct Velocity(pub [i16; 3]);

impl Velocity {
	pub fn from_ms_f32(ms: [f32; 3]) -> Self {
		Self(ms.map(|v| (8000.0 / 20.0 * v) as i16))
	}

	pub fn from_ms_f64(ms: [f64; 3]) -> Self {
		Self(ms.map(|v| (8000.0 / 20.0 * v) as i16))
	}

	pub fn to_ms_f32(self) -> [f32; 3] {
		self.0.map(|v| f32::from(v) / (8000.0 / 20.0))
	}

	pub fn to_ms_f64(self) -> [f64; 3] {
		self.0.map(|v| f64::from(v) / (8000.0 / 20.0))
	}
}

impl Debug for Velocity {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self, f)
	}
}

impl Display for Velocity {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let [x, y, z] = self.to_ms_f32();
		f.write_char('⟨')?;
		Display::fmt(&x, f)?;
		f.write_char(',')?;
		Display::fmt(&y, f)?;
		f.write_char(',')?;
		Display::fmt(&z, f)?;
		f.write_str("⟩ m/s")
	}
}

impl From<[i16; 3]> for Velocity {
	fn from(value: [i16; 3]) -> Self {
		Self(value)
	}
}

impl From<Velocity> for [i16; 3] {
	fn from(value: Velocity) -> Self {
		value.0
	}
}

#[cfg(test)]
mod tests {
	use super::Velocity;

	#[test]
	fn velocity_from_ms() {
		let val_1 = Velocity::from_ms_f32(std::array::from_fn(|_| -3.3575)).0[0];
		let val_2 = Velocity::from_ms_f64(std::array::from_fn(|_| -3.3575)).0[0];

		assert_eq!(val_1, val_2);
		assert_eq!(val_1, -1343);
	}
}
