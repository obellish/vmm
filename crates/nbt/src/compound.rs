use std::{
	borrow::{Borrow, Cow},
	fmt::{Debug, Formatter, Result as FmtResult},
	hash::Hash,
	ops::{Index, IndexMut},
};

use super::Value;

#[derive(Default, Clone)]
#[repr(transparent)]
pub struct Compound<S = String> {
	map: Map<S>,
}

#[allow(clippy::missing_const_for_fn)]
impl<S> Compound<S> {
	#[must_use]
	pub fn new() -> Self {
		Self { map: Map::new() }
	}

	#[must_use]
	pub fn with_capacity(cap: usize) -> Self {
		Self {
			#[cfg(not(feature = "preserve_order"))]
			map: {
				let _ = cap;
				Map::new()
			},
			#[cfg(feature = "preserve_order")]
			map: Map::with_capacity(cap),
		}
	}

	pub fn clear(&mut self) {
		self.map.clear();
	}
}

impl<S> Compound<S>
where
	S: Hash + Ord,
{
	pub fn get<Q>(&self, k: &Q) -> Option<&Value<S>>
	where
		Q: ?Sized + AsBorrowed<S>,
		<Q as AsBorrowed<S>>::Borrowed: Hash + Ord,
		S: Borrow<<Q as AsBorrowed<S>>::Borrowed>,
	{
		self.map.get(k.as_borrowed())
	}

	pub fn contains_key<Q>(&self, k: &Q) -> bool
	where
		Q: ?Sized + AsBorrowed<S>,
		<Q as AsBorrowed<S>>::Borrowed: Hash + Ord,
		S: Borrow<<Q as AsBorrowed<S>>::Borrowed>,
	{
		self.map.contains_key(k.as_borrowed())
	}

	pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut Value<S>>
	where
		Q: ?Sized + AsBorrowed<S>,
		<Q as AsBorrowed<S>>::Borrowed: Hash + Ord,
		S: Borrow<<Q as AsBorrowed<S>>::Borrowed>,
	{
		self.map.get_mut(k.as_borrowed())
	}

	pub fn get_key_value<Q>(&self, k: &Q) -> Option<(&S, &Value<S>)>
	where
		Q: ?Sized + AsBorrowed<S>,
		<Q as AsBorrowed<S>>::Borrowed: Hash + Ord,
		S: Borrow<<Q as AsBorrowed<S>>::Borrowed>,
	{
		self.map.get_key_value(k.as_borrowed())
	}

	pub fn insert(&mut self, k: impl Into<S>, v: impl Into<Value<S>>) -> Option<Value<S>> {
		self.map.insert(k.into(), v.into())
	}

	pub fn remove<Q>(&mut self, k: &Q) -> Option<Value<S>>
	where
		Q: ?Sized + AsBorrowed<S>,
		<Q as AsBorrowed<S>>::Borrowed: Hash + Ord,
		S: Borrow<<Q as AsBorrowed<S>>::Borrowed>,
	{
		#[cfg(feature = "preserve_order")]
		return self.swap_remove(k);
		#[cfg(not(feature = "preserve_order"))]
		return self.map.remove(k.as_borrowed());
	}

	#[cfg(feature = "preserve_order")]
	pub fn swap_remove<Q>(&mut self, k: &Q) -> Option<Value<S>>
	where
		Q: ?Sized + AsBorrowed<S>,
		<Q as AsBorrowed<S>>::Borrowed: Hash + Ord,
		S: Borrow<<Q as AsBorrowed<S>>::Borrowed>,
	{
		self.map.swap_remove(k.as_borrowed())
	}

	#[cfg(feature = "preserve_order")]
	pub fn shift_remove<Q>(&mut self, k: &Q) -> Option<Value<S>>
	where
		Q: ?Sized + AsBorrowed<S>,
		<Q as AsBorrowed<S>>::Borrowed: Hash + Ord,
		S: Borrow<<Q as AsBorrowed<S>>::Borrowed>,
	{
		self.map.shift_remove(k.as_borrowed())
	}

	pub fn remove_entry<Q>(&mut self, k: &Q) -> Option<(S, Value<S>)>
	where
		S: Borrow<Q>,
		Q: ?Sized + Hash + Ord,
	{
		#[cfg(feature = "preserve_order")]
		return self.swap_remove_entry(k);
		#[cfg(not(feature = "preserve_order"))]
		return self.map.remove_entry(k);
	}

	#[cfg(feature = "preserve_order")]
	pub fn swap_remove_entry<Q>(&mut self, k: &Q) -> Option<(S, Value<S>)>
	where
		S: Borrow<Q>,
		Q: ?Sized + Hash + Ord,
	{
		self.map.swap_remove_entry(k)
	}

	#[cfg(feature = "preserve_order")]
	pub fn shift_remove_entry<Q>(&mut self, k: &Q) -> Option<(S, Value<S>)>
	where
		S: Borrow<Q>,
		Q: ?Sized + Hash + Ord,
	{
		self.map.shift_remove_entry(k)
	}

	pub fn append(&mut self, other: &mut Self) {
		#[cfg(not(feature = "preserve_order"))]
		self.map.append(&mut other.map);

		#[cfg(feature = "preserve_order")]
		for (k, v) in std::mem::take(&mut other.map) {
			self.map.insert(k, v);
		}
	}

	pub fn entry(&mut self, k: impl Into<S>) -> Entry<'_, S> {
		#[cfg(not(feature = "preserve_order"))]
		use std::collections::btree_map::Entry as EntryImpl;

		#[cfg(feature = "preserve_order")]
		use indexmap::map::Entry as EntryImpl;

		match self.map.entry(k.into()) {
			EntryImpl::Vacant(ve) => Entry::Vacant(VacantEntry { entry: ve }),
			EntryImpl::Occupied(oe) => Entry::Occupied(OccupiedEntry { entry: oe }),
		}
	}

	#[must_use]
	pub fn len(&self) -> usize {
		self.map.len()
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.map.is_empty()
	}

	#[must_use]
	pub fn iter(&self) -> Iter<'_, S> {
		self.into_iter()
	}

	pub fn iter_mut(&mut self) -> IterMut<'_, S> {
		self.into_iter()
	}

	#[must_use]
	pub fn keys(&self) -> Keys<'_, S> {
		Keys {
			iter: self.map.keys(),
		}
	}

	#[must_use]
	pub fn values(&self) -> Values<'_, S> {
		Values {
			iter: self.map.values(),
		}
	}

	pub fn values_mut(&mut self) -> ValuesMut<'_, S> {
		ValuesMut {
			iter: self.map.values_mut(),
		}
	}

	pub fn retain(&mut self, f: impl FnMut(&S, &mut Value<S>) -> bool) {
		self.map.retain(f);
	}

	pub fn merge(&mut self, other: Self) {
		for (k, v) in other {
			match (self.entry(k), v) {
				(Entry::Occupied(mut oe), Value::Compound(other)) => {
					if let Value::Compound(this) = oe.get_mut() {
						this.merge(other);
					}
				}
				(Entry::Occupied(mut oe), value) => {
					oe.insert(value);
				}
				(Entry::Vacant(ve), value) => {
					ve.insert(value);
				}
			}
		}
	}
}

