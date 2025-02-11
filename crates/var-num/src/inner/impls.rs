#![allow(clippy::cast_lossless)]

use super::{VarInt, VarUInt, float::var_float_to_primitive};

macro_rules! impl_ops_uint {
    ($($trait:ident: $method:tt($op:tt)),*) => {
        $(
            impl ::core::ops::$trait for $crate::VarUInt {
                type Output = Self;

                fn $method(self, rhs: Self) -> Self::Output {
                    match Self::normalize(self, rhs) {
                        (Self::U8(lhs), Self::U8(rhs)) => Self::U8(lhs $op rhs),
                        (Self::U16(lhs), Self::U16(rhs)) => Self::U16(lhs $op rhs),
                        (Self::U32(lhs), Self::U32(rhs)) => Self::U32(lhs $op rhs),
                        (Self::U64(lhs), Self::U64(rhs)) => Self::U64(lhs $op rhs),
                        (Self::U128(lhs), Self::U128(rhs)) => Self::U128(lhs $op rhs),
                        (lhs, rhs) => panic!("normalization failed: {lhs} {} {rhs}", stringify!($op)),
                    }
                }
            }
        )*
    }
}

macro_rules! impl_op {
	($trait:ident: $method:tt($op:tt) -> $enm:ty, $byl:ident, $by:ty, $wrl:ident, $wrd:ty, $dll:ident, $dbl:ty, $qdl:ident, $qad:ty, $ocl:ident, $oct:ty) => {
		impl ::core::ops::$trait for $enm {
			type Output = Self;

			fn $method(self, rhs: Self) -> Self::Output {
				match Self::normalize(self, rhs) {
                    (Self::$byl(lhs), Self::$byl(rhs)) => (lhs $op rhs).into(),
                    (Self::$wrl(lhs), Self::$wrl(rhs)) => (lhs $op rhs).into(),
                    (Self::$dll(lhs), Self::$dll(rhs)) => (lhs $op rhs).into(),
                    (Self::$qdl(lhs), Self::$qdl(rhs)) => (lhs $op rhs).into(),
                    (Self::$ocl(lhs), Self::$ocl(rhs)) => (lhs $op rhs).into(),
                    (lhs, rhs) => panic!("normalization failed: {} {} {}", lhs, stringify!($op), rhs),
                }
			}
		}
	};
}

