#![allow(clippy::cast_lossless)]

use std::borrow::Cow;

use super::{Compound, List};

#[derive(Debug, Clone)]
pub enum Value<S = String> {
	Byte(i8),
	Short(i16),
	Int(i32),
	Long(i64),
	Float(f32),
	Double(f64),
	ByteArray(Vec<i8>),
	String(S),
	List(List<S>),
	Compound(Compound<S>),
	IntArray(Vec<i32>),
	LongArray(Vec<i64>),
}

impl<S> Value<S> {
	pub fn as_value_ref(&self) -> ValueRef<'_, S> {
		match self {
			Self::Byte(v) => ValueRef::Byte(v),
			Self::Short(v) => ValueRef::Short(v),
			Self::Int(v) => ValueRef::Int(v),
			Self::Long(v) => ValueRef::Long(v),
			Self::Float(v) => ValueRef::Float(v),
			Self::Double(v) => ValueRef::Double(v),
			Self::ByteArray(v) => ValueRef::ByteArray(&v[..]),
			Self::String(v) => ValueRef::String(v),
			Self::List(v) => ValueRef::List(v),
			Self::Compound(v) => ValueRef::Compound(v),
			Self::IntArray(v) => ValueRef::IntArray(&v[..]),
			Self::LongArray(v) => ValueRef::LongArray(&v[..]),
		}
	}

	pub const fn as_value_mut(&mut self) -> ValueMut<'_, S> {
		match self {
			Self::Byte(v) => ValueMut::Byte(v),
			Self::Short(v) => ValueMut::Short(v),
			Self::Int(v) => ValueMut::Int(v),
			Self::Long(v) => ValueMut::Long(v),
			Self::Float(v) => ValueMut::Float(v),
			Self::Double(v) => ValueMut::Double(v),
			Self::ByteArray(v) => ValueMut::ByteArray(v),
			Self::String(v) => ValueMut::String(v),
			Self::List(v) => ValueMut::List(v),
			Self::Compound(v) => ValueMut::Compound(v),
			Self::IntArray(v) => ValueMut::IntArray(v),
			Self::LongArray(v) => ValueMut::LongArray(v),
		}
	}
}

impl<S> From<bool> for Value<S> {
	fn from(b: bool) -> Self {
		Self::Byte(b.into())
	}
}

impl<S> From<Vec<i8>> for Value<S> {
	fn from(v: Vec<i8>) -> Self {
		Self::ByteArray(v)
	}
}

impl From<String> for Value<String> {
	fn from(v: String) -> Self {
		Self::String(v)
	}
}

impl From<&String> for Value<String> {
	fn from(value: &String) -> Self {
		Self::String(value.clone())
	}
}

impl<'a> From<&'a str> for Value<String> {
	fn from(v: &'a str) -> Self {
		Self::String(v.to_owned())
	}
}

impl<'a> From<Cow<'a, str>> for Value<String> {
	fn from(v: Cow<'a, str>) -> Self {
		Self::String(v.into_owned())
	}
}

impl From<String> for Value<Cow<'_, str>> {
	fn from(v: String) -> Self {
		Self::String(Cow::Owned(v))
	}
}

impl<'a> From<&'a String> for Value<Cow<'a, str>> {
	fn from(v: &'a String) -> Self {
		Self::String(Cow::Borrowed(v))
	}
}

impl<'a> From<&'a str> for Value<Cow<'a, str>> {
	fn from(v: &'a str) -> Self {
		Self::String(Cow::Borrowed(v))
	}
}

impl<'a> From<Cow<'a, str>> for Value<Cow<'a, str>> {
	fn from(v: Cow<'a, str>) -> Self {
		Self::String(v)
	}
}

#[cfg(feature = "java_string")]
impl From<java_string::JavaString> for Value<java_string::JavaString> {
	fn from(v: java_string::JavaString) -> Self {
		Self::String(v)
	}
}

#[cfg(feature = "java_string")]
impl From<&java_string::JavaString> for Value<java_string::JavaString> {
	fn from(v: &java_string::JavaString) -> Self {
		Self::String(v.clone())
	}
}

#[cfg(feature = "java_string")]
impl<'a> From<&'a java_string::JavaStr> for Value<java_string::JavaString> {
	fn from(v: &'a java_string::JavaStr) -> Self {
		Self::String(v.to_owned())
	}
}

#[cfg(feature = "java_string")]
impl<'a> From<Cow<'a, java_string::JavaStr>> for Value<java_string::JavaString> {
	fn from(v: Cow<'a, java_string::JavaStr>) -> Self {
		Self::String(v.into_owned())
	}
}

