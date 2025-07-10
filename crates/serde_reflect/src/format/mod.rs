mod named;
mod not_implemented;
// mod serde;

use std::{
	cell::{Ref, RefCell, RefMut},
	collections::{BTreeMap, btree_map::Entry},
	fmt::Debug,
	mem,
	ops::DerefMut,
	rc::Rc,
};

use ::serde::{
	Deserialize, Deserializer, Serialize, Serializer,
	ser::{SerializeMap, SerializeStruct},
};

pub use self::named::*;
use super::{Error, Result};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Variable<T>(Rc<RefCell<Option<T>>>);

impl<T> Variable<T> {
	pub(crate) fn new(content: Option<T>) -> Self {
		Self(Rc::new(RefCell::new(content)))
	}

	#[must_use]
	pub fn borrow(&self) -> Ref<'_, Option<T>> {
		self.0.as_ref().borrow()
	}

	#[must_use]
	pub fn borrow_mut(&self) -> RefMut<'_, Option<T>> {
		self.0.as_ref().borrow_mut()
	}

	fn into_inner(self) -> Option<T>
	where
		T: Clone,
	{
		match Rc::try_unwrap(self.0) {
			Ok(cell) => cell.into_inner(),
			Err(rc) => rc.borrow().clone(),
		}
	}
}