macro_rules! from_inner {
	($enm:ty = $byl:ident:$by:ty, $wrl:ident:$wrd:ty, $dll:ident:$dbl:ty, $qdl:ident:$qad:ty, $ocl:ident:$oct:ty) => {
		impl $enm {
            #[must_use]
            pub fn upgrade(self) -> Self {
                match self {
                    Self::$byl(value) => Self::$wrl(value.into()),
                    Self::$wrl(value) => Self::$dll(value.into()),
                    Self::$dll(value) => Self::$qdl(value.into()),
                    Self::$qdl(value) => Self::$ocl(value.into()),
                    Self::$ocl(o) => panic!("cannot upgrade octal value: {o}"),
                }
            }

            #[must_use]
            pub fn downgrade(self) -> Self {
                match self {
                    Self::$byl(value) => panic!("cannot downgrade byte value: {value}"),
                    Self::$wrl(value) => Self::$byl(value as $by),
                    Self::$dll(value) => Self::$wrl(value as $wrd),
                    Self::$qdl(value) => Self::$dll(value as $dbl),
                    Self::$ocl(value) => Self::$qdl(value as $qad),
                }
            }

            #[must_use]
            pub fn normalize(lhs: Self, rhs: Self) -> (Self, Self) {
                match (lhs, rhs) {
                    (Self::$ocl(lhs), Self::$ocl(rhs)) => (Self::$ocl(lhs), Self::$ocl(rhs)),
                    (Self::$ocl(lhs), rhs) => Self::normalize(Self::$ocl(lhs), rhs.upgrade()),
                    (lhs, Self::$ocl(rhs)) => Self::normalize(lhs.upgrade(), Self::$ocl(rhs)),
                    (Self::$qdl(lhs), Self::$qdl(rhs)) => (Self::$qdl(lhs), Self::$qdl(rhs)),
                    (Self::$qdl(lhs), rhs) => Self::normalize(Self::$qdl(lhs), rhs.upgrade()),
                    (lhs, Self::$qdl(rhs)) => Self::normalize(lhs.upgrade(), Self::$qdl(rhs)),
                    (Self::$dll(lhs), Self::$dll(rhs)) => (Self::$dll(lhs), Self::$dll(rhs)),
                    (Self::$dll(lhs), rhs) => Self::normalize(Self::$dll(lhs), rhs.upgrade()),
                    (lhs, Self::$dll(rhs)) => Self::normalize(lhs.upgrade(), Self::$dll(rhs)),
                    (Self::$wrl(lhs), Self::$wrl(rhs)) => (Self::$wrl(lhs), Self::$wrl(rhs)),
                    (Self::$wrl(lhs), rhs) => Self::normalize(Self::$wrl(lhs), rhs.upgrade()),
                    (lhs, Self::$wrl(rhs)) => Self::normalize(lhs.upgrade(), Self::$wrl(rhs)),
                    (Self::$byl(lhs), Self::$byl(rhs)) => (Self::$byl(lhs), Self::$byl(rhs)),

                }
            }

            #[must_use]
            pub fn to_bytes(self) -> ::alloc::vec::Vec<u8> {
                match self {
                    Self::$byl(v) => vec![v as u8],
                    Self::$wrl(v) => v.to_be_bytes().to_vec(),
                    Self::$dll(v) => v.to_be_bytes().to_vec(),
                    Self::$qdl(v) => v.to_be_bytes().to_vec(),
                    Self::$ocl(v) => v.to_be_bytes().to_vec(),
                }
            }

            #[must_use]
            pub fn from_bytes(raw: &[u8]) -> Self {
                match raw.len() {
                    1 => Self::$byl(raw[0] as $by),
                    2 => {
                        let mut bytes = [0; 2];
                        bytes.copy_from_slice(raw);
                        Self::$wrl(<$wrd>::from_be_bytes(bytes))
                    }
                    4 => {
                        let mut bytes = [0; 4];
                        bytes.copy_from_slice(raw);
                        Self::$dll(<$dbl>::from_be_bytes(bytes))
                    }
                    8 => {
                        let mut bytes = [0; 8];
                        bytes.copy_from_slice(raw);
                        Self::$qdl(<$qad>::from_be_bytes(bytes))
                    }
                    16 => {
                        let mut bytes = [0; 16];
                        bytes.copy_from_slice(raw);
                        Self::$ocl(<$oct>::from_be_bytes(bytes))
                    }
                    l => panic!("invalid byte length: {l}"),
                }
            }
        }

		impl ::core::convert::From<$by> for $enm {
			fn from(value: $by) -> Self {
				Self::$byl(value)
			}
		}

		impl ::core::convert::From<$wrd> for $enm {
			fn from(value: $wrd) -> Self {
				if <$by>::try_from(value).is_ok() {
					Self::$byl(value as $by)
				} else {
					Self::$wrl(value)
				}
			}
		}

		impl ::core::convert::From<$dbl> for $enm {
			fn from(value: $dbl) -> Self {
				if <$wrd>::try_from(value).is_ok() {
					(value as $wrd).into()
				} else {
					Self::$dll(value)
				}
			}
		}

		impl ::core::convert::From<$qad> for $enm {
			fn from(value: $qad) -> Self {
				if <$dbl>::try_from(value).is_ok() {
					(value as $dbl).into()
				} else {
					Self::$qdl(value)
				}
			}
		}

		impl ::core::convert::From<$oct> for $enm {
			fn from(value: $oct) -> Self {
				if <$qad>::try_from(value).is_ok() {
					(value as $qad).into()
				} else {
					Self::$ocl(value)
				}
			}
		}

        impl_op!(Rem: rem(%) -> $enm, $byl, $by, $wrl, $wrd, $dll, $dbl, $qdl, $qad, $ocl, $oct);
        impl_op!(BitAnd: bitand(&) -> $enm, $byl, $by, $wrl, $wrd, $dll, $dbl, $qdl, $qad, $ocl, $oct);
        impl_op!(BitOr: bitor(|) -> $enm, $byl, $by, $wrl, $wrd, $dll, $dbl, $qdl, $qad, $ocl, $oct);
        impl_op!(BitXor: bitxor(^) -> $enm, $byl, $by, $wrl, $wrd, $dll, $dbl, $qdl, $qad, $ocl, $oct);
        impl_op!(Shl: shl(<<) -> $enm, $byl, $by, $wrl, $wrd, $dll, $dbl, $qdl, $qad, $ocl, $oct);
        impl_op!(Shr: shr(>>) -> $enm, $byl, $by, $wrl, $wrd, $dll, $dbl, $qdl, $qad, $ocl, $oct);
	};
}

