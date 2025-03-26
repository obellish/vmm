use std::{borrow::Cow, hash::Hash, iter::FusedIterator};

use super::{
	Compound, Tag, Value,
	value::{ValueMut, ValueRef},
};

#[derive(Clone, Default, Debug)]
pub enum List<S = String> {
	#[default]
	End,
	Byte(Vec<i8>),
	Short(Vec<i16>),
	Int(Vec<i32>),
	Long(Vec<i64>),
	Float(Vec<f32>),
	Double(Vec<f64>),
	ByteArray(Vec<Vec<i8>>),
	String(Vec<S>),
	#[expect(clippy::enum_variant_names)]
	List(Vec<List<S>>),
	Compound(Vec<Compound<S>>),
	IntArray(Vec<Vec<i32>>),
	LongArray(Vec<Vec<i64>>),
}

impl<S> List<S> {
	#[must_use]
	pub const fn new() -> Self {
		Self::End
	}

	#[must_use]
	pub fn len(&self) -> usize {
		match self {
			Self::End => 0,
			Self::Byte(l) => l.len(),
			Self::Short(l) => l.len(),
			Self::Int(l) => l.len(),
			Self::Long(l) => l.len(),
			Self::Float(l) => l.len(),
			Self::Double(l) => l.len(),
			Self::ByteArray(l) => l.len(),
			Self::String(l) => l.len(),
			Self::List(l) => l.len(),
			Self::Compound(l) => l.len(),
			Self::IntArray(l) => l.len(),
			Self::LongArray(l) => l.len(),
		}
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		matches!(self.len(), 0)
	}

	#[must_use]
	pub const fn element_tag(&self) -> Tag {
		match self {
			Self::End => Tag::End,
			Self::Byte(_) => Tag::Byte,
			Self::Short(_) => Tag::Short,
			Self::Int(_) => Tag::Int,
			Self::Long(_) => Tag::Long,
			Self::Float(_) => Tag::Float,
			Self::Double(_) => Tag::Double,
			Self::ByteArray(_) => Tag::ByteArray,
			Self::String(_) => Tag::String,
			Self::List(_) => Tag::List,
			Self::Compound(_) => Tag::Compound,
			Self::IntArray(_) => Tag::IntArray,
			Self::LongArray(_) => Tag::LongArray,
		}
	}

