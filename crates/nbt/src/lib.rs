#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod compound;
pub mod conv;
mod error;
pub mod list;
#[cfg(feature = "serde")]
pub mod serde;
pub mod value;

pub use self::{compound::Compound, error::*, list::List, value::Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Tag {
	End,
	Byte,
	Short,
	Int,
	Long,
	Float,
	Double,
	ByteArray,
	String,
	List,
	Compound,
	IntArray,
	LongArray,
}

#[macro_export]
macro_rules! compound {
    (<$string_type:ty> $($key:expr => $value:expr),* $(,)?) => {
        <$crate::Compound<$string_type> as ::std::iter::FromIterator<($string_type, $crate::Value<$string_type>)>>::from_iter([
            $(
                (
                    ::std::convert::Into::<$string_type>::into($key),
                    ::std::convert::Into::<$crate::Value<$string_type>>::into($value)
                ),
            )*
        ])
    };

    ($($key:expr => $value:expr),* $(,)?) => {
        compound!(<::std::string::String> $($key => $value),*)
    };
}

#[cfg(feature = "java_string")]
#[macro_export]
macro_rules! jcompound {
    ($($key:expr => $value:expr),* $(,)?) => {
        compound!(<::java_string::JavaString> $($key => $value),*)
    }
}
