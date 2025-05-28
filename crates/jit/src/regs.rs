use core::{
	fmt::{Debug, Display, Formatter, Result as FmtResult, Write as _},
	hash::Hash,
	marker::PhantomData,
	mem,
	ops::Range,
};

use cranelift_bitset::ScalarBitSet;
use tap::Conv;

use super::U6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct BinaryOperands<D, S1 = D, S2 = D> {
	pub dst: D,
	pub src1: S1,
	pub src2: S2,
}

impl<D, S1, S2> BinaryOperands<D, S1, S2> {
	pub const fn new(dst: D, src1: S1, src2: S2) -> Self {
		Self { dst, src1, src2 }
	}
}

impl<D: Reg, S1: Reg, S2: Reg> BinaryOperands<D, S1, S2> {
	pub fn to_bits(self) -> u16 {
		let dst = self.dst.to_u8();
		let src1 = self.src1.to_u8();
		let src2 = self.src2.to_u8();

		u16::from(dst) | (u16::from(src1) << 5) | (u16::from(src2) << 10)
	}

	#[must_use]
	pub fn from_bits(bits: u16) -> Option<Self> {
		Some(Self {
			dst: D::new((bits & 0b11111) as u8)?,
			src1: S1::new(((bits >> 5) & 0b11111) as u8)?,
			src2: S2::new(((bits >> 10) & 0b11111) as u8)?,
		})
	}

	#[must_use]
	pub unsafe fn from_bits_unchecked(bits: u16) -> Self {
		unsafe { Self::from_bits(bits).unwrap_unchecked() }
	}
}

impl<D: Reg, S1: Reg> BinaryOperands<D, S1, U6> {
	pub fn to_bits(self) -> u16 {
		let dst = self.dst.to_u8();
		let src1 = self.src1.to_u8();
		let src2 = self.src2.conv::<u8>();

		u16::from(dst) | (u16::from(src1) << 5) | (u16::from(src2) << 10)
	}

	#[must_use]
	pub fn from_bits(bits: u16) -> Option<Self> {
		Some(Self {
			dst: D::new((bits & 0b11111) as u8)?,
			src1: S1::new(((bits >> 5) & 0b11111) as u8)?,
			src2: U6::new(((bits >> 10) & 0b11111) as u8)?,
		})
	}

	#[must_use]
	pub unsafe fn from_bits_unchecked(bits: u16) -> Self {
		unsafe { Self::from_bits(bits).unwrap_unchecked() }
	}
}

#[repr(transparent)]
pub struct UpperRegSet<R> {
	bitset: ScalarBitSet<u16>,
	marker: PhantomData<R>,
}

impl<R: Reg> UpperRegSet<R> {
	#[must_use]
	pub const fn from_bitset(bitset: ScalarBitSet<u16>) -> Self {
		Self {
			bitset,
			marker: PhantomData,
		}
	}

	#[must_use]
	pub const fn into_bitset(self) -> ScalarBitSet<u16> {
		self.bitset
	}
}

#[cfg(feature = "arbitrary")]
impl<'a, R: Reg> arbitrary::Arbitrary<'a> for UpperRegSet<R> {
	fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
		ScalarBitSet::arbitrary(u).map(Self::from)
	}
}

#[allow(clippy::expl_impl_clone_on_copy)]
impl<R: Reg> Clone for UpperRegSet<R> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<R: Reg> Copy for UpperRegSet<R> {}

impl<R: Reg> Debug for UpperRegSet<R> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_set().entries(*self).finish()
	}
}

impl<R: Reg> Default for UpperRegSet<R> {
	fn default() -> Self {
		Self {
			bitset: ScalarBitSet::default(),
			marker: PhantomData,
		}
	}
}

impl<R: Reg> Eq for UpperRegSet<R> {}

impl<R: Reg> From<ScalarBitSet<u16>> for UpperRegSet<R> {
	fn from(value: ScalarBitSet<u16>) -> Self {
		Self::from_bitset(value)
	}
}

impl<R: Reg> From<UpperRegSet<R>> for ScalarBitSet<u16> {
	fn from(value: UpperRegSet<R>) -> Self {
		value.into_bitset()
	}
}

impl<R: Reg> IntoIterator for UpperRegSet<R> {
	type IntoIter = UpperRegSetIntoIter<R>;
	type Item = R;

