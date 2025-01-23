use super::Reg;

macro_rules! declare_val {
	($typename:ident, $num:ident, $inum:ident) => {
		#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
		pub enum $typename {
			Reg($crate::asm::Reg),
			Lit($num),
		}

		impl $typename {
			#[must_use]
			pub const fn reg(reg: $crate::asm::Reg) -> Self {
				Self::Reg(reg)
			}

			#[must_use]
			pub const fn lit(lit: $num) -> Self {
				Self::Lit(lit)
			}

			#[must_use]
			pub const fn signed_lit(lit: $inum) -> Self {
				Self::lit(lit as $num)
			}

			#[must_use]
			pub const fn is_reg(self) -> bool {
				matches!(self, Self::Reg(_))
			}

			#[must_use]
			pub const fn is_lit(self) -> bool {
				matches!(self, Self::Lit(_))
			}

			#[must_use]
			pub const fn value(self) -> $num {
				match self {
					Self::Reg(reg) => reg.code() as _,
					Self::Lit(num) => num,
				}
			}

			#[must_use]
			pub fn to_lasm_signed(self) -> String {
				match self {
					Self::Reg(reg) => reg.name().to_owned(),
					Self::Lit(num) => match num as $inum {
						num @ $inum::MIN..=-1 => format!("-{:#X}", -num),
						num @ 0..=$inum::MAX => format!("{num:#X}"),
					},
				}
			}

			pub fn to_lasm_with(self, formatter: impl FnOnce($num) -> String) -> String {
				match self {
					Self::Reg(reg) => reg.name().to_owned(),
					Self::Lit(num) => formatter(num),
				}
			}
		}

		impl ::std::convert::From<$typename> for $num {
			fn from(reg_or_lit: $typename) -> Self {
				reg_or_lit.value()
			}
		}

		impl ::std::convert::From<$crate::asm::Reg> for $typename {
			fn from(reg: $crate::asm::Reg) -> Self {
				Self::reg(reg)
			}
		}

		impl ::std::convert::From<$num> for $typename {
			fn from(lit: $num) -> Self {
				Self::lit(lit)
			}
		}

		impl ::std::convert::From<$inum> for $typename {
			fn from(lit: $inum) -> Self {
				Self::signed_lit(lit)
			}
		}

		impl $crate::asm::ToLasm for $typename {
			fn to_lasm(&self) -> ::std::borrow::Cow<'static, str> {
				match self {
					Self::Reg(reg) => ::std::borrow::Cow::Borrowed(reg.name()),
					Self::Lit(num) => ::std::borrow::Cow::Owned(format!("{num:#X}")),
				}
			}
		}
	};
}

declare_val!(RegOrLit1, u8, i8);
declare_val!(RegOrLit2, u16, i16);

impl From<u8> for RegOrLit2 {
	fn from(value: u8) -> Self {
		Self::lit(value.into())
	}
}

impl From<i8> for RegOrLit2 {
	fn from(value: i8) -> Self {
		Self::signed_lit(value.into())
	}
}