	pub fn get(&self, index: usize) -> Option<ValueRef<'_, S>> {
		match self {
			Self::End => None,
			Self::Byte(list) => list.get(index).map(ValueRef::Byte),
			Self::Short(list) => list.get(index).map(ValueRef::Short),
			Self::Int(list) => list.get(index).map(ValueRef::Int),
			Self::Long(list) => list.get(index).map(ValueRef::Long),
			Self::Float(list) => list.get(index).map(ValueRef::Float),
			Self::Double(list) => list.get(index).map(ValueRef::Double),
			Self::ByteArray(list) => list.get(index).map(|arr| ValueRef::ByteArray(&arr[..])),
			Self::String(list) => list.get(index).map(ValueRef::String),
			Self::List(list) => list.get(index).map(ValueRef::List),
			Self::Compound(list) => list.get(index).map(ValueRef::Compound),
			Self::IntArray(list) => list.get(index).map(|arr| ValueRef::IntArray(&arr[..])),
			Self::LongArray(list) => list.get(index).map(|arr| ValueRef::LongArray(&arr[..])),
		}
	}

	pub fn get_mut(&mut self, index: usize) -> Option<ValueMut<'_, S>> {
		match self {
			Self::End => None,
			Self::Byte(list) => list.get_mut(index).map(ValueMut::Byte),
			Self::Short(list) => list.get_mut(index).map(ValueMut::Short),
			Self::Int(list) => list.get_mut(index).map(ValueMut::Int),
			Self::Long(list) => list.get_mut(index).map(ValueMut::Long),
			Self::Float(list) => list.get_mut(index).map(ValueMut::Float),
			Self::Double(list) => list.get_mut(index).map(ValueMut::Double),
			Self::ByteArray(list) => list.get_mut(index).map(ValueMut::ByteArray),
			Self::String(list) => list.get_mut(index).map(ValueMut::String),
			Self::List(list) => list.get_mut(index).map(ValueMut::List),
			Self::Compound(list) => list.get_mut(index).map(ValueMut::Compound),
			Self::IntArray(list) => list.get_mut(index).map(ValueMut::IntArray),
			Self::LongArray(list) => list.get_mut(index).map(ValueMut::LongArray),
		}
	}

	pub fn try_push(&mut self, value: impl Into<Value<S>>) -> bool {
		let value = value.into();
		match self {
			Self::End => {
				*self = Self::from(value);
				true
			}
			Self::Byte(list) => {
				if let Value::Byte(value) = value {
					list.push(value);
					true
				} else {
					false
				}
			}
			Self::Short(list) => {
				if let Value::Short(value) = value {
					list.push(value);
					true
				} else {
					false
				}
			}
			Self::Int(list) => {
				if let Value::Int(value) = value {
					list.push(value);
					true
				} else {
					false
				}
			}
			Self::Long(list) => {
				if let Value::Long(value) = value {
					list.push(value);
					true
				} else {
					false
				}
			}
			Self::Float(list) => {
				if let Value::Float(value) = value {
					list.push(value);
					true
				} else {
					false
				}
			}
			Self::Double(list) => {
				if let Value::Double(value) = value {
					list.push(value);
					true
				} else {
					false
				}
			}
			Self::ByteArray(list) => {
				if let Value::ByteArray(value) = value {
					list.push(value);
					true
				} else {
					false
				}
			}
			Self::String(list) => {
				if let Value::String(value) = value {
					list.push(value);
					true
				} else {
					false
				}
			}
			Self::List(list) => {
				if let Value::List(value) = value {
					list.push(value);
					true
				} else {
					false
				}
			}
			Self::Compound(list) => {
				if let Value::Compound(value) = value {
					list.push(value);
					true
				} else {
					false
				}
			}
			Self::IntArray(list) => {
				if let Value::IntArray(value) = value {
					list.push(value);
					true
				} else {
					false
				}
			}
			Self::LongArray(list) => {
				if let Value::LongArray(value) = value {
					list.push(value);
					true
				} else {
					false
				}
			}
		}
	}

	#[must_use]
	pub fn try_insert(&mut self, index: usize, value: impl Into<Value<S>>) -> bool {
		#[cold]
		#[inline(never)]
		fn assert_failed(index: usize, len: usize) -> ! {
			panic!("insertion index (is {index}) should be <= len (is {len})");
		}

		let value = value.into();

		match self {
			Self::End => {
				if index > 0 {
					assert_failed(index, 0);
				}
				*self = Self::from(value);
				true
			}
			Self::Byte(list) => {
				if let Value::Byte(value) = value {
					list.insert(index, value);
					true
				} else {
					false
				}
			}
			Self::Short(list) => {
				if let Value::Short(value) = value {
					list.insert(index, value);
					true
				} else {
					false
				}
			}
			Self::Int(list) => {
				if let Value::Int(value) = value {
					list.insert(index, value);
					true
				} else {
					false
				}
			}
			Self::Long(list) => {
				if let Value::Long(value) = value {
					list.insert(index, value);
					true
				} else {
					false
				}
			}
			Self::Float(list) => {
				if let Value::Float(value) = value {
					list.insert(index, value);
					true
				} else {
					false
				}
			}
			Self::Double(list) => {
				if let Value::Double(value) = value {
					list.insert(index, value);
					true
				} else {
					false
				}
			}
			Self::ByteArray(list) => {
				if let Value::ByteArray(value) = value {
					list.insert(index, value);
					true
				} else {
					false
				}
			}
			Self::String(list) => {
				if let Value::String(value) = value {
					list.insert(index, value);
					true
				} else {
					false
				}
			}
			Self::List(list) => {
				if let Value::List(value) = value {
					list.insert(index, value);
					true
				} else {
					false
				}
			}
			Self::Compound(list) => {
				if let Value::Compound(value) = value {
					list.insert(index, value);
					true
				} else {
					false
				}
			}
			Self::IntArray(list) => {
				if let Value::IntArray(value) = value {
					list.insert(index, value);
					true
				} else {
					false
				}
			}
			Self::LongArray(list) => {
				if let Value::LongArray(value) = value {
					list.insert(index, value);
					true
				} else {
					false
				}
			}
		}
	}

	#[track_caller]
	pub fn remove(&mut self, index: usize) -> Value<S> {
		#[cold]
		#[inline(never)]
		#[track_caller]
		fn assert_failed(index: usize, len: usize) -> ! {
			panic!("removal index (is {index}) should be < len (is {len})");
		}

		let removed = match self {
			Self::End => assert_failed(index, 0),
			Self::Byte(list) => Value::Byte(list.remove(index)),
			Self::Short(list) => Value::Short(list.remove(index)),
			Self::Int(list) => Value::Int(list.remove(index)),
			Self::Long(list) => Value::Long(list.remove(index)),
			Self::Float(list) => Value::Float(list.remove(index)),
			Self::Double(list) => Value::Double(list.remove(index)),
			Self::ByteArray(list) => Value::ByteArray(list.remove(index)),
			Self::String(list) => Value::String(list.remove(index)),
			Self::List(list) => Value::List(list.remove(index)),
			Self::Compound(list) => Value::Compound(list.remove(index)),
			Self::IntArray(list) => Value::IntArray(list.remove(index)),
			Self::LongArray(list) => Value::LongArray(list.remove(index)),
		};

		if self.is_empty() {
			*self = Self::End;
		}

		removed
	}

	pub fn retain(&mut self, mut f: impl FnMut(ValueMut<'_, S>) -> bool) {
		match self {
			Self::End => {}
			Self::Byte(list) => list.retain_mut(|v| f(ValueMut::Byte(v))),
			Self::Short(list) => list.retain_mut(|v| f(ValueMut::Short(v))),
			Self::Int(list) => list.retain_mut(|v| f(ValueMut::Int(v))),
			Self::Long(list) => list.retain_mut(|v| f(ValueMut::Long(v))),
			Self::Float(list) => list.retain_mut(|v| f(ValueMut::Float(v))),
			Self::Double(list) => list.retain_mut(|v| f(ValueMut::Double(v))),
			Self::ByteArray(list) => list.retain_mut(|v| f(ValueMut::ByteArray(v))),
			Self::String(list) => list.retain_mut(|v| f(ValueMut::String(v))),
			Self::List(list) => list.retain_mut(|v| f(ValueMut::List(v))),
			Self::Compound(list) => list.retain_mut(|v| f(ValueMut::Compound(v))),
			Self::IntArray(list) => list.retain_mut(|v| f(ValueMut::IntArray(v))),
			Self::LongArray(list) => list.retain_mut(|v| f(ValueMut::LongArray(v))),
		}

		if self.is_empty() {
			*self = Self::End;
		}
	}

	#[must_use]
	pub fn iter(&self) -> Iter<'_, S> {
		Iter {
			inner: match self {
				Self::End => IterInner::End,
				Self::Byte(list) => IterInner::Byte(list.iter()),
				Self::Short(list) => IterInner::Short(list.iter()),
				Self::Int(list) => IterInner::Int(list.iter()),
				Self::Long(list) => IterInner::Long(list.iter()),
				Self::Float(list) => IterInner::Float(list.iter()),
				Self::Double(list) => IterInner::Double(list.iter()),
				Self::ByteArray(list) => IterInner::ByteArray(list.iter()),
				Self::String(list) => IterInner::String(list.iter()),
				Self::List(list) => IterInner::List(list.iter()),
				Self::Compound(list) => IterInner::Compound(list.iter()),
				Self::IntArray(list) => IterInner::IntArray(list.iter()),
				Self::LongArray(list) => IterInner::LongArray(list.iter()),
			},
		}
	}

	pub fn iter_mut(&mut self) -> IterMut<'_, S> {
		IterMut {
			inner: match self {
				Self::End => IterMutInner::End,
				Self::Byte(list) => IterMutInner::Byte(list.iter_mut()),
				Self::Short(list) => IterMutInner::Short(list.iter_mut()),
				Self::Int(list) => IterMutInner::Int(list.iter_mut()),
				Self::Long(list) => IterMutInner::Long(list.iter_mut()),
				Self::Float(list) => IterMutInner::Float(list.iter_mut()),
				Self::Double(list) => IterMutInner::Double(list.iter_mut()),
				Self::ByteArray(list) => IterMutInner::ByteArray(list.iter_mut()),
				Self::String(list) => IterMutInner::String(list.iter_mut()),
				Self::List(list) => IterMutInner::List(list.iter_mut()),
				Self::Compound(list) => IterMutInner::Compound(list.iter_mut()),
				Self::IntArray(list) => IterMutInner::IntArray(list.iter_mut()),
				Self::LongArray(list) => IterMutInner::LongArray(list.iter_mut()),
			},
		}
	}
}

