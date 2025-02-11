macro_rules! from_primitive_var_float {
    ($($type:ty)*) => {
        $(
            #[allow(clippy::cast_lossless)]
            impl ::core::convert::From<$type> for $crate::VarFloat {
                fn from(value: $type) -> Self {
                    Self::F32(value as f32)
                }
            }
        )*
    };
}

macro_rules! from_primitive_var_double {
    ($($type:ty)*) => {
        $(
            #[allow(clippy::cast_lossless)]
            impl ::core::convert::From<$type> for $crate::VarFloat {
                fn from(value: $type) -> Self {
                    if value <= f32::MAX as $type {
                        (value as f32).into()
                    } else {
                        Self::F64(value as f64)
                    }
                }
            }
        )*
    };
}

macro_rules! impl_bin_var_float_to_bits {
    ($($trait:ident: $method:ident($op:tt),)*) => {
        $(
            impl ::core::ops::$trait for $crate::VarFloat {
                type Output = Self;

                fn $method(self, rhs: Self) -> Self::Output {
                    match Self::normalize(self, rhs) {
                        (Self::F32(lhs), Self::F32(rhs)) => Self::F32(f32::from_bits(lhs.to_bits() $op rhs.to_bits())),
                        (Self::F64(lhs), Self::F64(rhs)) => Self::F64(f64::from_bits(lhs.to_bits() $op rhs.to_bits())),
                        (lhs, rhs) => panic!("normalization failed: {lhs} {} {rhs}", stringify!($op)),
                    }
                }
            }
        )*
    };
}

macro_rules! impl_ops_float {
    ($($trait:ident: $method:tt($op:tt)),*) => {
        $(
            impl ::core::ops::$trait for $crate::VarFloat {
                type Output = Self;

                fn $method(self, rhs: Self) -> Self::Output {
                    match Self::normalize(self, rhs) {
                        (Self::F32(lhs), Self::F32(rhs)) => Self::F32(lhs $op rhs),
                        (Self::F64(lhs), Self::F64(rhs)) => Self::F64(lhs $op rhs),
                        (lhs, rhs) => panic!("normalization failed: {lhs} {} {rhs}", stringify!($op)),
                    }
                }
            }
        )*
    };
}

macro_rules! var_float_to_primitive {
    ($($t:ty)*) => {
        $(
            impl ::core::convert::From<$crate::VarFloat> for $t {
                fn from(value: $crate::VarFloat) -> Self {
                    match value {
                        $crate::VarFloat::F32(f) => f as $t,
                        $crate::VarFloat::F64(f) => f as $t,
                    }
                }
            }
        )*
    };
}

pub(crate) use var_float_to_primitive;

from_primitive_var_float!(f32 u8 u16 u32 i8 i16 i32);
from_primitive_var_double!(f64 u64 i64 usize isize);
impl_bin_var_float_to_bits! {
	BitAnd: bitand(&),
	BitOr: bitor(|),
	BitXor: bitxor(^),
	Shl: shl(<<),
	Shr: shr(>>),
}

impl_ops_float! {
	Add: add(+),
	Sub: sub(-),
	Mul: mul(*),
	Div: div(/)
}
