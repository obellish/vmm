use super::{VarFloat, VarInt, VarNum, VarUInt};

macro_rules! impl_var_num {
    ($($trait:ident: $method:ident),*) => {
        $(
            impl ::core::ops::$trait for $crate::VarNum {
                type Output = Self;

                fn $method(self) -> Self::Output {
                    match self {
                        Self::Int(i) => Self::Int(i.$method()),
                        Self::UInt(u) => Self::UInt(u.$method()),
                        Self::Float(f) => Self::Float(f.$method()),
                    }
                }
            }
        )*
    };
    ($($trait:ident: $method:ident($op:tt)),*) => {
        $(
            impl ::core::ops::$trait for $crate::VarNum {
                type Output = Self;

                fn $method(self, rhs: Self) -> Self::Output {
                    match (self, rhs) {
                        (lhs, rhs) => {
                            let (lhs, rhs) = Self::normalize(lhs, rhs);
                            match (lhs, rhs) {
                                (Self::Int(lhs), Self::Int(rhs)) => Self::Int(lhs $op rhs),
                                (Self::UInt(lhs), Self::UInt(rhs)) => Self::UInt(lhs $op rhs),
                                (Self::Float(lhs), Self::Float(rhs)) => Self::Float(lhs $op rhs),
                                (lhs, rhs) => panic!("normalization failed: {lhs} {} {rhs}", stringify!($op)),
                            }
                        }
                    }
                }
            }
        )*
    }
}

macro_rules! from_var_num {
	($($enm:ident: $($t:ty)*),*) => {
        $(
            $(
                impl ::core::convert::From<$t> for $crate::VarNum {
                    fn from(value: $t) -> Self {
                        use ::core::convert::Into as _;
                        Self::$enm(value.into())
                    }
                }
            )*
        )*
    };
}

macro_rules! impl_assign {
	($($ty:ty => [$($trait:ident($fn:ident $op:tt)),*]),*) => {
        $(
            $(
                impl ::core::ops::$trait for $ty {
                    fn $fn(&mut self, other: Self) {
                        *self = *self $op other;
                    }
                }
            )*
        )*
    };
    ($($ty:ty),*) => {
        $(
            impl_assign! {
                $ty => [
                    AddAssign(add_assign +),
                    SubAssign(sub_assign -),
                    MulAssign(mul_assign *),
                    DivAssign(div_assign /),
                    RemAssign(rem_assign %),
                    BitAndAssign(bitand_assign &),
                    BitOrAssign(bitor_assign |),
                    BitXorAssign(bitxor_assign ^),
                    ShlAssign(shl_assign <<),
                    ShrAssign(shr_assign >>)
                ]
            }
        )*
    }
}

impl_var_num! {
	Neg: neg,
	Not: not
}

impl_var_num! {
	Add: add(+),
	Sub: sub(-),
	Mul: mul(*),
	Div: div(/),
	Rem: rem(%),
	BitAnd: bitand(&),
	BitOr: bitor(|),
	BitXor: bitxor(^),
	Shl: shl(<<),
	Shr: shr(>>)
}

from_var_num! {
	Int: isize i8 i16 i32 i64 i128,
	UInt: usize bool u8 u16 u32 u64 u128,
	Float: f32 f64
}

impl_assign! { VarInt, VarUInt, VarFloat, VarNum }