impl<S: Debug> Debug for Compound<S> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self.map, f)
	}
}

#[cfg(feature = "serde")]
impl<'de, S> serde::Deserialize<'de> for Compound<S>
where
	S: serde::Deserialize<'de> + Hash + Ord,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		Map::<S>::deserialize(deserializer).map(|map| Self { map })
	}
}

impl<S> Extend<(S, Value<S>)> for Compound<S>
where
	S: Hash + Ord,
{
	fn extend<T: IntoIterator<Item = (S, Value<S>)>>(&mut self, iter: T) {
		self.map.extend(iter);
	}
}

impl<S> FromIterator<(S, Value<S>)> for Compound<S>
where
	S: Hash + Ord,
{
	fn from_iter<T: IntoIterator<Item = (S, Value<S>)>>(iter: T) -> Self {
		Self {
			map: Map::from_iter(iter),
		}
	}
}

impl<S, Q> Index<&'_ Q> for Compound<S>
where
	S: Borrow<Q> + Hash + Ord,
	Q: ?Sized + Hash + Ord,
{
	type Output = Value<S>;

	fn index(&self, index: &'_ Q) -> &Self::Output {
		self.map.index(index)
	}
}

impl<S, Q> IndexMut<&'_ Q> for Compound<S>
where
	S: Borrow<Q> + Hash + Ord,
	Q: ?Sized + Hash + Ord,
{
	fn index_mut(&mut self, index: &'_ Q) -> &mut Self::Output {
		self.map.get_mut(index).expect("no entry found for key")
	}
}

