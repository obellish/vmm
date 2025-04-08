use std::{
	collections::HashMap,
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::items::Item;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryEntry {
	pub id: u32,
	pub slot: i8,
	pub count: i8,
	pub nbt: Option<Vec<u8>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SignBlockEntity {
	pub front_rows: [String; 4],
	pub back_rows: [String; 4],
}

#[derive(Debug)]
#[repr(transparent)]
pub struct TryParseContainerTypeError(String);

impl Display for TryParseContainerTypeError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("invalid container type ")?;
		f.write_str(&self.0)
	}
}

impl StdError for TryParseContainerTypeError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerType {
	Furnace,
	Barrel,
	Hopper,
}

impl ContainerType {
	#[must_use]
	pub const fn slots(self) -> u8 {
		match self {
			Self::Furnace => 3,
			Self::Barrel => 27,
			Self::Hopper => 5,
		}
	}

	#[must_use]
	pub const fn window_type(self) -> u8 {
		match self {
			Self::Furnace => 14,
			Self::Barrel => 2,
			Self::Hopper => 16,
		}
	}
}

impl Display for ContainerType {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str("minecraft:")?;
		f.write_str(match self {
			Self::Furnace => "furnace",
			Self::Barrel => "barrel",
			Self::Hopper => "hopper",
		})
	}
}

impl FromStr for ContainerType {
	type Err = TryParseContainerTypeError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"barrel" => Self::Barrel,
			"furnace" => Self::Furnace,
			"hopper" => Self::Hopper,
			s => return Err(TryParseContainerTypeError(s.to_owned())),
		})
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockEntity {
	Comparator {
		output_strength: u8,
	},
	Container {
		comparator_override: u8,
		inventory: Vec<InventoryEntry>,
		ty: ContainerType,
	},
	Sign(SignBlockEntity),
}

impl BlockEntity {
	#[must_use]
	pub const fn ty(&self) -> i32 {
		match self {
			Self::Comparator { .. } => 18,
			Self::Sign(_) => 7,
			Self::Container { ty, .. } => match ty {
				ContainerType::Furnace => 0,
				ContainerType::Barrel => 26,
				ContainerType::Hopper => 17,
			},
		}
	}

	fn load_container(slots_nbt: &[nbt::Value], ty: ContainerType) -> Option<Self> {
		let num_slots = ty.slots();
		let mut fullness_sum = 0.0f32;
		let mut inventory = Vec::new();
		for item in slots_nbt {
			let nbt::Value::Compound(item_compound) = item else {
				return None;
			};

			let nbt::Value::Byte(count) = item_compound["Count"] else {
				return None;
			};

			let nbt::Value::Byte(slot) = item_compound["Slot"] else {
				return None;
			};

			let nbt::Value::String(namespaced_name) = item_compound
				.get("Id")
				.or_else(|| item_compound.get("id"))?
			else {
				return None;
			};

			let item_type = Item::from_name(namespaced_name.split(':').next_back()?);

			let mut blob = nbt::Blob::new();
			for (k, v) in item_compound {
				blob.insert(k, v.clone()).ok()?;
			}

			let mut data = Vec::new();
			blob.to_writer(&mut data).ok()?;

			let tag = match item_compound.get("tag") {
				Some(nbt::Value::Compound(map)) => {
					let mut blob = nbt::Blob::new();
					for (k, v) in map {
						blob.insert(k, v.clone()).ok()?;
					}

					let mut data = Vec::new();
					blob.to_writer(&mut data).ok()?;
					Some(data)
				}
				_ => None,
			};

			inventory.push(InventoryEntry {
				slot,
				count,
				id: item_type.unwrap_or(Item::Redstone {}).id(),
				nbt: tag,
			});

			fullness_sum += f32::from(count) / item_type.map_or(64, Item::max_stack_size) as f32;
		}

		Some(Self::Container {
			comparator_override: (fullness_sum / f32::from(num_slots))
				.mul_add(14.0, if fullness_sum > 0.0 { 1.0 } else { 0.0 })
				.floor() as u8,
			inventory,
			ty,
		})
	}

