#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod decode;
mod encode;
mod header;
mod segments;
pub mod simple;
pub mod tag;

use half::f16;

pub use self::{
	decode::*,
	encode::*,
	header::*,
	segments::{Segment, Segments},
};

#[derive(Debug)]
struct InvalidError(());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Title(pub Major, pub Minor);

impl From<Header> for Title {
	fn from(value: Header) -> Self {
		let int = |i: u64| match i {
			x if x <= 23 => Minor::This(i as u8),
			x if u8::try_from(x).is_ok() => Minor::Next1([i as u8]),
			x if u16::try_from(x).is_ok() => Minor::Next2((i as u16).to_be_bytes()),
			x if u32::try_from(x).is_ok() => Minor::Next4((i as u32).to_be_bytes()),
			x => Minor::Next8(x.to_be_bytes()),
		};

		let len = |l: Option<usize>| l.map_or(Minor::More, |x| int(x as u64));

		match value {
			Header::Positive(x) => Self(Major::Positive, int(x)),
			Header::Negative(x) => Self(Major::Negative, int(x)),
			Header::Bytes(x) => Self(Major::Bytes, len(x)),
			Header::Text(x) => Self(Major::Text, len(x)),
			Header::Array(x) => Self(Major::Array, len(x)),
			Header::Map(x) => Self(Major::Map, len(x)),
			Header::Tag(x) => Self(Major::Tag, int(x)),
			Header::Break => Self(Major::Other, Minor::More),
			Header::Simple(x) => match x {
				x @ 0..=23 => Self(Major::Other, Minor::This(x)),
				x => Self(Major::Other, Minor::Next1([x])),
			},
			Header::Float(n64) => {
				let n16 = f16::from_f64(n64);
				let n32 = n64 as f32;

				Self(
					Major::Other,
					if f64::from(n16).to_bits() == n64.to_bits() {
						Minor::Next2(n16.to_be_bytes())
					} else if f64::from(n32).to_bits() == n64.to_bits() {
						Minor::Next4(n32.to_be_bytes())
					} else {
						Minor::Next8(n64.to_be_bytes())
					},
				)
			}
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Major {
	Positive,
	Negative,
	Bytes,
	Text,
	Array,
	Map,
	Tag,
	Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Minor {
	This(u8),
	Next1([u8; 1]),
	Next2([u8; 2]),
	Next4([u8; 4]),
	Next8([u8; 8]),
	More,
}

impl AsMut<[u8]> for Minor {
	fn as_mut(&mut self) -> &mut [u8] {
		match self {
			Self::This(..) | Self::More => &mut [],
			Self::Next1(x) => x.as_mut(),
			Self::Next2(x) => x.as_mut(),
			Self::Next4(x) => x.as_mut(),
			Self::Next8(x) => x.as_mut(),
		}
	}
}

impl AsRef<[u8]> for Minor {
	fn as_ref(&self) -> &[u8] {
		match self {
			Self::This(..) | Self::More => &[],
			Self::Next1(x) => x.as_ref(),
			Self::Next2(x) => x.as_ref(),
			Self::Next4(x) => x.as_ref(),
			Self::Next8(x) => x.as_ref(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	macro_rules! neg {
		($i:expr) => {
			$crate::Header::Negative((($i as i128) ^ !0) as u64)
		};
	}

	#[test]
	fn leaf() {
		let data = &[
			(Header::Positive(0), "00", true),
			(Header::Positive(1), "01", true),
			(Header::Positive(10), "0a", true),
			(Header::Positive(23), "17", true),
			(Header::Positive(24), "1818", true),
			(Header::Positive(25), "1819", true),
			(Header::Positive(100), "1864", true),
			(Header::Positive(1000), "1903e8", true),
			(Header::Positive(1_000_000), "1a000f4240", true),
			(
				Header::Positive(1_000_000_000_000),
				"1b000000e8d4a51000",
				true,
			),
			(
				Header::Positive(18_446_744_073_709_551_615),
				"1bffffffffffffffff",
				true,
			),
			(
				neg!(-18_446_744_073_709_551_616),
				"3bffffffffffffffff",
				true,
			),
			(neg!(-1), "20", true),
			(neg!(-10), "29", true),
			(neg!(-100), "3863", true),
			(neg!(-1000), "3903e7", true),
			(Header::Float(0.0), "f90000", true),
			(Header::Float(-0.0), "f98000", true),
			(Header::Float(1.0), "f93c00", true),
			(Header::Float(1.1), "fb3ff199999999999a", true),
			(Header::Float(1.5), "f93e00", true),
			(Header::Float(65504.0), "f97bff", true),
			(Header::Float(100_000.0), "fa47c35000", true),
			(
				Header::Float(3.402_823_466_385_288_6e+38),
				"fa7f7fffff",
				true,
			),
			(Header::Float(1.0e+300), "fb7e37e43c8800759c", true),
			(Header::Float(5.960_464_477_539_063e-8), "f90001", true),
			(Header::Float(0.000_061_035_156_25), "f90400", true),
			(Header::Float(-4.0), "f9c400", true),
			(Header::Float(-4.1), "fbc010666666666666", true),
			(Header::Float(f64::INFINITY), "f97c00", true),
			(Header::Float(f64::NAN), "f97e00", true),
			(Header::Float(-f64::INFINITY), "f9fc00", true),
			(Header::Float(f64::INFINITY), "fa7f800000", false),
			(Header::Float(f64::NAN), "fa7fc00000", false),
			(Header::Float(-f64::INFINITY), "faff800000", false),
			(Header::Float(f64::INFINITY), "fb7ff0000000000000", false),
			(Header::Float(f64::NAN), "fb7ff8000000000000", false),
			(Header::Float(-f64::INFINITY), "fbfff0000000000000", false),
			(Header::Simple(simple::FALSE), "f4", true),
			(Header::Simple(simple::TRUE), "f5", true),
			(Header::Simple(simple::NULL), "f6", true),
			(Header::Simple(simple::UNDEFINED), "f7", true),
			(Header::Simple(16), "f0", true),
			(Header::Simple(24), "f818", true),
			(Header::Simple(255), "f8ff", true),
			(Header::Tag(0), "c0", true),
			(Header::Tag(1), "c1", true),
			(Header::Tag(23), "d7", true),
			(Header::Tag(24), "d818", true),
			(Header::Tag(32), "d820", true),
			(Header::Bytes(Some(0)), "40", true),
			(Header::Bytes(Some(4)), "44", true),
			(Header::Text(Some(0)), "60", true),
			(Header::Text(Some(4)), "64", true),
		];

		for (header, bytes, encode) in data.iter().copied() {
			let bytes = hex::decode(bytes).unwrap();

			let mut decoder = Decoder::from(&bytes[..]);
			match (header, decoder.pull().unwrap()) {
				(Header::Float(l), Header::Float(r)) if l.is_nan() && r.is_nan() => {}
				(l, r) => assert_eq!(l, r),
			}

			if encode {
				let mut buffer = [0u8; 1024];
				let mut writer = &mut buffer[..];
				let mut encoder = Encoder::from(&mut writer);
				encoder.push(header).unwrap();

				let len = writer.len();
				assert_eq!(&bytes[..], &buffer[..1024 - len]);
			}
		}
	}

	#[test]
	fn node() {
		let data: &[(&str, &[Header])] = &[
			("80", &[Header::Array(Some(0))]),
			(
				"83010203",
				&[
					Header::Array(Some(3)),
					Header::Positive(1),
					Header::Positive(2),
					Header::Positive(3),
				],
			),
			(
				"98190102030405060708090a0b0c0d0e0f101112131415161718181819",
				&[
					Header::Array(Some(25)),
					Header::Positive(1),
					Header::Positive(2),
					Header::Positive(3),
					Header::Positive(4),
					Header::Positive(5),
					Header::Positive(6),
					Header::Positive(7),
					Header::Positive(8),
					Header::Positive(9),
					Header::Positive(10),
					Header::Positive(11),
					Header::Positive(12),
					Header::Positive(13),
					Header::Positive(14),
					Header::Positive(15),
					Header::Positive(16),
					Header::Positive(17),
					Header::Positive(18),
					Header::Positive(19),
					Header::Positive(20),
					Header::Positive(21),
					Header::Positive(22),
					Header::Positive(23),
					Header::Positive(24),
					Header::Positive(25),
				],
			),
			("a0", &[Header::Map(Some(0))]),
			(
				"a201020304",
				&[
					Header::Map(Some(2)),
					Header::Positive(1),
					Header::Positive(2),
					Header::Positive(3),
					Header::Positive(4),
				],
			),
			("9fff", &[Header::Array(None), Header::Break]),
			(
				"9f018202039f0405ffff",
				&[
					Header::Array(None),
					Header::Positive(1),
					Header::Array(Some(2)),
					Header::Positive(2),
					Header::Positive(3),
					Header::Array(None),
					Header::Positive(4),
					Header::Positive(5),
					Header::Break,
					Header::Break,
				],
			),
			(
				"9f01820203820405ff",
				&[
					Header::Array(None),
					Header::Positive(1),
					Header::Array(Some(2)),
					Header::Positive(2),
					Header::Positive(3),
					Header::Array(Some(2)),
					Header::Positive(4),
					Header::Positive(5),
					Header::Break,
				],
			),
			(
				"83018202039f0405ff",
				&[
					Header::Array(Some(3)),
					Header::Positive(1),
					Header::Array(Some(2)),
					Header::Positive(2),
					Header::Positive(3),
					Header::Array(None),
					Header::Positive(4),
					Header::Positive(5),
					Header::Break,
				],
			),
			(
				"83019f0203ff820405",
				&[
					Header::Array(Some(3)),
					Header::Positive(1),
					Header::Array(None),
					Header::Positive(2),
					Header::Positive(3),
					Header::Break,
					Header::Array(Some(2)),
					Header::Positive(4),
					Header::Positive(5),
				],
			),
			(
				"9f0102030405060708090a0b0c0d0e0f101112131415161718181819ff",
				&[
					Header::Array(None),
					Header::Positive(1),
					Header::Positive(2),
					Header::Positive(3),
					Header::Positive(4),
					Header::Positive(5),
					Header::Positive(6),
					Header::Positive(7),
					Header::Positive(8),
					Header::Positive(9),
					Header::Positive(10),
					Header::Positive(11),
					Header::Positive(12),
					Header::Positive(13),
					Header::Positive(14),
					Header::Positive(15),
					Header::Positive(16),
					Header::Positive(17),
					Header::Positive(18),
					Header::Positive(19),
					Header::Positive(20),
					Header::Positive(21),
					Header::Positive(22),
					Header::Positive(23),
					Header::Positive(24),
					Header::Positive(25),
					Header::Break,
				],
			),
			("c340", &[Header::Tag(3), Header::Bytes(Some(0))]),
			(
				"c35fff",
				&[Header::Tag(3), Header::Bytes(None), Header::Break],
			),
		];

		for (bytes, headers) in data {
			let bytes = hex::decode(bytes).unwrap();

			let mut decoder = Decoder::from(&bytes[..]);
			for header in *headers {
				assert_eq!(*header, decoder.pull().unwrap());
			}

			let mut buffer = [0u8; 1024];
			let mut writer = &mut buffer[..];
			let mut encoder = Encoder::from(&mut writer);

			for header in headers.iter().copied() {
				encoder.push(header).unwrap();
			}

			let len = writer.len();

			assert_eq!(&bytes[..], &buffer[..1024 - len]);
		}
	}
}