impl<S> From<Vec<i8>> for List<S> {
	fn from(value: Vec<i8>) -> Self {
		Self::Byte(value)
	}
}

impl<S> From<Vec<i16>> for List<S> {
	fn from(value: Vec<i16>) -> Self {
		Self::Short(value)
	}
}

impl<S> From<Vec<i32>> for List<S> {
	fn from(value: Vec<i32>) -> Self {
		Self::Int(value)
	}
}

impl<S> From<Vec<i64>> for List<S> {
	fn from(value: Vec<i64>) -> Self {
		Self::Long(value)
	}
}

impl<S> From<Vec<f32>> for List<S> {
	fn from(value: Vec<f32>) -> Self {
		Self::Float(value)
	}
}

impl<S> From<Vec<f64>> for List<S> {
	fn from(value: Vec<f64>) -> Self {
		Self::Double(value)
	}
}

impl<S> From<Vec<Vec<i8>>> for List<S> {
	fn from(value: Vec<Vec<i8>>) -> Self {
		Self::ByteArray(value)
	}
}

impl From<Vec<String>> for List<String> {
	fn from(value: Vec<String>) -> Self {
		Self::String(value)
	}
}

impl<'a> From<Vec<Cow<'a, str>>> for List<Cow<'a, str>> {
	fn from(value: Vec<Cow<'a, str>>) -> Self {
		Self::String(value)
	}
}

