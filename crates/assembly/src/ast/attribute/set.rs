use alloc::vec::Vec;
use core::{
	fmt::{Debug, Formatter, Result as FmtResult},
	slice,
};

use super::Attribute;
use crate::ast::Ident;

#[derive(Default, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct AttributeSet {
	attrs: Vec<Attribute>,
}

impl AttributeSet {
	#[must_use]
	pub const fn new() -> Self {
		Self { attrs: Vec::new() }
	}

	#[must_use]
	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			attrs: Vec::with_capacity(capacity),
		}
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.attrs.is_empty()
	}

	#[must_use]
	pub fn len(&self) -> usize {
		self.attrs.len()
	}

	pub fn has(&self, name: impl AsRef<str>) -> bool {
		self.get(name).is_some()
	}

	pub fn get(&self, name: impl AsRef<str>) -> Option<&Attribute> {
		let name = name.as_ref();
		self.attrs
			.binary_search_by_key(&name, |attr| attr.name())
			.map_or(None, |index| self.attrs.get(index))
	}

	pub fn get_mut(&mut self, name: impl AsRef<str>) -> Option<&mut Attribute> {
		let name = name.as_ref();
		self.attrs
			.binary_search_by_key(&name, |attr| attr.name())
			.map_or(None, |index| self.attrs.get_mut(index))
	}

	pub fn iter(&self) -> slice::Iter<'_, Attribute> {
		self.attrs.iter()
	}

	pub fn iter_mut(&mut self) -> slice::IterMut<'_, Attribute> {
		self.attrs.iter_mut()
	}

	pub fn insert(&mut self, attr: Attribute) -> bool {
		let name = attr.name();
		match self.attrs.binary_search_by_key(&name, |attr| attr.name()) {
			Ok(index) => {
				self.attrs[index] = attr;
				false
			}
			Err(index) => {
				self.attrs.insert(index, attr);
				true
			}
		}
	}

	pub fn insert_new(&mut self, attr: Attribute) -> Result<(), Attribute> {
		if self.has(attr.name()) {
			Err(attr)
		} else {
			self.insert(attr);
			Ok(())
		}
	}

	pub fn remove(&mut self, name: impl AsRef<str>) -> Option<Attribute> {
		let name = name.as_ref();
		self.attrs
			.binary_search_by_key(&name, |attr| attr.name())
			.map_or(None, |index| Some(self.attrs.remove(index)))
	}

	pub fn entry(&mut self, key: Ident) -> AttributeSetEntry<'_> {
		match self
			.attrs
			.binary_search_by_key(&key.as_str(), |attr| attr.name())
		{
			Ok(index) => AttributeSetEntry::occupied(self, index),
			Err(index) => AttributeSetEntry::vacant(self, key, index),
		}
	}

	pub fn clear(&mut self) {
		self.attrs.clear();
	}
}

impl Debug for AttributeSet {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let mut builder = f.debug_map();

		for attr in self {
			match attr.metadata() {
				None => builder.entry(&attr.name(), &"None"),
				Some(meta) => builder.entry(&attr.name(), &meta),
			};
		}

		builder.finish()
	}
}

impl Extend<Attribute> for AttributeSet {
	fn extend<T>(&mut self, iter: T)
	where
		T: IntoIterator<Item = Attribute>,
	{
		self.attrs.extend(iter);
	}
}

impl FromIterator<Attribute> for AttributeSet {
	fn from_iter<T>(iter: T) -> Self
	where
		T: IntoIterator<Item = Attribute>,
	{
		let mut this = Self {
			attrs: iter.into_iter().collect(),
		};
		this.attrs.sort_by_key(|attr| attr.id().clone());
		this.attrs.dedup_by_key(|attr| attr.id().clone());
		this
	}
}

impl<'a> IntoIterator for &'a AttributeSet {
	type IntoIter = slice::Iter<'a, Attribute>;
	type Item = &'a Attribute;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a> IntoIterator for &'a mut AttributeSet {
	type IntoIter = slice::IterMut<'a, Attribute>;
	type Item = &'a mut Attribute;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

pub struct AttributeSetOccupiedEntry<'a> {
	set: &'a mut AttributeSet,
	index: usize,
}

impl AttributeSetOccupiedEntry<'_> {
	#[must_use]
	pub fn get(&self) -> &Attribute {
		&self.set.attrs[self.index]
	}

	pub fn get_mut(&mut self) -> &mut Attribute {
		&mut self.set.attrs[self.index]
	}

	pub fn insert(self, attr: Attribute) {
		if attr.name() == self.get().name() {
			self.set.attrs[self.index] = attr;
		} else {
			self.set.insert(attr);
		}
	}

	#[must_use]
	pub fn remove(self) -> Attribute {
		self.set.attrs.remove(self.index)
	}
}

pub struct AttributeSetVacantEntry<'a> {
	set: &'a mut AttributeSet,
	key: Ident,
	index: usize,
}

impl AttributeSetVacantEntry<'_> {
	pub fn insert(self, attr: Attribute) {
		if &self.key == attr.id() {
			self.set.attrs.insert(self.index, attr);
		} else {
			self.set.insert(attr);
		}
	}
}

pub enum AttributeSetEntry<'a> {
	Occupied(AttributeSetOccupiedEntry<'a>),
	Vacant(AttributeSetVacantEntry<'a>),
}

impl<'a> AttributeSetEntry<'a> {
	fn occupied(set: &'a mut AttributeSet, index: usize) -> Self {
		Self::Occupied(AttributeSetOccupiedEntry { set, index })
	}

	fn vacant(set: &'a mut AttributeSet, key: Ident, index: usize) -> Self {
		Self::Vacant(AttributeSetVacantEntry { set, key, index })
	}
}
