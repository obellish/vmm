#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod prelude;

use std::{
	any::Any,
	fmt::{Debug, Display, Formatter, Result as FmtResult, Write as _},
	hash::{Hash, Hasher},
	ops::{BitAnd, BitOr, Not},
	sync::LazyLock,
};

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_reflect::{
	ApplyError, GetTypeRegistration, OpaqueInfo, ReflectMut, ReflectOwned, ReflectRef, TypeInfo,
	TypePath, TypeRegistration, Typed, prelude::*, utility::NonGenericTypeInfoCell,
};
use bevy_utils::HashSet;
use serde::{Deserialize, Serialize};

#[macro_export]
macro_rules! tags {
    ($v0:vis $n0:ident, $($v:vis $n:ident),* $(,)?) => {
        $v0 const $n0: $crate::Tag = $crate::Tag::new(stringify!($n0));
        $crate::tags!($($v $n),*);
    };

    ($v:vis $name:ident) => {
        $v const $name: $crate::Tag = $crate::Tag::new(stringify!($name));
    };
}

pub static EMPTY_TAGS: LazyLock<Tags> = LazyLock::new(Tags::new);

pub struct TagPlugin;

impl Plugin for TagPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<Tag>()
			.register_type::<Tags>()
			.register_type::<HashSet<Tag>>()
			.register_type_data::<HashSet<Tag>, ReflectSerialize>()
			.register_type_data::<HashSet<Tag>, ReflectDeserialize>()
			.register_type::<TagFilter>()
			.register_type::<TagFilterDyn>()
			.register_type_data::<TagFilterDyn, ReflectSerialize>()
			.register_type_data::<TagFilterDyn, ReflectDeserialize>();
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[reflect(Hash, PartialEq)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Tag(u64);

impl Tag {
	#[must_use]
	pub const fn new(source: &str) -> Self {
		Self::from_hash(const_fnv1a_hash::fnv1a_hash_str_64(source))
	}

	#[must_use]
	pub const fn from_hash(hash: u64) -> Self {
		Self(hash)
	}

	#[must_use]
	pub const fn hash(self) -> u64 {
		self.0
	}
}

impl<T> BitOr<T> for Tag
where
	T: Into<TagFilter>,
{
	type Output = TagFilter;

	fn bitor(self, rhs: T) -> Self::Output {
		TagFilter::Eq(self.into()) | rhs
	}
}

impl Debug for Tag {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("Tag(")?;
		Display::fmt(&self.0, f)?;
		f.write_char(')')
	}
}

impl FromWorld for Tag {
	fn from_world(_: &mut World) -> Self {
		Self(u64::MAX)
	}
}

impl Hash for Tag {
	fn hash<H: Hasher>(&self, state: &mut H) {
		state.write_u64(self.0);
	}
}

impl Not for Tag {
	type Output = TagFilter;

	fn not(self) -> Self::Output {
		!TagFilter::Eq(self.into())
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Component, Reflect)]
#[reflect(Component)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Tags(HashSet<Tag>);

impl Tags {
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	#[must_use]
	pub fn len(&self) -> usize {
		self.0.len()
	}

	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	#[must_use]
	pub fn contains(&self, tag: Tag) -> bool {
		self.0.contains(&tag)
	}

	pub fn insert(&mut self, tag: Tag) -> bool {
		self.0.insert(tag)
	}

	pub fn remove(&mut self, tag: Tag) -> bool {
		self.0.remove(&tag)
	}

	pub fn iter(&self) -> impl Iterator<Item = Tag> + '_ {
		self.0.iter().copied()
	}

	#[must_use]
	pub fn union(mut self, other: impl IntoTags) -> Self {
		self.0.extend(other.into_tags());
		self
	}

	#[must_use]
	pub fn is_disjoint(&self, tags: &Self) -> bool {
		self.0.is_disjoint(&tags.0)
	}

	#[must_use]
	pub fn is_subset(&self, tags: &Self) -> bool {
		self.0.is_subset(&tags.0)
	}

	#[must_use]
	pub fn is_superset(&self, tags: &Self) -> bool {
		self.0.is_superset(&tags.0)
	}
}

impl From<Tag> for Tags {
	fn from(value: Tag) -> Self {
		let mut tags = Self::new();
		tags.insert(value);
		tags
	}
}