#[cfg(feature = "java_string")]
impl From<Vec<java_string::JavaString>> for List<java_string::JavaString> {
	fn from(value: Vec<java_string::JavaString>) -> Self {
		Self::String(value)
	}
}

#[cfg(feature = "java_string")]
impl<'a> From<Vec<Cow<'a, java_string::JavaStr>>> for List<Cow<'a, java_string::JavaStr>> {
	fn from(value: Vec<Cow<'a, java_string::JavaStr>>) -> Self {
		Self::String(value)
	}
}

impl<S> From<Vec<Self>> for List<S> {
	fn from(value: Vec<Self>) -> Self {
		Self::List(value)
	}
}

impl<S> From<Vec<Compound<S>>> for List<S> {
	fn from(value: Vec<Compound<S>>) -> Self {
		Self::Compound(value)
	}
}

impl<S> From<Vec<Vec<i32>>> for List<S> {
	fn from(value: Vec<Vec<i32>>) -> Self {
		Self::IntArray(value)
	}
}

impl<S> From<Vec<Vec<i64>>> for List<S> {
	fn from(value: Vec<Vec<i64>>) -> Self {
		Self::LongArray(value)
	}
}

impl<S> From<Value<S>> for List<S> {
	fn from(value: Value<S>) -> Self {
		match value {
			Value::Byte(v) => Self::Byte(vec![v]),
			Value::Short(v) => Self::Short(vec![v]),
			Value::Int(v) => Self::Int(vec![v]),
			Value::Long(v) => Self::Long(vec![v]),
			Value::Float(v) => Self::Float(vec![v]),
			Value::Double(v) => Self::Double(vec![v]),
			Value::ByteArray(v) => Self::ByteArray(vec![v]),
			Value::String(v) => Self::String(vec![v]),
			Value::List(v) => Self::List(vec![v]),
			Value::Compound(v) => Self::Compound(vec![v]),
			Value::IntArray(v) => Self::IntArray(vec![v]),
			Value::LongArray(v) => Self::LongArray(vec![v]),
		}
	}
}