#[cfg(feature = "java_string")]
impl From<String> for Value<java_string::JavaString> {
	fn from(v: String) -> Self {
		Self::String(java_string::JavaString::from(v))
	}
}

#[cfg(feature = "java_string")]
impl From<&String> for Value<java_string::JavaString> {
	fn from(v: &String) -> Self {
		Self::String(java_string::JavaString::from(v))
	}
}

#[cfg(feature = "java_string")]
impl<'a> From<&'a str> for Value<java_string::JavaString> {
	fn from(v: &'a str) -> Self {
		Self::String(java_string::JavaString::from(v))
	}
}

#[cfg(feature = "java_string")]
impl<'a> From<Cow<'a, str>> for Value<java_string::JavaString> {
	fn from(v: Cow<'a, str>) -> Self {
		Self::String(java_string::JavaString::from(v))
	}
}

#[cfg(feature = "java_string")]
impl From<java_string::JavaString> for Value<Cow<'_, java_string::JavaStr>> {
	fn from(v: java_string::JavaString) -> Self {
		Self::String(Cow::Owned(v))
	}
}

#[cfg(feature = "java_string")]
impl<'a> From<&'a java_string::JavaString> for Value<Cow<'a, java_string::JavaStr>> {
	fn from(v: &'a java_string::JavaString) -> Self {
		Self::String(Cow::Borrowed(v))
	}
}

#[cfg(feature = "java_string")]
impl<'a> From<&'a java_string::JavaStr> for Value<Cow<'a, java_string::JavaStr>> {
	fn from(v: &'a java_string::JavaStr) -> Self {
		Self::String(Cow::Borrowed(v))
	}
}

#[cfg(feature = "java_string")]
impl<'a> From<Cow<'a, java_string::JavaStr>> for Value<Cow<'a, java_string::JavaStr>> {
	fn from(v: Cow<'a, java_string::JavaStr>) -> Self {
		Self::String(v)
	}
}

#[cfg(feature = "java_string")]
impl From<String> for Value<Cow<'_, java_string::JavaStr>> {
	fn from(v: String) -> Self {
		Self::String(Cow::Owned(java_string::JavaString::from(v)))
	}
}

#[cfg(feature = "java_string")]
impl<'a> From<&'a String> for Value<Cow<'a, java_string::JavaStr>> {
	fn from(v: &'a String) -> Self {
		Self::String(Cow::Borrowed(java_string::JavaStr::from_str(v)))
	}
}

#[cfg(feature = "java_string")]
impl<'a> From<&'a str> for Value<Cow<'a, java_string::JavaStr>> {
	fn from(v: &'a str) -> Self {
		Self::String(Cow::Borrowed(java_string::JavaStr::from_str(v)))
	}
}

#[cfg(feature = "java_string")]
impl<'a> From<Cow<'a, str>> for Value<Cow<'a, java_string::JavaStr>> {
	fn from(v: Cow<'a, str>) -> Self {
		Self::String(match v {
			Cow::Borrowed(str) => Cow::Borrowed(java_string::JavaStr::from_str(str)),
			Cow::Owned(str) => Cow::Owned(java_string::JavaString::from(str)),
		})
	}
}

impl<S> From<Vec<i32>> for Value<S> {
	fn from(v: Vec<i32>) -> Self {
		Self::IntArray(v)
	}
}

impl<S> From<Vec<i64>> for Value<S> {
	fn from(v: Vec<i64>) -> Self {
		Self::LongArray(v)
	}
}

impl<S> From<ValueRef<'_, S>> for Value<S>
where
	S: Clone,
{
	fn from(v: ValueRef<'_, S>) -> Self {
		v.to_value()
	}
}

impl<S> From<&ValueRef<'_, S>> for Value<S>
where
	S: Clone,
{
	fn from(v: &ValueRef<'_, S>) -> Self {
		v.to_value()
	}
}

impl<S> From<ValueMut<'_, S>> for Value<S>
where
	S: Clone,
{
	fn from(v: ValueMut<'_, S>) -> Self {
		v.to_value()
	}
}

impl<S> From<&ValueMut<'_, S>> for Value<S>
where
	S: Clone,
{
	fn from(v: &ValueMut<'_, S>) -> Self {
		v.to_value()
	}
}

#[cfg(feature = "uuid")]
impl<S> From<uuid::Uuid> for Value<S> {
	fn from(value: uuid::Uuid) -> Self {
		let (most, least) = value.as_u64_pair();

		let first = (most >> 32) as i32;
		let second = most as i32;
		let third = (least >> 32) as i32;
		let fourth = least as i32;

		Self::IntArray(vec![first, second, third, fourth])
	}
}