	#[must_use]
	pub fn from_nbt(id: &str, nbt: &HashMap<String, nbt::Value>) -> Option<Self> {
		match id.trim_start_matches("minecraft:") {
			"comparator" => Some(Self::Comparator {
				output_strength: {
					let nbt::Value::Int(value) = &nbt["OutputSignal"] else {
						return None;
					};

					*value as u8
				},
			}),
			"furnace" => Self::load_container(
				{
					let nbt::Value::List(value) = &nbt["Items"] else {
						return None;
					};

					value
				},
				ContainerType::Furnace,
			),
			"barrel" => Self::load_container(
				{
					let nbt::Value::List(value) = &nbt["Items"] else {
						return None;
					};

					value
				},
				ContainerType::Barrel,
			),
			"hopper" => Self::load_container(
				{
					let nbt::Value::List(value) = &nbt["Items"] else {
						return None;
					};

					value
				},
				ContainerType::Hopper,
			),
			"sign" => {
				let sign = if nbt.contains_key("Text1") {
					SignBlockEntity {
						front_rows: [
							{
								let nbt::Value::String(s) = nbt["Text1"].clone() else {
									return None;
								};

								s
							},
							{
								let nbt::Value::String(s) = nbt["Text2"].clone() else {
									return None;
								};

								s
							},
							{
								let nbt::Value::String(s) = nbt["Text3"].clone() else {
									return None;
								};

								s
							},
							{
								let nbt::Value::String(s) = nbt["Text4"].clone() else {
									return None;
								};

								s
							},
						],
						back_rows: Default::default(),
					}
				} else {
					let get_side = |side| {
						let nbt::Value::Compound(messages) = &nbt[side] else {
							return None;
						};

						let messages = messages.get("messages")?.clone();

						let nbt::Value::List(messages) = messages else {
							return None;
						};

						let mut messages = messages.iter().cloned();

						Some([
							{
								let nbt::Value::String(s) = messages.next()? else {
									return None;
								};

								s
							},
							{
								let nbt::Value::String(s) = messages.next()? else {
									return None;
								};

								s
							},
							{
								let nbt::Value::String(s) = messages.next()? else {
									return None;
								};

								s
							},
							{
								let nbt::Value::String(s) = messages.next()? else {
									return None;
								};

								s
							},
						])
					};

					SignBlockEntity {
						front_rows: get_side("front_text")?,
						back_rows: get_side("back_text")?,
					}
				};

				Some(Self::Sign(sign))
			}
			_ => None,
		}
	}

	#[must_use]
	pub fn to_nbt(&self, sign_only: bool) -> Option<nbt::Blob> {
		if sign_only && !matches!(self, Self::Sign(_)) {
			return None;
		}

		match self {
			Self::Sign(sign) => Some({
				let front = sign
					.front_rows
					.iter()
					.map(|str| nbt::Value::String(str.clone()));
				let back = sign
					.back_rows
					.iter()
					.map(|str| nbt::Value::String(str.clone()));

				let mut map = HashMap::new();
				map.insert("is_waxed".to_owned(), nbt::Value::Byte(0));
				map.insert(
					"front_text".to_owned(),
					nbt::Value::Compound({
						let mut map = HashMap::new();
						map.insert("has_glowing_text".to_owned(), nbt::Value::Byte(0));
						map.insert("color".to_owned(), nbt::Value::String("black".to_owned()));
						map.insert("messages".to_owned(), nbt::Value::List(front.collect()));

						map
					}),
				);
				map.insert(
					"back_text".to_owned(),
					nbt::Value::Compound({
						let mut map = HashMap::new();
						map.insert("has_glowing_text".to_owned(), nbt::Value::Byte(0));
						map.insert("color".to_owned(), nbt::Value::String("black".to_owned()));
						map.insert("messages".to_owned(), nbt::Value::List(back.collect()));

						map
					}),
				);

				map.insert(
					"id".to_owned(),
					nbt::Value::String("minecraft:sign".to_owned()),
				);

				nbt::Blob::with_content(map)
			}),
			Self::Comparator { output_strength } => Some({
				let mut map = HashMap::new();
				map.insert(
					"OutputSignal".to_owned(),
					nbt::Value::Int(i32::from(*output_strength)),
				);
				map.insert(
					"id".to_owned(),
					nbt::Value::String("minecraft:comparator".to_owned()),
				);

				nbt::Blob::with_content(map)
			}),
			Self::Container { inventory, ty, .. } => Some({
				let mut items = Vec::new();
				for entry in inventory {
					let mut nbt = HashMap::new();

					nbt.insert("Count".to_owned(), nbt::Value::Byte(entry.count));
					nbt.insert(
						"id".to_owned(),
						nbt::Value::String(
							"minecraft:".to_owned() + Item::from_id(entry.id).name(),
						),
					);
					nbt.insert("Slot".to_owned(), nbt::Value::Byte(entry.slot));

					items.push(nbt::Value::Compound(nbt));
				}

				let mut map = HashMap::new();

				map.insert("id".to_owned(), nbt::Value::String(ty.to_string()));
				map.insert("Items".to_owned(), nbt::Value::List(items));

				nbt::Blob::with_content(map)
			}),
		}
	}
}