impl<S> IntoIterator for List<S> {
	type IntoIter = IntoIter<S>;
	type Item = Value<S>;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			inner: match self {
				Self::End => IntoIterInner::End,
				Self::Byte(list) => IntoIterInner::Byte(list.into_iter()),
				Self::Short(list) => IntoIterInner::Short(list.into_iter()),
				Self::Int(list) => IntoIterInner::Int(list.into_iter()),
				Self::Long(list) => IntoIterInner::Long(list.into_iter()),
				Self::Float(list) => IntoIterInner::Float(list.into_iter()),
				Self::Double(list) => IntoIterInner::Double(list.into_iter()),
				Self::ByteArray(list) => IntoIterInner::ByteArray(list.into_iter()),
				Self::String(list) => IntoIterInner::String(list.into_iter()),
				Self::List(list) => IntoIterInner::List(list.into_iter()),
				Self::Compound(list) => IntoIterInner::Compound(list.into_iter()),
				Self::IntArray(list) => IntoIterInner::IntArray(list.into_iter()),
				Self::LongArray(list) => IntoIterInner::LongArray(list.into_iter()),
			},
		}
	}
}

impl<'a, S> IntoIterator for &'a List<S> {
	type IntoIter = Iter<'a, S>;
	type Item = ValueRef<'a, S>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, S> IntoIterator for &'a mut List<S> {
	type IntoIter = IterMut<'a, S>;
	type Item = ValueMut<'a, S>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

impl<S> PartialEq for List<S>
where
	S: Hash + Ord,
{
	fn eq(&self, other: &Self) -> bool {
		match self {
			Self::End => matches!(other, Self::End),
			Self::Byte(list) => matches!(other, Self::Byte(other_list) if list == other_list),
			Self::Short(list) => matches!(other, Self::Short(other_list) if list == other_list),
			Self::Int(list) => matches!(other, Self::Int(other_list) if list == other_list),
			Self::Long(list) => matches!(other, Self::Long(other_list) if list == other_list),
			Self::Float(list) => matches!(other, Self::Float(other_list) if list == other_list),
			Self::Double(list) => matches!(other, Self::Double(other_list) if list == other_list),
			Self::ByteArray(list) => {
				matches!(other, Self::ByteArray(other_list) if list == other_list)
			}
			Self::String(list) => matches!(other, Self::String(other_list) if list == other_list),
			Self::List(list) => matches!(other, Self::List(other_list) if list == other_list),
			Self::Compound(list) => {
				matches!(other, Self::Compound(other_list) if list == other_list)
			}
			Self::IntArray(list) => {
				matches!(other, Self::IntArray(other_list) if list == other_list)
			}
			Self::LongArray(list) => {
				matches!(other, Self::LongArray(other_list) if list == other_list)
			}
		}
	}
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct IntoIter<S = String> {
	inner: IntoIterInner<S>,
}

impl<S> DoubleEndedIterator for IntoIter<S> {
	fn next_back(&mut self) -> Option<Self::Item> {
		match &mut self.inner {
			IntoIterInner::End => None,
			IntoIterInner::Byte(i) => i.next_back().map(Value::Byte),
			IntoIterInner::Short(i) => i.next_back().map(Value::Short),
			IntoIterInner::Int(i) => i.next_back().map(Value::Int),
			IntoIterInner::Long(i) => i.next_back().map(Value::Long),
			IntoIterInner::Float(i) => i.next_back().map(Value::Float),
			IntoIterInner::Double(i) => i.next_back().map(Value::Double),
			IntoIterInner::ByteArray(i) => i.next_back().map(Value::ByteArray),
			IntoIterInner::String(i) => i.next_back().map(Value::String),
			IntoIterInner::List(i) => i.next_back().map(Value::List),
			IntoIterInner::Compound(i) => i.next_back().map(Value::Compound),
			IntoIterInner::IntArray(i) => i.next_back().map(Value::IntArray),
			IntoIterInner::LongArray(i) => i.next_back().map(Value::LongArray),
		}
	}
}

impl<S> ExactSizeIterator for IntoIter<S> {
	fn len(&self) -> usize {
		match &self.inner {
			IntoIterInner::End => 0,
			IntoIterInner::Byte(i) => i.len(),
			IntoIterInner::Short(i) => i.len(),
			IntoIterInner::Int(i) => i.len(),
			IntoIterInner::Long(i) => i.len(),
			IntoIterInner::Float(i) => i.len(),
			IntoIterInner::Double(i) => i.len(),
			IntoIterInner::ByteArray(i) => i.len(),
			IntoIterInner::String(i) => i.len(),
			IntoIterInner::List(i) => i.len(),
			IntoIterInner::Compound(i) => i.len(),
			IntoIterInner::IntArray(i) => i.len(),
			IntoIterInner::LongArray(i) => i.len(),
		}
	}
}

impl<S> FusedIterator for IntoIter<S> {}

impl<S> Iterator for IntoIter<S> {
	type Item = Value<S>;

	fn next(&mut self) -> Option<Self::Item> {
		match &mut self.inner {
			IntoIterInner::End => None,
			IntoIterInner::Byte(i) => i.next().map(Value::Byte),
			IntoIterInner::Short(i) => i.next().map(Value::Short),
			IntoIterInner::Int(i) => i.next().map(Value::Int),
			IntoIterInner::Long(i) => i.next().map(Value::Long),
			IntoIterInner::Float(i) => i.next().map(Value::Float),
			IntoIterInner::Double(i) => i.next().map(Value::Double),
			IntoIterInner::ByteArray(i) => i.next().map(Value::ByteArray),
			IntoIterInner::String(i) => i.next().map(Value::String),
			IntoIterInner::List(i) => i.next().map(Value::List),
			IntoIterInner::Compound(i) => i.next().map(Value::Compound),
			IntoIterInner::IntArray(i) => i.next().map(Value::IntArray),
			IntoIterInner::LongArray(i) => i.next().map(Value::LongArray),
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		match &self.inner {
			IntoIterInner::End => (0, Some(0)),
			IntoIterInner::Byte(i) => i.size_hint(),
			IntoIterInner::Short(i) => i.size_hint(),
			IntoIterInner::Int(i) => i.size_hint(),
			IntoIterInner::Long(i) => i.size_hint(),
			IntoIterInner::Float(i) => i.size_hint(),
			IntoIterInner::Double(i) => i.size_hint(),
			IntoIterInner::ByteArray(i) => i.size_hint(),
			IntoIterInner::String(i) => i.size_hint(),
			IntoIterInner::List(i) => i.size_hint(),
			IntoIterInner::Compound(i) => i.size_hint(),
			IntoIterInner::IntArray(i) => i.size_hint(),
			IntoIterInner::LongArray(i) => i.size_hint(),
		}
	}
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Iter<'a, S = String> {
	inner: IterInner<'a, S>,
}

impl<S> DoubleEndedIterator for Iter<'_, S> {
	fn next_back(&mut self) -> Option<Self::Item> {
		match &mut self.inner {
			IterInner::End => None,
			IterInner::Byte(i) => i.next_back().map(ValueRef::Byte),
			IterInner::Short(i) => i.next_back().map(ValueRef::Short),
			IterInner::Int(i) => i.next_back().map(ValueRef::Int),
			IterInner::Long(i) => i.next_back().map(ValueRef::Long),
			IterInner::Float(i) => i.next_back().map(ValueRef::Float),
			IterInner::Double(i) => i.next_back().map(ValueRef::Double),
			IterInner::ByteArray(i) => i.next_back().map(|arr| ValueRef::ByteArray(&arr[..])),
			IterInner::String(i) => i.next_back().map(ValueRef::String),
			IterInner::List(i) => i.next_back().map(ValueRef::List),
			IterInner::Compound(i) => i.next_back().map(ValueRef::Compound),
			IterInner::IntArray(i) => i.next_back().map(|arr| ValueRef::IntArray(&arr[..])),
			IterInner::LongArray(i) => i.next_back().map(|arr| ValueRef::LongArray(&arr[..])),
		}
	}
}

impl<S> ExactSizeIterator for Iter<'_, S> {
	fn len(&self) -> usize {
		match &self.inner {
			IterInner::End => 0,
			IterInner::Byte(i) => i.len(),
			IterInner::Short(i) => i.len(),
			IterInner::Int(i) => i.len(),
			IterInner::Long(i) => i.len(),
			IterInner::Float(i) => i.len(),
			IterInner::Double(i) => i.len(),
			IterInner::ByteArray(i) => i.len(),
			IterInner::String(i) => i.len(),
			IterInner::List(i) => i.len(),
			IterInner::Compound(i) => i.len(),
			IterInner::IntArray(i) => i.len(),
			IterInner::LongArray(i) => i.len(),
		}
	}
}

impl<S> FusedIterator for Iter<'_, S> {}

impl<'a, S> Iterator for Iter<'a, S> {
	type Item = ValueRef<'a, S>;

	fn next(&mut self) -> Option<Self::Item> {
		match &mut self.inner {
			IterInner::End => None,
			IterInner::Byte(i) => i.next().map(ValueRef::Byte),
			IterInner::Short(i) => i.next().map(ValueRef::Short),
			IterInner::Int(i) => i.next().map(ValueRef::Int),
			IterInner::Long(i) => i.next().map(ValueRef::Long),
			IterInner::Float(i) => i.next().map(ValueRef::Float),
			IterInner::Double(i) => i.next().map(ValueRef::Double),
			IterInner::ByteArray(i) => i.next().map(|arr| ValueRef::ByteArray(&arr[..])),
			IterInner::String(i) => i.next().map(ValueRef::String),
			IterInner::List(i) => i.next().map(ValueRef::List),
			IterInner::Compound(i) => i.next().map(ValueRef::Compound),
			IterInner::IntArray(i) => i.next().map(|arr| ValueRef::IntArray(&arr[..])),
			IterInner::LongArray(i) => i.next().map(|arr| ValueRef::LongArray(&arr[..])),
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		match &self.inner {
			IterInner::End => (0, Some(0)),
			IterInner::Byte(i) => i.size_hint(),
			IterInner::Short(i) => i.size_hint(),
			IterInner::Int(i) => i.size_hint(),
			IterInner::Long(i) => i.size_hint(),
			IterInner::Float(i) => i.size_hint(),
			IterInner::Double(i) => i.size_hint(),
			IterInner::ByteArray(i) => i.size_hint(),
			IterInner::String(i) => i.size_hint(),
			IterInner::List(i) => i.size_hint(),
			IterInner::Compound(i) => i.size_hint(),
			IterInner::IntArray(i) => i.size_hint(),
			IterInner::LongArray(i) => i.size_hint(),
		}
	}
}

#[derive(Debug)]
#[repr(transparent)]
pub struct IterMut<'a, S = String> {
	inner: IterMutInner<'a, S>,
}

impl<S> DoubleEndedIterator for IterMut<'_, S> {
	fn next_back(&mut self) -> Option<Self::Item> {
		match &mut self.inner {
			IterMutInner::End => None,
			IterMutInner::Byte(i) => i.next_back().map(ValueMut::Byte),
			IterMutInner::Short(i) => i.next_back().map(ValueMut::Short),
			IterMutInner::Int(i) => i.next_back().map(ValueMut::Int),
			IterMutInner::Long(i) => i.next_back().map(ValueMut::Long),
			IterMutInner::Float(i) => i.next_back().map(ValueMut::Float),
			IterMutInner::Double(i) => i.next_back().map(ValueMut::Double),
			IterMutInner::ByteArray(i) => i.next_back().map(ValueMut::ByteArray),
			IterMutInner::String(i) => i.next_back().map(ValueMut::String),
			IterMutInner::List(i) => i.next_back().map(ValueMut::List),
			IterMutInner::Compound(i) => i.next_back().map(ValueMut::Compound),
			IterMutInner::IntArray(i) => i.next_back().map(ValueMut::IntArray),
			IterMutInner::LongArray(i) => i.next_back().map(ValueMut::LongArray),
		}
	}
}

impl<S> ExactSizeIterator for IterMut<'_, S> {
	fn len(&self) -> usize {
		match &self.inner {
			IterMutInner::End => 0,
			IterMutInner::Byte(i) => i.len(),
			IterMutInner::Short(i) => i.len(),
			IterMutInner::Int(i) => i.len(),
			IterMutInner::Long(i) => i.len(),
			IterMutInner::Float(i) => i.len(),
			IterMutInner::Double(i) => i.len(),
			IterMutInner::ByteArray(i) => i.len(),
			IterMutInner::String(i) => i.len(),
			IterMutInner::List(i) => i.len(),
			IterMutInner::Compound(i) => i.len(),
			IterMutInner::IntArray(i) => i.len(),
			IterMutInner::LongArray(i) => i.len(),
		}
	}
}