impl<'a, S> IntoIterator for &'a Compound<S> {
	type IntoIter = Iter<'a, S>;
	type Item = (&'a S, &'a Value<S>);

	fn into_iter(self) -> Self::IntoIter {
		Iter {
			iter: self.map.iter(),
		}
	}
}

impl<'a, S> IntoIterator for &'a mut Compound<S> {
	type IntoIter = IterMut<'a, S>;
	type Item = (&'a S, &'a mut Value<S>);

	fn into_iter(self) -> Self::IntoIter {
		IterMut {
			iter: self.map.iter_mut(),
		}
	}
}

impl<S> IntoIterator for Compound<S> {
	type IntoIter = IntoIter<S>;
	type Item = (S, Value<S>);

	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			iter: self.map.into_iter(),
		}
	}
}

impl<S> PartialEq for Compound<S>
where
	S: Hash + Ord,
{
	fn eq(&self, other: &Self) -> bool {
		self.map == other.map
	}
}

#[cfg(feature = "serde")]
impl<Str> serde::Serialize for Compound<Str>
where
	Str: Hash + Ord + serde::Serialize,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.map.serialize(serializer)
	}
}

#[repr(transparent)]
pub struct VacantEntry<'a, S = String> {
	#[cfg(not(feature = "preserve_order"))]
	entry: std::collections::btree_map::VacantEntry<'a, S, Value<S>>,
	#[cfg(feature = "preserve_order")]
	entry: indexmap::map::VacantEntry<'a, S, Value<S>>,
}

impl<'a, S> VacantEntry<'a, S>
where
	S: Hash + Ord,
{
	pub fn key(&self) -> &S {
		self.entry.key()
	}

	pub fn insert(self, v: impl Into<Value<S>>) -> &'a mut Value<S> {
		self.entry.insert(v.into())
	}
}

impl<S> Debug for VacantEntry<'_, S>
where
	S: Debug + Ord,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("VacantEntry")
			.field("entry", &self.entry)
			.finish()
	}
}

#[repr(transparent)]
pub struct OccupiedEntry<'a, S = String> {
	#[cfg(not(feature = "preserve_order"))]
	entry: std::collections::btree_map::OccupiedEntry<'a, S, Value<S>>,
	#[cfg(feature = "preserve_order")]
	entry: indexmap::map::OccupiedEntry<'a, S, Value<S>>,
}

impl<'a, S> OccupiedEntry<'a, S>
where
	S: Hash + Ord,
{
	#[must_use]
	pub fn key(&self) -> &S {
		self.entry.key()
	}

	#[must_use]
	pub fn get(&self) -> &Value<S> {
		self.entry.get()
	}

	pub fn get_mut(&mut self) -> &mut Value<S> {
		self.entry.get_mut()
	}

	#[must_use]
	pub fn into_mut(self) -> &'a mut Value<S> {
		self.entry.into_mut()
	}

	pub fn insert(&mut self, v: impl Into<Value<S>>) -> Value<S> {
		self.entry.insert(v.into())
	}

	#[must_use]
	pub fn remove(self) -> Value<S> {
		#[cfg(feature = "preserve_order")]
		return self.swap_remove();
		#[cfg(not(feature = "preserve_order"))]
		return self.entry.remove();
	}

	#[cfg(feature = "preserve_order")]
	#[must_use]
	pub fn swap_remove(self) -> Value<S> {
		self.entry.swap_remove()
	}

	#[cfg(feature = "preserve_order")]
	#[must_use]
	pub fn shift_remove(self) -> Value<S> {
		self.entry.shift_remove()
	}
}