	fn into_iter(self) -> Self::IntoIter {
		UpperRegSetIntoIter {
			iter: self.bitset.into_iter(),
			marker: PhantomData,
		}
	}
}

impl<R: PartialEq> PartialEq for UpperRegSet<R> {
	fn eq(&self, other: &Self) -> bool {
		self.bitset == other.bitset
	}
}

#[repr(transparent)]
pub struct UpperRegSetIntoIter<R> {
	iter: cranelift_bitset::scalar::Iter<u16>,
	marker: PhantomData<R>,
}

impl<R: Reg> DoubleEndedIterator for UpperRegSetIntoIter<R> {
	fn next_back(&mut self) -> Option<Self::Item> {
		R::new(self.iter.next_back()? + 16)
	}
}

impl<R: Reg> Iterator for UpperRegSetIntoIter<R> {
	type Item = R;

	fn next(&mut self) -> Option<Self::Item> {
		R::new(self.iter.next()? + 16)
	}
}

#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AddrO32 {
	pub addr: XReg,
	pub offset: i32,
}

#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AddrZ {
	pub addr: XReg,
	pub offset: i32,
}

#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AddrG32 {
	pub host_heap_base: XReg,
	pub host_heap_bound: XReg,
	pub wasm_addr: XReg,
	pub offset: u16,
}

impl AddrG32 {
	#[must_use]
	pub fn from_bits(bits: u32) -> Option<Self> {
		let host_heap_base = XReg::new(((bits >> 26) & 0b11111) as u8)?;
		let bound_reg = XReg::new(((bits >> 21) & 0b11111) as u8)?;
		let wasm_addr = XReg::new(((bits >> 16) & 0b11111) as u8)?;

		Some(Self {
			host_heap_base,
			host_heap_bound: bound_reg,
			wasm_addr,
			offset: bits as u16,
		})
	}

	#[must_use]
	pub unsafe fn from_bits_unchecked(bits: u32) -> Self {
		unsafe { Self::from_bits(bits).unwrap_unchecked() }
	}

	#[must_use]
	pub fn to_bits(self) -> u32 {
		u32::from(self.offset)
			| (u32::from(self.wasm_addr.to_u8()) << 16)
			| (u32::from(self.host_heap_bound.to_u8()) << 21)
			| (u32::from(self.host_heap_base.to_u8()) << 26)
	}
}

#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AddrG32Bne {
	pub host_heap_base: XReg,
	pub host_heap_bound_addr: XReg,
	pub host_heap_bound_offset: u8,
	pub wasm_addr: XReg,
	pub offset: u8,
}

impl AddrG32Bne {
	#[must_use]
	pub fn from_bits(bits: u32) -> Option<Self> {
		let host_heap_base = XReg::new(((bits >> 26) & 0b11111) as u8)?;
		let bound_reg = XReg::new(((bits >> 21) & 0b11111) as u8)?;
		let wasm_addr = XReg::new(((bits >> 16) & 0b11111) as u8)?;
		Some(Self {
			host_heap_base,
			host_heap_bound_addr: bound_reg,
			host_heap_bound_offset: (bits >> 8) as u8,
			wasm_addr,
			offset: bits as u8,
		})
	}

	#[must_use]
	pub unsafe fn from_bits_unchecked(bits: u32) -> Self {
		unsafe { Self::from_bits(bits).unwrap_unchecked() }
	}

    #[must_use]
	pub fn to_bits(self) -> u32 {
		u32::from(self.offset)
			| (u32::from(self.host_heap_bound_offset) << 8)
			| (u32::from(self.wasm_addr.to_u8()) << 16)
			| (u32::from(self.host_heap_bound_addr.to_u8()) << 21)
			| (u32::from(self.host_heap_base.to_u8()) << 26)
	}
}

#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum XReg {
	X0,
	X1,
	X2,
	X3,
	X4,
	X5,
	X6,
	X7,
	X8,
	X9,
	X10,
	X11,
	X12,
	X13,
	X14,
	X15,
	X16,
	X17,
	X18,
	X19,
	X20,
	X21,
	X22,
	X23,
	X24,
	X25,
	X26,
	X27,
	X28,
	X29,
	Sp,
	SpillTmp0,
}

impl XReg {
	pub const SPECIAL_START: u8 = Self::Sp as u8;

	#[must_use]
	pub const fn is_special(self) -> bool {
		matches!(self, Self::Sp | Self::SpillTmp0)
	}
}