impl<S> FusedIterator for IterMut<'_, S> {}

impl<'a, S> Iterator for IterMut<'a, S> {
	type Item = ValueMut<'a, S>;

	fn next(&mut self) -> Option<Self::Item> {
		match &mut self.inner {
			IterMutInner::End => None,
			IterMutInner::Byte(i) => i.next().map(ValueMut::Byte),
			IterMutInner::Short(i) => i.next().map(ValueMut::Short),
			IterMutInner::Int(i) => i.next().map(ValueMut::Int),
			IterMutInner::Long(i) => i.next().map(ValueMut::Long),
			IterMutInner::Float(i) => i.next().map(ValueMut::Float),
			IterMutInner::Double(i) => i.next().map(ValueMut::Double),
			IterMutInner::ByteArray(i) => i.next().map(ValueMut::ByteArray),
			IterMutInner::String(i) => i.next().map(ValueMut::String),
			IterMutInner::List(i) => i.next().map(ValueMut::List),
			IterMutInner::Compound(i) => i.next().map(ValueMut::Compound),
			IterMutInner::IntArray(i) => i.next().map(ValueMut::IntArray),
			IterMutInner::LongArray(i) => i.next().map(ValueMut::LongArray),
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		match &self.inner {
			IterMutInner::End => (0, Some(0)),
			IterMutInner::Byte(i) => i.size_hint(),
			IterMutInner::Short(i) => i.size_hint(),
			IterMutInner::Int(i) => i.size_hint(),
			IterMutInner::Long(i) => i.size_hint(),
			IterMutInner::Float(i) => i.size_hint(),
			IterMutInner::Double(i) => i.size_hint(),
			IterMutInner::ByteArray(i) => i.size_hint(),
			IterMutInner::String(i) => i.size_hint(),
			IterMutInner::List(i) => i.size_hint(),
			IterMutInner::Compound(i) => i.size_hint(),
			IterMutInner::IntArray(i) => i.size_hint(),
			IterMutInner::LongArray(i) => i.size_hint(),
		}
	}
}

#[derive(Debug, Clone)]
enum IntoIterInner<S> {
	End,
	Byte(std::vec::IntoIter<i8>),
	Short(std::vec::IntoIter<i16>),
	Int(std::vec::IntoIter<i32>),
	Long(std::vec::IntoIter<i64>),
	Float(std::vec::IntoIter<f32>),
	Double(std::vec::IntoIter<f64>),
	ByteArray(std::vec::IntoIter<Vec<i8>>),
	String(std::vec::IntoIter<S>),
	List(std::vec::IntoIter<List<S>>),
	Compound(std::vec::IntoIter<Compound<S>>),
	IntArray(std::vec::IntoIter<Vec<i32>>),
	LongArray(std::vec::IntoIter<Vec<i64>>),
}

#[derive(Debug, Clone)]
enum IterInner<'a, S> {
	End,
	Byte(std::slice::Iter<'a, i8>),
	Short(std::slice::Iter<'a, i16>),
	Int(std::slice::Iter<'a, i32>),
	Long(std::slice::Iter<'a, i64>),
	Float(std::slice::Iter<'a, f32>),
	Double(std::slice::Iter<'a, f64>),
	ByteArray(std::slice::Iter<'a, Vec<i8>>),
	String(std::slice::Iter<'a, S>),
	List(std::slice::Iter<'a, List<S>>),
	Compound(std::slice::Iter<'a, Compound<S>>),
	IntArray(std::slice::Iter<'a, Vec<i32>>),
	LongArray(std::slice::Iter<'a, Vec<i64>>),
}

#[derive(Debug)]
enum IterMutInner<'a, S> {
	End,
	Byte(std::slice::IterMut<'a, i8>),
	Short(std::slice::IterMut<'a, i16>),
	Int(std::slice::IterMut<'a, i32>),
	Long(std::slice::IterMut<'a, i64>),
	Float(std::slice::IterMut<'a, f32>),
	Double(std::slice::IterMut<'a, f64>),
	ByteArray(std::slice::IterMut<'a, Vec<i8>>),
	String(std::slice::IterMut<'a, S>),
	List(std::slice::IterMut<'a, List<S>>),
	Compound(std::slice::IterMut<'a, Compound<S>>),
	IntArray(std::slice::IterMut<'a, Vec<i32>>),
	LongArray(std::slice::IterMut<'a, Vec<i64>>),
}