impl<S> Debug for OccupiedEntry<'_, S>
where
	S: Debug + Ord,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.debug_struct("OccupiedEntry")
			.field("entry", &self.entry)
			.finish()
	}
}

macro_rules! impl_iterator_traits {
    (($name:ident $($generics:tt)*) => $item:ty) => {
        impl $($generics)* ::std::iter::Iterator for $name $($generics)* {
            type Item = $item;
            #[inline]
            fn next(&mut self) -> ::std::option::Option<Self::Item> {
                self.iter.next()
            }
            #[inline]
            fn size_hint(&self) -> (usize, ::std::option::Option<usize>) {
                self.iter.size_hint()
            }
        }

        #[cfg(feature = "preserve_order")]
        impl $($generics)* ::std::iter::DoubleEndedIterator for $name $($generics)* {
            #[inline]
            fn next_back(&mut self) -> ::std::option::Option<Self::Item> {
                self.iter.next_back()
            }
        }

        impl $($generics)* ::std::iter::ExactSizeIterator for $name $($generics)* {
            #[inline]
            fn len(&self) -> usize {
                self.iter.len()
            }
        }

        impl $($generics)* ::std::iter::FusedIterator for $name $($generics)* {}
    }
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Iter<'a, S = String> {
	#[cfg(not(feature = "preserve_order"))]
	iter: std::collections::btree_map::Iter<'a, S, Value<S>>,
	#[cfg(feature = "preserve_order")]
	iter: indexmap::map::Iter<'a, S, Value<S>>,
}