macro_rules! from_cast {
    ($($enm:ident: $($t:ty as $to:ty)*),*) => {
        $(
            $(
                impl ::core::convert::From<$t> for $enm {
                    #[allow(clippy::cast_lossless)]
                    fn from(value: $t) -> Self {
                        (value as $to).into()
                    }
                }
            )*
        )*
    };
}

macro_rules! var_int_to_primitive {
    ($($t:ty)*) => {
        $(
            impl ::core::convert::From<$crate::VarInt> for $t {
                fn from(value: $crate::VarInt) -> Self {
                    match value {
                        $crate::VarInt::I8(i) => i as $t,
                        $crate::VarInt::I16(i) => i as $t,
                        $crate::VarInt::I32(i) => i as $t,
                        $crate::VarInt::I64(i) => i as $t,
                        $crate::VarInt::I128(i) => i as $t,
                    }
                }
            }
        )*
    };
}

macro_rules! var_uint_to_primitive {
    ($($t:ty)*) => {
        $(
            impl ::core::convert::From<$crate::VarUInt> for $t {
                fn from(value: $crate::VarUInt) -> Self {
                    match value {
                        $crate::VarUInt::U8(v) => v as $t,
                        $crate::VarUInt::U16(v) => v as $t,
                        $crate::VarUInt::U32(v) => v as $t,
                        $crate::VarUInt::U64(v) => v as $t,
                        $crate::VarUInt::U128(v) => v as $t,
                    }
                }
            }
        )*
    };
}

macro_rules! var_num_to_primitive {
    ($($t:ty)*) => {
        $(
            impl ::core::convert::From<$crate::VarNum> for $t {
                fn from(value: $crate::VarNum) -> Self {
                    use ::core::convert::Into as _;

                    match value {
                        $crate::VarNum::Int(i) => i.into(),
                        $crate::VarNum::UInt(u) => u.into(),
                        $crate::VarNum::Float(f) => f.into(),
                    }
                }
            }
        )*
    };
}

macro_rules! to_primitive {
    ($($t:ty)*) => {
        var_int_to_primitive! { $($t)* }
        var_uint_to_primitive! { $($t)* }
        var_float_to_primitive! { $($t)* }
        var_num_to_primitive! { $($t)* }
    };
}

to_primitive! { isize i8 i16 i32 i64 i128 usize u8 u16 u32 u64 u128 f32 f64 }

from_inner!(VarInt = I8: i8, I16: i16, I32: i32, I64: i64, I128: i128);
from_inner!(VarUInt = U8: u8, U16: u16, U32: u32, U64: u64, U128: u128);

from_cast! {
	VarUInt: usize as u64,
	VarInt: isize as i64
}

from_cast! {
	VarUInt: bool as u8,
	VarInt: bool as i8
}

impl_ops_uint! {
	Add: add(+),
	Sub: sub(-),
	Mul: mul(*),
	Div: div(/)
}
