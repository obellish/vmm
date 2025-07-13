use alloc::{rc::Rc, vec::Vec};
use core::cell::RefCell;

use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeSeq};

pub fn deserialize_option<'de, D, T>(deserializer: D) -> Result<Option<Rc<RefCell<T>>>, D::Error>
where
	D: Deserializer<'de>,
	T: Deserialize<'de>,
{
	let opt = Option::<T>::deserialize(deserializer)?;
	Ok(opt.map(into_rc))
}

pub fn deserialize_vec<'de, D, T>(deserializer: D) -> Result<Vec<Rc<RefCell<T>>>, D::Error>
where
	D: Deserializer<'de>,
	T: Deserialize<'de>,
{
	let vec = Vec::<T>::deserialize(deserializer)?;
	Ok(vec.into_iter().map(into_rc).collect())
}

fn into_rc<T>(value: T) -> Rc<RefCell<T>> {
	Rc::new(RefCell::new(value))
}

pub fn serialize_vec<S: Serializer, T: Serialize>(
	value: &Vec<Rc<RefCell<T>>>,
	serializer: S,
) -> Result<S::Ok, S::Error> {
	let mut seq = serializer.serialize_seq(Some(value.len()))?;

	for item in value {
		seq.serialize_element(&*item.borrow())?;
	}

	seq.end()
}

#[expect(clippy::ref_option, reason = "used for #[serde(with = ...)]")]
pub fn serialize_option<S: Serializer, T: Serialize>(
	value: &Option<Rc<RefCell<T>>>,
	serializer: S,
) -> Result<S::Ok, S::Error> {
	match value {
		Some(rc) => serialize(rc, serializer),
		None => serializer.serialize_none(),
	}
}

fn serialize<S: Serializer, T: Serialize>(
	rc: &Rc<RefCell<T>>,
	serializer: S,
) -> Result<S::Ok, S::Error> {
	T::serialize(&*rc.borrow(), serializer)
}