impl<T> FormatHolder for Variable<T>
where
	T: Clone + Debug + FormatHolder,
{
	fn visit<'a>(&'a self, _: &mut dyn FnMut(&'a Format) -> Result<()>) -> Result<()> {
		Err(Error::NotSupported(
			"cannot immutably visit formats with variables",
		))
	}

	fn visit_mut(&mut self, f: &mut dyn FnMut(&mut Format) -> Result<()>) -> Result<()> {
		match self.borrow_mut().deref_mut() {
			None => Err(Error::UnknownFormat),
			Some(value) => value.visit_mut(f),
		}
	}

	fn unify(&mut self, _: Self) -> Result<()> {
		Err(Error::NotSupported("cannot unify variables directly"))
	}

	fn is_unknown(&self) -> bool {
		match self.borrow().as_ref() {
			None => true,
			Some(format) => format.is_unknown(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Format {
	Variable(#[serde(with = "not_implemented")] Variable<Self>),
	TypeName(String),
	Unit,
	Bool,
	I8,
	I16,
	I32,
	I64,
	I128,
	U8,
	U16,
	U32,
	U64,
	U128,
	F32,
	F64,
	Char,
	Str,
	Bytes,
	Option(Box<Self>),
	Seq(Box<Self>),
	#[serde(rename_all = "UPPERCASE")]
	Map {
		key: Box<Self>,
		value: Box<Self>,
	},
	Tuple(Vec<Self>),
	#[serde(rename_all = "UPPERCASE")]
	TupleArray {
		content: Box<Self>,
		size: usize,
	},
}

impl Format {
	#[must_use]
	pub fn unknown() -> Self {
		Self::Variable(Variable::new(None))
	}
}

impl Default for Format {
	fn default() -> Self {
		Self::unknown()
	}
}

impl FormatHolder for Format {
	fn visit<'a>(&'a self, f: &mut dyn FnMut(&'a Format) -> Result<()>) -> Result<()> {
		match self {
			Self::Variable(variable) => variable.visit(f)?,
			Self::Option(format)
			| Self::Seq(format)
			| Self::TupleArray {
				content: format, ..
			} => format.visit(f)?,
			Self::Map { key, value } => {
				key.visit(f)?;
				value.visit(f)?;
			}
			Self::Tuple(formats) => {
				formats.iter().try_for_each(|format| format.visit(f))?;
			}
			_ => {}
		}

		f(self)
	}

	fn visit_mut(&mut self, f: &mut dyn FnMut(&mut Format) -> Result<()>) -> Result<()> {
		match self {
			Self::Variable(v) => {
				v.visit_mut(f)?;

				*self = mem::take(v).into_inner().expect("variable is known");
			}
			Self::Option(format)
			| Self::Seq(format)
			| Self::TupleArray {
				content: format, ..
			} => format.visit_mut(f)?,
			Self::Map { key, value } => {
				key.visit_mut(f)?;
				value.visit_mut(f)?;
			}
			Self::Tuple(formats) => {
				formats
					.iter_mut()
					.try_for_each(|format| format.visit_mut(f))?;
			}
			_ => {}
		}

		f(self)
	}

	fn unify(&mut self, other: Self) -> Result<()> {
		match (self, other) {
			(format1, Self::Variable(variable2)) => {
				if let Some(format2) = variable2.borrow_mut().deref_mut() {
					format1.unify(mem::take(format2))?;
				}

				*variable2.borrow_mut() = Some(format1.clone());
			}
			(Self::Variable(variable1), format2) => {
				let inner_variable = match variable1.borrow_mut().deref_mut() {
					value1 @ None => {
						*value1 = Some(format2);
						None
					}
					Some(format1) => {
						format1.unify(format2)?;
						match format1 {
							Self::Variable(variable) => Some(variable.clone()),
							_ => None,
						}
					}
				};

				if let Some(variable) = inner_variable {
					*variable1 = variable;
				}
			}
			(Self::Unit, Self::Unit)
			| (Self::Bool, Self::Bool)
			| (Self::I8, Self::I8)
			| (Self::I16, Self::I16)
			| (Self::I32, Self::I32)
			| (Self::I64, Self::I64)
			| (Self::I128, Self::I128)
			| (Self::U8, Self::U8)
			| (Self::U16, Self::U16)
			| (Self::U32, Self::U32)
			| (Self::U64, Self::U64)
			| (Self::U128, Self::U128)
			| (Self::F32, Self::F32)
			| (Self::F64, Self::F64)
			| (Self::Char, Self::Char)
			| (Self::Str, Self::Str)
			| (Self::Bytes, Self::Bytes) => {}
			(Self::TypeName(name1), Self::TypeName(name2)) if *name1 == *name2 => {}
			(Self::Option(format1), Self::Option(format2))
			| (Self::Seq(format1), Self::Seq(format2)) => format1.as_mut().unify(*format2)?,
			(Self::Tuple(formats1), Self::Tuple(formats2)) if formats1.len() == formats2.len() => {
				formats1
					.iter_mut()
					.zip(formats2.into_iter())
					.try_for_each(|(first, second)| first.unify(second))?;
			}
			(
				Self::Map {
					key: first_key,
					value: first_value,
				},
				Self::Map {
					key: second_key,
					value: second_value,
				},
			) => {
				first_key.as_mut().unify(*second_key)?;
				first_value.as_mut().unify(*second_value)?;
			}
			(format1, format2) => return Err(unification_error(format1, format2)),
		}

		Ok(())
	}

	fn is_unknown(&self) -> bool {
		if let Self::Variable(v) = self {
			v.is_unknown()
		} else {
			false
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ContainerFormat {
	Unit,
	Newtype(Box<Format>),
	Tuple(Vec<Format>),
	Struct(Vec<Named<Format>>),
	Enum(BTreeMap<u32, Named<VariantFormat>>),
}

impl FormatHolder for ContainerFormat {
	fn visit<'a>(&'a self, f: &mut dyn FnMut(&'a Format) -> Result<()>) -> Result<()> {
		match self {
			Self::Newtype(format) => format.visit(f)?,
			Self::Tuple(formats) => formats.iter().try_for_each(|format| format.visit(f))?,
			Self::Struct(named_formats) => named_formats
				.iter()
				.try_for_each(|format| format.visit(f))?,
			Self::Enum(variants) => variants.values().try_for_each(|format| format.visit(f))?,
			Self::Unit => {}
		}

		Ok(())
	}

	fn visit_mut(&mut self, f: &mut dyn FnMut(&mut Format) -> Result<()>) -> Result<()> {
		match self {
			Self::Newtype(format) => format.visit_mut(f)?,
			Self::Tuple(formats) => formats
				.iter_mut()
				.try_for_each(|format| format.visit_mut(f))?,
			Self::Struct(named_formats) => named_formats
				.iter_mut()
				.try_for_each(|format| format.visit_mut(f))?,
			Self::Enum(variants) => variants
				.values_mut()
				.try_for_each(|format| format.visit_mut(f))?,
			Self::Unit => {}
		}

		Ok(())
	}

	fn unify(&mut self, other: Self) -> Result<()> {
		match (self, other) {
			(Self::Newtype(format1), Self::Newtype(format2)) => format1.as_mut().unify(*format2)?,
			(Self::Tuple(formats1), Self::Tuple(formats2)) if formats1.len() == formats2.len() => {
				formats1
					.iter_mut()
					.zip(formats2.into_iter())
					.try_for_each(|(format1, format2)| format1.unify(format2))?;
			}
			(Self::Struct(named1), Self::Struct(named2)) if named1.len() == named2.len() => named1
				.iter_mut()
				.zip(named2.into_iter())
				.try_for_each(|(format1, format2)| format1.unify(format2))?,
			(Self::Enum(variants1), Self::Enum(variants2)) => {
				for (index2, variant2) in variants2 {
					match variants1.entry(index2) {
						Entry::Vacant(e) => {
							e.insert(variant2);
						}
						Entry::Occupied(mut e) => e.get_mut().unify(variant2)?,
					}
				}
			}
			(Self::Unit, Self::Unit) => {}
			(format1, format2) => return Err(unification_error(format1, format2)),
		}

		Ok(())
	}

	fn is_unknown(&self) -> bool {
		false
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum VariantFormat {
	Variable(#[serde(with = "not_implemented")] Variable<Self>),
	Unit,
	Newtype(Box<Format>),
	Tuple(Vec<Format>),
	Struct(Vec<Named<Format>>),
}

impl VariantFormat {
	#[must_use]
	pub fn unknown() -> Self {
		Self::Variable(Variable::new(None))
	}
}

impl Default for VariantFormat {
	fn default() -> Self {
		Self::unknown()
	}
}

impl FormatHolder for VariantFormat {
	fn visit<'a>(&'a self, f: &mut dyn FnMut(&'a Format) -> Result<()>) -> Result<()> {
		match self {
			Self::Variable(v) => v.visit(f)?,
			Self::Newtype(format) => format.visit(f)?,
			Self::Tuple(formats) => formats.iter().try_for_each(|format| format.visit(f))?,
			Self::Struct(named) => named.iter().try_for_each(|format| format.visit(f))?,
			Self::Unit => {}
		}

		Ok(())
	}

	fn visit_mut(&mut self, f: &mut dyn FnMut(&mut Format) -> Result<()>) -> Result<()> {
		match self {
			Self::Variable(v) => v.visit_mut(f)?,
			Self::Newtype(format) => format.visit_mut(f)?,
			Self::Tuple(formats) => formats
				.iter_mut()
				.try_for_each(|format| format.visit_mut(f))?,
			Self::Struct(named) => named
				.iter_mut()
				.try_for_each(|format| format.visit_mut(f))?,
			Self::Unit => {}
		}

		Ok(())
	}

	fn unify(&mut self, other: Self) -> Result<()> {
		match (self, other) {
			(format1, Self::Variable(variable2)) => {
				if let Some(format2) = variable2.borrow_mut().deref_mut() {
					format1.unify(mem::take(format2))?;
				}

				*variable2.borrow_mut() = Some(format1.clone());
			}
			(Self::Variable(variable1), format2) => {
				let inner_variable = match variable1.borrow_mut().deref_mut() {
					value1 @ None => {
						*value1 = Some(format2);
						None
					}
					Some(format1) => {
						format1.unify(format2)?;
						match format1 {
							Self::Variable(variable) => Some(variable.clone()),
							_ => None,
						}
					}
				};

				if let Some(variable) = inner_variable {
					*variable1 = variable;
				}
			}
			(Self::Newtype(format1), Self::Newtype(format2)) => {
				format1.as_mut().unify(*format2)?;
			}
			(Self::Tuple(formats1), Self::Tuple(formats2)) if formats1.len() == formats2.len() => {
				formats1
					.iter_mut()
					.zip(formats2.into_iter())
					.try_for_each(|(format1, format2)| format1.unify(format2))?;
			}
			(Self::Struct(named1), Self::Struct(named2)) if named1.len() == named2.len() => named1
				.iter_mut()
				.zip(named2.into_iter())
				.try_for_each(|(format1, format2)| format1.unify(format2))?,
			(Self::Unit, Self::Unit) => {}
			(format1, format2) => return Err(unification_error(format1, format2)),
		}

		Ok(())
	}

	fn is_unknown(&self) -> bool {
		if let Self::Variable(v) = self {
			v.is_unknown()
		} else {
			false
		}
	}
}

pub trait FormatHolder {
	fn visit<'a>(&'a self, f: &mut dyn FnMut(&'a Format) -> Result<()>) -> Result<()>;

	fn visit_mut(&mut self, f: &mut dyn FnMut(&mut Format) -> Result<()>) -> Result<()>;

	fn unify(&mut self, other: Self) -> Result<()>;

	fn is_unknown(&self) -> bool;

	fn normalize(&mut self) -> Result<()> {
		self.visit_mut(&mut |format: &mut Format| {
			let normalized = match format {
				Format::Tuple(formats) => {
					let size = formats.len();
					if size <= 1 {
						return Ok(());
					}

					let first_format = &formats[0];
					for format in formats.iter().skip(1) {
						if format != first_format {
							return Ok(());
						}
					}

					Format::TupleArray {
						content: Box::new(mem::take(&mut formats[0])),
						size,
					}
				}
				_ => return Ok(()),
			};

			*format = normalized;

			Ok(())
		})
	}

	fn reduce(&mut self) {
		self.visit_mut(&mut |_| Ok(())).unwrap_or(());
	}
}

pub(crate) trait ContainerFormatEntry {
	fn unify(self, format: ContainerFormat) -> Result<()>;
}

impl<K> ContainerFormatEntry for Entry<'_, K, ContainerFormat>
where
	K: Ord,
{
	fn unify(self, format: ContainerFormat) -> Result<()> {
		match self {
			Self::Vacant(e) => {
				e.insert(format);
				Ok(())
			}
			Self::Occupied(e) => e.into_mut().unify(format),
		}
	}
}

fn unification_error(v1: impl Debug, v2: impl Debug) -> Error {
	Error::Incompatible(format!("{v1:?}"), format!("{v2:?}"))
}
