use alloc::{
	borrow::Cow,
	boxed::Box,
	string::{String, ToString as _},
};
use core::fmt::{Display, Formatter, Result as FmtResult, Write as _};

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use tap::prelude::*;

use super::{FunctionName, ValueName};
use crate::{PrimitiveTypes, ast::GetName};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct TypeName(Cow<'static, str>);

impl Display for TypeName {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(&self.0)
	}
}

impl From<String> for TypeName {
	fn from(value: String) -> Self {
		Self(Cow::Owned(value))
	}
}

impl From<&'static str> for TypeName {
	fn from(value: &'static str) -> Self {
		Self(Cow::Borrowed(value))
	}
}

impl From<Cow<'static, str>> for TypeName {
	fn from(value: Cow<'static, str>) -> Self {
		Self(value)
	}
}

impl GetName for TypeName {
	fn name(&self) -> Cow<'static, str> {
		self.0.clone()
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructTypes {
	pub name: Cow<'static, str>,
	pub attributes: HashMap<ValueName, StructAttributeType>,
	pub methods: HashMap<String, FunctionName>,
}

impl From<crate::ast::StructTypes<'_>> for StructTypes {
	fn from(value: crate::ast::StructTypes<'_>) -> Self {
		Self {
			name: value.name(),
			attributes: {
				let mut res = HashMap::new();
				for (index, val) in value.attributes.iter().enumerate() {
					let name = val.name().into_owned();
					let mut v = val.clone().convert::<StructAttributeType>();
					v.index = index as u32;
					res.insert(name.into(), v);
				}
				res
			},
			methods: HashMap::new(),
		}
	}
}

impl TypeAttributes for StructTypes {
	fn get_attribute_index(&self, attr_name: &ValueName) -> Option<u32> {
		self.attributes.get(attr_name).map(|attr| attr.index)
	}

	fn get_attribute_type(&self, attr_name: &ValueName) -> Option<Type> {
		self.attributes.get(attr_name).map(|attr| attr.ty.clone())
	}

	fn get_method<Q>(&self, method_name: Q) -> Option<FunctionName>
	where
		Q: AsRef<str>,
	{
		self.methods.get(method_name.as_ref()).cloned()
	}

	fn is_attribute(&self, name: &ValueName) -> bool {
		self.attributes.contains_key(name)
	}

	fn is_method<Q>(&self, name: Q) -> bool
	where
		Q: AsRef<str>,
	{
		self.methods.contains_key(name.as_ref())
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructAttributeType {
	pub name: ValueName,
	pub ty: Type,
	pub index: u32,
}

impl From<crate::ast::StructType<'_>> for StructAttributeType {
	fn from(value: crate::ast::StructType<'_>) -> Self {
		Self {
			name: value.name().into(),
			ty: value.attr_type.into(),
			index: 0,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
	Primitive(PrimitiveTypes),
	Struct(StructTypes),
	Array(Box<Self>, u32),
}

impl Type {
	#[must_use]
	pub fn name(&self) -> TypeName {
		self.to_string().into()
	}

	#[must_use]
	pub fn as_struct(&self) -> Option<StructTypes> {
		let Self::Struct(s) = self else {
			return None;
		};

		Some(s.clone())
	}
}

impl Display for Type {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Primitive(p) => Display::fmt(&p, f),
			Self::Struct(s) => f.write_str(&s.name),
			Self::Array(array_type, size) => {
				f.write_char('[')?;
				Display::fmt(&**array_type, f)?;
				f.write_char(';')?;
				Display::fmt(&size, f)?;
				f.write_char(']')
			}
		}
	}
}

impl From<crate::ast::Type<'_>> for Type {
	fn from(value: crate::ast::Type<'_>) -> Self {
		match value {
			crate::ast::Type::Array(v, s) => Self::Array(Box::new((*v).into()), s),
			crate::ast::Type::Primitive(p) => Self::Primitive(p),
			crate::ast::Type::Struct(s) => Self::Struct(s.into()),
		}
	}
}

impl TypeAttributes for Type {
	fn get_attribute_index(&self, attr_name: &ValueName) -> Option<u32> {
		match self {
			Self::Struct(s) => s.get_attribute_index(attr_name),
			_ => None,
		}
	}

	fn get_attribute_type(&self, attr_name: &ValueName) -> Option<Type> {
		match self {
			Self::Struct(s) => s.get_attribute_type(attr_name),
			_ => None,
		}
	}

	fn get_method<Q>(&self, method_name: Q) -> Option<FunctionName>
	where
		Q: AsRef<str>,
	{
		match self {
			Self::Struct(s) => s.get_method(method_name),
			_ => None,
		}
	}

	fn is_attribute(&self, name: &ValueName) -> bool {
		match self {
			Self::Struct(s) => s.is_attribute(name),
			_ => false,
		}
	}

	fn is_method<Q>(&self, name: Q) -> bool
	where
		Q: AsRef<str>,
	{
		match self {
			Self::Struct(s) => s.is_method(name),
			_ => false,
		}
	}
}

pub trait TypeAttributes {
	fn get_attribute_index(&self, attr_name: &ValueName) -> Option<u32>;

	fn get_attribute_type(&self, attr_name: &ValueName) -> Option<Type>;

	fn get_method<Q>(&self, method_name: Q) -> Option<FunctionName>
	where
		Q: AsRef<str>;

	fn is_attribute(&self, name: &ValueName) -> bool;

	fn is_method<Q>(&self, name: Q) -> bool
	where
		Q: AsRef<str>;
}