#[cfg(feature = "vmm_ident")]
impl<I, S> From<vmm_ident::Ident<I>> for Value<S>
where
	I: Into<Self>,
{
	fn from(value: vmm_ident::Ident<I>) -> Self {
		value.into_inner().into()
	}
}

#[derive(Debug, Clone, Copy)]
pub enum ValueRef<'a, S = String> {
	Byte(&'a i8),
	Short(&'a i16),
	Int(&'a i32),
	Long(&'a i64),
	Float(&'a f32),
	Double(&'a f64),
	ByteArray(&'a [i8]),
	String(&'a S),
	List(&'a List<S>),
	Compound(&'a Compound<S>),
	IntArray(&'a [i32]),
	LongArray(&'a [i64]),
}

impl<S: Clone> ValueRef<'_, S> {
	#[must_use]
	pub fn to_value(&self) -> Value<S> {
		match *self {
			Self::Byte(v) => Value::Byte(*v),
			Self::Short(v) => Value::Short(*v),
			Self::Int(v) => Value::Int(*v),
			Self::Long(v) => Value::Long(*v),
			Self::Float(v) => Value::Float(*v),
			Self::Double(v) => Value::Double(*v),
			Self::ByteArray(v) => Value::ByteArray(v.to_vec()),
			Self::String(v) => Value::String(v.to_owned()),
			Self::List(v) => Value::List(v.clone()),
			Self::Compound(v) => Value::Compound(v.clone()),
			Self::IntArray(v) => Value::IntArray(v.to_vec()),
			Self::LongArray(v) => Value::LongArray(v.to_vec()),
		}
	}
}

impl<'a> From<&'a [i8]> for ValueRef<'a> {
	fn from(v: &'a [i8]) -> Self {
		Self::ByteArray(v)
	}
}

impl<'a> From<&'a String> for ValueRef<'a, String> {
	fn from(v: &'a String) -> Self {
		Self::String(v)
	}
}

impl<'a, S> From<&'a [i32]> for ValueRef<'a, S> {
	fn from(v: &'a [i32]) -> Self {
		Self::IntArray(v)
	}
}

impl<'a, S> From<&'a [i64]> for ValueRef<'a, S> {
	fn from(v: &'a [i64]) -> Self {
		Self::LongArray(v)
	}
}

impl<'a, S> From<&'a Value<S>> for ValueRef<'a, S> {
	fn from(v: &'a Value<S>) -> Self {
		v.as_value_ref()
	}
}

impl<'a, S> From<ValueMut<'a, S>> for ValueRef<'a, S> {
	fn from(v: ValueMut<'a, S>) -> Self {
		v.into_value_ref()
	}
}

#[cfg(feature = "vmm_ident")]
impl<'a> From<&'a vmm_ident::Ident<String>> for ValueRef<'a, String> {
	fn from(v: &'a vmm_ident::Ident<String>) -> Self {
		Self::String(v.as_ref())
	}
}

#[derive(Debug)]
pub enum ValueMut<'a, S = String> {
	Byte(&'a mut i8),
	Short(&'a mut i16),
	Int(&'a mut i32),
	Long(&'a mut i64),
	Float(&'a mut f32),
	Double(&'a mut f64),
	ByteArray(&'a mut Vec<i8>),
	String(&'a mut S),
	List(&'a mut List<S>),
	Compound(&'a mut Compound<S>),
	IntArray(&'a mut Vec<i32>),
	LongArray(&'a mut Vec<i64>),
}

impl<'a, S> ValueMut<'a, S> {
	#[must_use]
	pub fn to_value(&self) -> Value<S>
	where
		S: Clone,
	{
		match self {
			Self::Byte(v) => Value::Byte(**v),
			Self::Short(v) => Value::Short(**v),
			Self::Int(v) => Value::Int(**v),
			Self::Long(v) => Value::Long(**v),
			Self::Float(v) => Value::Float(**v),
			Self::Double(v) => Value::Double(**v),
			Self::ByteArray(v) => Value::ByteArray((*v).clone()),
			Self::String(v) => Value::String((*v).clone()),
			Self::List(v) => Value::List((*v).clone()),
			Self::Compound(v) => Value::Compound((*v).clone()),
			Self::IntArray(v) => Value::IntArray((*v).clone()),
			Self::LongArray(v) => Value::LongArray((*v).clone()),
		}
	}

	#[must_use]
	pub fn into_value_ref(self) -> ValueRef<'a, S> {
		match self {
			Self::Byte(v) => ValueRef::Byte(v),
			Self::Short(v) => ValueRef::Short(v),
			Self::Int(v) => ValueRef::Int(v),
			Self::Long(v) => ValueRef::Long(v),
			Self::Float(v) => ValueRef::Float(v),
			Self::Double(v) => ValueRef::Double(v),
			Self::ByteArray(v) => ValueRef::ByteArray(&v[..]),
			Self::String(v) => ValueRef::String(v),
			Self::List(v) => ValueRef::List(v),
			Self::Compound(v) => ValueRef::Compound(v),
			Self::IntArray(v) => ValueRef::IntArray(&v[..]),
			Self::LongArray(v) => ValueRef::LongArray(&v[..]),
		}
	}
}

impl<'a, S> From<&'a mut Vec<i8>> for ValueMut<'a, S> {
	fn from(v: &'a mut Vec<i8>) -> Self {
		Self::ByteArray(v)
	}
}

impl<'a> From<&'a mut String> for ValueMut<'a, String> {
	fn from(v: &'a mut String) -> Self {
		Self::String(v)
	}
}

impl<'a, S> From<&'a mut Vec<i32>> for ValueMut<'a, S> {
	fn from(v: &'a mut Vec<i32>) -> Self {
		Self::IntArray(v)
	}
}

impl<'a, S> From<&'a mut Vec<i64>> for ValueMut<'a, S> {
	fn from(v: &'a mut Vec<i64>) -> Self {
		Self::LongArray(v)
	}
}

impl<'a, S> From<&'a mut Value<S>> for ValueMut<'a, S> {
	fn from(v: &'a mut Value<S>) -> Self {
		v.as_value_mut()
	}
}

macro_rules! impl_value {
    ($name:ident, $($lifetime:lifetime)?, ($($deref:tt)*), $($reference:tt)*) => {
        macro_rules! as_number {
            ($method_name:ident, $ty:ty, $($deref)*) => {
                #[doc = concat!("If this value is a number, returns the `", stringify!($ty), "` representation of this value.")]
                #[must_use]
                pub fn $method_name(&self) -> ::std::option::Option<$ty> {
                    #[allow(trivial_numeric_casts)]
                    match self {
                        Self::Byte(v) => ::std::option::Option::Some($($deref)* v as $ty),
                        Self::Short(v) => ::std::option::Option::Some($($deref)* v as $ty),
                        Self::Int(v) => ::std::option::Option::Some($($deref)* v as $ty),
                        Self::Long(v) => ::std::option::Option::Some($($deref)* v as $ty),
                        Self::Float(v) => ::std::option::Option::Some(v.floor() as $ty),
                        Self::Double(v) => ::std::option::Option::Some(v.floor() as $ty),
                        _ => ::std::option::Option::None,
                    }
                }
            }
        }

        macro_rules! as_number_float {
            ($method_name:ident, $ty:ty, $($deref)*) => {
                #[doc = concat!("If this value is a number, returns the `", stringify!($ty), "` representation of this value.")]
                #[must_use]
                pub const fn $method_name(&self) -> ::std::option::Option<$ty> {
                    #[allow(trivial_numeric_casts)]
                    match self {
                        Self::Byte(v) => ::std::option::Option::Some($($deref)* v as $ty),
                        Self::Short(v) => ::std::option::Option::Some($($deref)* v as $ty),
                        Self::Int(v) => ::std::option::Option::Some($($deref)* v as $ty),
                        Self::Long(v) => ::std::option::Option::Some($($deref)* v as $ty),
                        Self::Float(v) => ::std::option::Option::Some($($deref)* v as $ty),
                        Self::Double(v) => ::std::option::Option::Some($($deref)* v as $ty),
                        _ => ::std::option::Option::None,
                    }
                }
            }
        }

        impl <$($lifetime,)? S> $name<$($lifetime,)? S> {
            /// Returns the type of this value.
            #[must_use]
            pub const fn tag(&self) -> $crate::Tag {
                match self {
                    Self::Byte(_) => $crate::Tag::Byte,
                    Self::Short(_) => $crate::Tag::Short,
                    Self::Int(_) => $crate::Tag::Int,
                    Self::Long(_) => $crate::Tag::Long,
                    Self::Float(_) => $crate::Tag::Float,
                    Self::Double(_) => $crate::Tag::Double,
                    Self::ByteArray(_) => $crate::Tag::ByteArray,
                    Self::String(_) => $crate::Tag::String,
                    Self::List(_) => $crate::Tag::List,
                    Self::Compound(_) => $crate::Tag::Compound,
                    Self::IntArray(_) => $crate::Tag::IntArray,
                    Self::LongArray(_) => $crate::Tag::LongArray,
                }
            }

            /// Returns whether this value is a number, i.e. a byte, short, int, long, float or double.
            #[must_use]
            pub const fn is_number(&self) -> bool {
                match self {
                    Self::Byte(_) | Self::Short(_) | Self::Int(_) | Self::Long(_) | Self::Float(_) | Self::Double(_) => true,
                    _ => false,
                }
            }

            as_number!(as_i8, i8, $($deref)*);
            as_number!(as_i16, i16, $($deref)*);
            as_number!(as_i32, i32, $($deref)*);
            as_number!(as_i64, i64, $($deref)*);
            as_number_float!(as_f32, f32, $($deref)*);
            as_number_float!(as_f64, f64, $($deref)*);

            /// If this value is a number, returns the `bool` representation of this value.
            #[must_use]
            pub fn as_bool(&self) -> ::std::option::Option<bool> {
                self.as_i8().map(|v| v != 0)
            }
        }

        impl <$($lifetime,)? S> ::std::convert::From<$($reference)* i8> for $name<$($lifetime,)? S> {
            fn from(v: $($reference)* i8) -> Self {
                Self::Byte(v)
            }
        }

        impl <$($lifetime,)? S> ::std::convert::From<$($reference)* i16> for $name<$($lifetime,)? S> {
            fn from(v: $($reference)* i16) -> Self {
                Self::Short(v)
            }
        }

        impl <$($lifetime,)? S> ::std::convert::From<$($reference)* i32> for $name<$($lifetime,)? S> {
            fn from(v: $($reference)* i32) -> Self {
                Self::Int(v)
            }
        }

        impl <$($lifetime,)? S> ::std::convert::From<$($reference)* i64> for $name<$($lifetime,)? S> {
            fn from(v: $($reference)* i64) -> Self {
                Self::Long(v)
            }
        }

        impl <$($lifetime,)? S> ::std::convert::From<$($reference)* f32> for $name<$($lifetime,)? S> {
            fn from(v: $($reference)* f32) -> Self {
                Self::Float(v)
            }
        }

        impl <$($lifetime,)? S> ::std::convert::From<$($reference)* f64> for $name<$($lifetime,)? S> {
            fn from(v: $($reference)* f64) -> Self {
                Self::Double(v)
            }
        }

        impl <$($lifetime,)? S> ::std::convert::From<$($reference)* $crate::List<S>> for $name<$($lifetime,)? S> {
            fn from(v: $($reference)* $crate::List<S>) -> Self {
                Self::List(v)
            }
        }

        impl <$($lifetime,)? S> ::std::convert::From<$($reference)* $crate::Compound<S>> for $name<$($lifetime,)? S> {
            fn from(v: $($reference)* $crate::Compound<S>) -> Self {
                Self::Compound(v)
            }
        }

        impl <$($lifetime,)? S> ::std::cmp::PartialEq<Self> for $name<$($lifetime,)? S> where S: ::std::cmp::Ord + ::std::hash::Hash {
            fn eq(&self, other: &Self) -> bool {
                match self {
                    Self::Byte(v) => matches!(other, Self::Byte(other_v) if v == other_v),
                    Self::Short(v) => matches!(other, Self::Short(other_v) if v == other_v),
                    Self::Int(v) => matches!(other, Self::Int(other_v) if v == other_v),
                    Self::Long(v) => matches!(other, Self::Long(other_v) if v == other_v),
                    Self::Float(v) => matches!(other, Self::Float(other_v) if v == other_v),
                    Self::Double(v) => matches!(other, Self::Double(other_v) if v == other_v),
                    Self::ByteArray(v) => matches!(other, Self::ByteArray(other_v) if v == other_v),
                    Self::String(v) => matches!(other, Self::String(other_v) if v == other_v),
                    Self::List(v) => matches!(other, Self::List(other_v) if v == other_v),
                    Self::Compound(v) => matches!(other, Self::Compound(other_v) if v == other_v),
                    Self::IntArray(v) => matches!(other, Self::IntArray(other_v) if v == other_v),
                    Self::LongArray(v) => matches!(other, Self::LongArray(other_v) if v == other_v),
                }
            }
        }
    }
}

impl_value!(Value,,(*),);
impl_value!(ValueRef, 'a, (**), &'a);
impl_value!(ValueMut, 'a, (**), &'a mut);