impl Debug for XReg {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self, f)
	}
}

impl Display for XReg {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		if self.is_special() {
			match self {
				Self::Sp => f.write_str("sp")?,
				Self::SpillTmp0 => f.write_str("spilltmp0")?,
				_ => unreachable!(),
			}
		} else {
			f.write_char('x')?;
			let value = *self as u8;

			Display::fmt(&value, f)?;
		}

		Ok(())
	}
}

impl Reg for XReg {
	const RANGE: Range<u8> = 0..32;

	unsafe fn new_unchecked(index: u8) -> Self {
		unsafe { mem::transmute(index) }
	}

	fn to_u8(self) -> u8 {
		self as u8
	}
}

#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum FReg {
	F0,
	F1,
	F2,
	F3,
	F4,
	F5,
	F6,
	F7,
	F8,
	F9,
	F10,
	F11,
	F12,
	F13,
	F14,
	F15,
	F16,
	F17,
	F18,
	F19,
	F20,
	F21,
	F22,
	F23,
	F24,
	F25,
	F26,
	F27,
	F28,
	F29,
	F30,
	F31,
}

impl Debug for FReg {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self, f)
	}
}

impl Display for FReg {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_char('f')?;
		let value = *self as u8;
		Display::fmt(&value, f)
	}
}

impl Reg for FReg {
	const RANGE: Range<u8> = 0..32;

	unsafe fn new_unchecked(index: u8) -> Self {
		unsafe { mem::transmute(index) }
	}

	fn to_u8(self) -> u8 {
		self as u8
	}
}

#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum VReg {
	V0,
	V1,
	V2,
	V3,
	V4,
	V5,
	V6,
	V7,
	V8,
	V9,
	V10,
	V11,
	V12,
	V13,
	V14,
	V15,
	V16,
	V17,
	V18,
	V19,
	V20,
	V21,
	V22,
	V23,
	V24,
	V25,
	V26,
	V27,
	V28,
	V29,
	V30,
	V31,
}

impl Debug for VReg {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self, f)
	}
}

impl Display for VReg {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_char('v')?;
		let value = *self as u8;
		Display::fmt(&value, f)
	}
}

impl Reg for VReg {
	const RANGE: Range<u8> = 0..32;

	unsafe fn new_unchecked(index: u8) -> Self {
		unsafe { mem::transmute(index) }
	}

	fn to_u8(self) -> u8 {
		self as u8
	}
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum AnyReg {
	X(XReg),
	F(FReg),
	V(VReg),
}

impl Debug for AnyReg {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self, f)
	}
}

impl Display for AnyReg {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match *self {
			Self::F(r) => Display::fmt(&r, f),
			Self::X(r) => Display::fmt(&r, f),
			Self::V(r) => Display::fmt(&r, f),
		}
	}
}

impl From<FReg> for AnyReg {
	fn from(value: FReg) -> Self {
		Self::F(value)
	}
}

impl From<XReg> for AnyReg {
	fn from(value: XReg) -> Self {
		Self::X(value)
	}
}

impl From<VReg> for AnyReg {
	fn from(value: VReg) -> Self {
		Self::V(value)
	}
}

pub trait Reg: Copy + Debug + Display + Eq + Hash + Into<AnyReg> + Ord + Sized {
	const RANGE: Range<u8>;

	unsafe fn new_unchecked(index: u8) -> Self;

	#[must_use]
	fn new(index: u8) -> Option<Self> {
		if Self::RANGE.contains(&index) {
			Some(unsafe { Self::new_unchecked(index) })
		} else {
			None
		}
	}

	fn to_u8(self) -> u8;

	fn index(self) -> usize {
		self.to_u8().into()
	}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_operands() {
        let mut i = 0;
        for src2 in XReg::RANGE {
            for src1 in XReg::RANGE {
                for dst in XReg::RANGE {
                    let operands = BinaryOperands {
                        dst: XReg::new(dst).unwrap(),
                        src1: XReg::new(src1).unwrap(),
                        src2: XReg::new(src2).unwrap()
                    };

                    assert_eq!(operands.to_bits(), i);
                    assert_eq!(BinaryOperands::<XReg>::from_bits(i).unwrap(), operands);
                    assert_eq!(BinaryOperands::<XReg>::from_bits(0x8000 | i).unwrap(), operands);
                    i += 1;
                }
            }
        }
    }
}