impl IntoIterator for Tags {
	type IntoIter = bevy_utils::hashbrown::hash_set::IntoIter<Self::Item>;
	type Item = Tag;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<const N: usize> PartialEq<[Tag; N]> for Tags {
	fn eq(&self, other: &[Tag; N]) -> bool {
		self.len() == N && other.iter().all(|tag| self.contains(*tag))
	}
}

impl<const N: usize> PartialEq<Tags> for [Tag; N] {
	fn eq(&self, other: &Tags) -> bool {
		other == self
	}
}

#[derive(Default, Clone, Serialize, Deserialize, Reflect)]
pub enum TagFilter {
	None,
	#[default]
	Any,
	Eq(Tags),
	All(Tags),
	Some(Tags),
	And(TagFilterDyn, TagFilterDyn),
	Or(TagFilterDyn, TagFilterDyn),
	Not(TagFilterDyn),
}

impl TagFilter {
	#[must_use]
	pub fn allows(&self, tags: &Tags) -> bool {
		match self {
			Self::None => false,
			Self::Any => true,
			Self::Eq(a) => a == tags,
			Self::All(a) => tags.is_subset(a),
			Self::Some(a) => !tags.is_disjoint(a),
			Self::And(a, b) => a.allows(tags) && b.allows(tags),
			Self::Or(a, b) => a.allows(tags) || b.allows(tags),
			Self::Not(a) => !a.allows(tags),
		}
	}
}

impl<T> BitAnd<T> for TagFilter
where
	T: Into<Self>,
{
	type Output = Self;

	fn bitand(self, rhs: T) -> Self::Output {
		Self::And(Box::new(self), Box::new(rhs.into()))
	}
}

impl<T> BitOr<T> for TagFilter
where
	T: Into<Self>,
{
	type Output = Self;

	fn bitor(self, rhs: T) -> Self::Output {
		Self::Or(Box::new(self), Box::new(rhs.into()))
	}
}

impl<T: IntoTags> From<T> for TagFilter {
	fn from(value: T) -> Self {
		Self::Eq(value.into_tags())
	}
}

impl Not for TagFilter {
	type Output = Self;

	fn not(self) -> Self::Output {
		Self::Not(Box::new(self))
	}
}

pub trait GetTags {
	fn tags(&self, entity: Entity) -> &Tags;
}

impl GetTags for World {
	fn tags(&self, entity: Entity) -> &Tags {
		self.get(entity).unwrap_or(&EMPTY_TAGS)
	}
}

pub trait GetEntityTags {
	fn tags(&self) -> &Tags;
}

impl GetEntityTags for EntityRef<'_> {
	fn tags(&self) -> &Tags {
		self.get().unwrap_or(&EMPTY_TAGS)
	}
}

impl GetEntityTags for EntityMut<'_> {
	fn tags(&self) -> &Tags {
		self.get().unwrap_or(&EMPTY_TAGS)
	}
}

impl GetEntityTags for EntityWorldMut<'_> {
	fn tags(&self) -> &Tags {
		self.get().unwrap_or(&EMPTY_TAGS)
	}
}

pub trait IntoTags {
	fn into_tags(self) -> Tags;
}

impl IntoTags for () {
	fn into_tags(self) -> Tags {
		Tags::new()
	}
}

impl IntoTags for Tag {
	fn into_tags(self) -> Tags {
		Tags::from(self)
	}
}

impl<const N: usize> IntoTags for [Tag; N] {
	fn into_tags(self) -> Tags {
		Tags(self.into_iter().flat_map(IntoTags::into_tags).collect())
	}
}

impl IntoTags for Tags {
	fn into_tags(self) -> Tags {
		self
	}
}

pub trait TagFilterOr {
	fn or(self, other: impl Into<TagFilter>) -> TagFilter;
}

impl<T> TagFilterOr for T
where
	T: Into<TagFilter>,
{
	fn or(self, other: impl Into<TagFilter>) -> TagFilter {
		self.into() | other.into()
	}
}

pub trait TagFilterAnd {
	fn and(self, other: impl Into<TagFilter>) -> TagFilter;
}

impl<T> TagFilterAnd for T
where
	T: Into<TagFilter>,
{
	fn and(self, other: impl Into<TagFilter>) -> TagFilter {
		self.into() & other.into()
	}
}

type TagFilterDyn = Box<TagFilter>;

impl FromReflect for TagFilterDyn {
	fn from_reflect(reflect: &dyn PartialReflect) -> Option<Self> {
		TagFilter::from_reflect(reflect).map(Self::new)
	}
}

impl GetTypeRegistration for TagFilterDyn {
	fn get_type_registration() -> TypeRegistration {
		TypeRegistration::of::<Self>()
	}
}