impl_iterator_traits!((Iter<'a, S>) => (&'a S, &'a Value<S>));

#[derive(Debug)]
#[repr(transparent)]
pub struct IterMut<'a, S = String> {
	#[cfg(not(feature = "preserve_order"))]
	iter: std::collections::btree_map::IterMut<'a, S, Value<S>>,
	#[cfg(feature = "preserve_order")]
	iter: indexmap::map::IterMut<'a, S, Value<S>>,
}

impl_iterator_traits!((IterMut<'a, S>) => (&'a S, &'a mut Value<S>));

#[derive(Debug)]
#[repr(transparent)]
pub struct IntoIter<S = String> {
	#[cfg(not(feature = "preserve_order"))]
	iter: std::collections::btree_map::IntoIter<S, Value<S>>,
	#[cfg(feature = "preserve_order")]
	iter: indexmap::map::IntoIter<S, Value<S>>,
}

impl_iterator_traits!((IntoIter<S>) => (S, Value<S>));

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Keys<'a, S = String> {
	#[cfg(not(feature = "preserve_order"))]
	iter: std::collections::btree_map::Keys<'a, S, Value<S>>,
	#[cfg(feature = "preserve_order")]
	iter: indexmap::map::Keys<'a, S, Value<S>>,
}

impl_iterator_traits!((Keys<'a, S>) => &'a S);

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Values<'a, S = String> {
	#[cfg(not(feature = "preserve_order"))]
	iter: std::collections::btree_map::Values<'a, S, Value<S>>,
	#[cfg(feature = "preserve_order")]
	iter: indexmap::map::Values<'a, S, Value<S>>,
}

impl_iterator_traits!((Values<'a, S>) => &'a Value<S>);

#[derive(Debug)]
#[repr(transparent)]
pub struct ValuesMut<'a, S = String> {
	#[cfg(not(feature = "preserve_order"))]
	iter: std::collections::btree_map::ValuesMut<'a, S, Value<S>>,
	#[cfg(feature = "preserve_order")]
	iter: indexmap::map::ValuesMut<'a, S, Value<S>>,
}

impl_iterator_traits!((ValuesMut<'a, S>) => &'a mut Value<S>);

pub enum Entry<'a, S = String> {
	Vacant(VacantEntry<'a, S>),
	Occupied(OccupiedEntry<'a, S>),
}

impl<'a, S> Entry<'a, S>
where
	S: Hash + Ord,
{
	pub fn key(&self) -> &S {
		match self {
			Self::Vacant(v) => v.key(),
			Self::Occupied(o) => o.key(),
		}
	}

	pub fn or_insert(self, default: impl Into<Value<S>>) -> &'a mut Value<S> {
		match self {
			Self::Vacant(v) => v.insert(default),
			Self::Occupied(o) => o.into_mut(),
		}
	}

	pub fn or_insert_with<V>(self, default: impl FnOnce() -> V) -> &'a mut Value<S>
	where
		V: Into<Value<S>>,
	{
		match self {
			Self::Vacant(v) => v.insert(default()),
			Self::Occupied(o) => o.into_mut(),
		}
	}

	#[must_use]
	pub fn and_modify(self, f: impl FnOnce(&mut Value<S>)) -> Self {
		match self {
			Self::Vacant(v) => Self::Vacant(v),
			Self::Occupied(mut o) => {
				f(o.get_mut());
				Self::Occupied(o)
			}
		}
	}
}

impl<S> Debug for Entry<'_, S>
where
	S: Debug + Ord,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Vacant(v) => f.debug_tuple("Vacant").field(v).finish(),
			Self::Occupied(o) => f.debug_tuple("Occupied").field(o).finish(),
		}
	}
}

pub trait AsBorrowed<S> {
	type Borrowed: ?Sized;

	fn as_borrowed(&self) -> &Self::Borrowed;
}

impl<Q: ?Sized> AsBorrowed<String> for Q
where
	String: Borrow<Q>,
{
	type Borrowed = Q;

	fn as_borrowed(&self) -> &Self::Borrowed {
		self
	}
}

impl<'a, Q: ?Sized> AsBorrowed<Cow<'a, str>> for Q
where
	Cow<'a, str>: Borrow<Q>,
{
	type Borrowed = Q;

	fn as_borrowed(&self) -> &Self::Borrowed {
		self
	}
}

#[cfg(feature = "java_string")]
impl<Q: ?Sized> AsBorrowed<java_string::JavaString> for Q
where
	for<'a> &'a Q: Into<&'a java_string::JavaStr>,
{
	type Borrowed = java_string::JavaStr;

	fn as_borrowed(&self) -> &Self::Borrowed {
		self.into()
	}
}

#[cfg(feature = "java_string")]
impl<Q: ?Sized> AsBorrowed<Cow<'_, java_string::JavaStr>> for Q
where
	for<'a> &'a Q: Into<&'a java_string::JavaStr>,
{
	type Borrowed = java_string::JavaStr;

	fn as_borrowed(&self) -> &Self::Borrowed {
		self.into()
	}
}

#[cfg(not(feature = "preserve_order"))]
type Map<S> = std::collections::BTreeMap<S, Value<S>>;

#[cfg(feature = "preserve_order")]
type Map<S> = indexmap::IndexMap<S, Value<S>>;

#[cfg(all(feature = "preserve_order", test))]
mod tests {
	use super::Compound;

	#[test]
	fn compound_preserves_order() {
		let letters = ["g", "b", "d", "e", "h", "z", "m", "a", "q"];

		let mut c = Compound::<String>::new();
		for l in letters {
			c.insert(l, 0i8);
		}

		for (k, l) in c.keys().zip(letters) {
			assert_eq!(k, l);
		}
	}
}