impl PartialReflect for TagFilterDyn {
	fn get_represented_type_info(&self) -> Option<&'static TypeInfo> {
		(**self).get_represented_type_info()
	}

	fn into_partial_reflect(self: Box<Self>) -> Box<dyn PartialReflect> {
		(*self).into_partial_reflect()
	}

	fn as_partial_reflect(&self) -> &dyn PartialReflect {
		(**self).as_partial_reflect()
	}

	fn as_partial_reflect_mut(&mut self) -> &mut dyn PartialReflect {
		(**self).as_partial_reflect_mut()
	}

	fn try_into_reflect(self: Box<Self>) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>> {
		(*self).try_into_reflect()
	}

	fn try_as_reflect(&self) -> Option<&dyn Reflect> {
		(**self).try_as_reflect()
	}

	fn try_as_reflect_mut(&mut self) -> Option<&mut dyn Reflect> {
		(**self).try_as_reflect_mut()
	}

	fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError> {
		(**self).try_apply(value)
	}

	fn reflect_ref(&self) -> ReflectRef<'_> {
		(**self).reflect_ref()
	}

	fn reflect_mut(&mut self) -> ReflectMut<'_> {
		(**self).reflect_mut()
	}

	fn reflect_owned(self: Box<Self>) -> ReflectOwned {
		(*self).reflect_owned()
	}

	fn clone_value(&self) -> Box<dyn PartialReflect> {
		(**self).clone_value()
	}
}

impl Reflect for TagFilterDyn {
	fn into_any(self: Box<Self>) -> Box<dyn Any> {
		(*self).into_any()
	}

	fn as_any(&self) -> &dyn Any {
		(**self).as_any()
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		(**self).as_any_mut()
	}

	fn into_reflect(self: Box<Self>) -> Box<dyn Reflect> {
		(*self).into_reflect()
	}

	fn as_reflect(&self) -> &dyn Reflect {
		(**self).as_reflect()
	}

	fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
		(**self).as_reflect_mut()
	}

	fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
		(**self).set(value)
	}
}

impl TypePath for TagFilterDyn {
	fn type_path() -> &'static str {
		TagFilter::type_path()
	}

	fn short_type_path() -> &'static str {
		TagFilter::short_type_path()
	}
}

impl Typed for TagFilterDyn {
	fn type_info() -> &'static TypeInfo {
		static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
		CELL.get_or_set(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
	}
}

#[must_use]
pub fn none() -> TagFilter {
	TagFilter::Eq(().into_tags())
}

#[must_use]
pub const fn any() -> TagFilter {
	TagFilter::Any
}

pub fn all(all: impl IntoTags) -> TagFilter {
	TagFilter::All(all.into_tags())
}

pub fn some(some: impl IntoTags) -> TagFilter {
	TagFilter::Some(some.into_tags())
}

pub fn not(not: impl Into<TagFilter>) -> TagFilter {
	!not.into()
}

pub fn matches(tags: impl IntoTags, filter: impl Into<TagFilter>) -> bool {
	filter.into().allows(&tags.into_tags())
}

#[cfg(test)]
mod tests {
	use super::*;

	tags!(A, B, C);

	#[test]
	fn match_eq() {
		assert_eq!(A, A);
		assert_ne!(A, B);
	}

	#[test]
	fn match_empty() {
		assert!(matches((), ()));
		assert!(matches((), any()));
		assert!(matches((), all(())));
		assert!(matches((), none()));
		assert!(matches((), any() | ()));
		assert!(matches((), any() | none()));
		assert!(matches((), any() & ()));
		assert!(matches((), any() & none()));

		assert!(!matches((), !TagFilter::from(())));
		assert!(!matches((), !any()));
		assert!(!matches((), !none()));
		assert!(!matches((), A));
		assert!(!matches((), [A, B]));
		assert!(!matches((), some(A)));
		assert!(!matches((), some([A, B])));
	}

	#[test]
	fn match_tag() {
		assert!(matches(A, A));
		assert!(matches(A, any()));
		assert!(matches(A, A | B));
		assert!(matches(A, B | A));
		assert!(matches(A, !none()));
		assert!(matches(A, !B));
		assert!(matches(A, !C));
		assert!(matches(A, some(A)));
		assert!(matches(A, some([A, B])));
		assert!(matches(B, some([A, B])));

		assert!(!matches(A, B));
		assert!(!matches(A, none()));
		assert!(!matches(A, B | C));
		assert!(!matches(A, !A));
		assert!(!matches(A, [A, B]));
		assert!(!matches(A, some([B, C])));
	}

	#[test]
	fn match_tags() {
		assert!(matches([A, B], [A, B]));
		assert!(matches([A, B], [B, A]));
		assert!(matches([A, B], any()));
		assert!(matches([A, B], [A, B].or([B, C])));
		assert!(matches([A, B], !none()));
		assert!(matches([A, B], !A));
		assert!(matches([A, B], !C));
		assert!(matches([A, B], not([B, C])));
		assert!(matches([A, B], some([A, B])));

		assert!(!matches([A, B], [A, C]));
		assert!(!matches([A, B], none()));
		assert!(!matches([A, B], [A, C].or([B, C])));
		assert!(!matches([A, B], A));
		assert!(!matches([A, B], [A, B, C]));
		assert!(!matches([A, B], some(C)));
	}
}
