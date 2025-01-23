mod category;
mod types;

pub use self::{category::DeviceCategory, types::*};
use super::bytes::bytes_to_words;

#[derive(Debug, Clone, Copy)]
pub struct DeviceMetadata {
	pub hw_id: u64,
	pub size_bytes: u32,
	pub category: DeviceCategory,
	pub model: Option<u32>,
	pub data: Option<u64>,
}

impl DeviceMetadata {
	#[must_use]
	pub const fn new(
		hw_id: u64,
		size_bytes: u32,
		category: DeviceCategory,
		model: Option<u32>,
		data: Option<u64>,
	) -> Self {
		assert!(
			matches!(size_bytes % 4, 0),
			"components' size must be a multiple of 4"
		);

		Self {
			hw_id,
			size_bytes,
			category,
			model,
			data,
		}
	}

	#[must_use]
	pub const fn set_size(mut self, new_size: u32) -> Self {
		self.size_bytes = new_size;
		self
	}

	#[must_use]
	pub fn to_bytes(self) -> [u8; 32] {
		let mut bytes = [0; 32];

		bytes[0..=7].copy_from_slice(&self.hw_id.to_be_bytes());
		bytes[8..=11].copy_from_slice(&self.size_bytes.to_be_bytes());
		bytes[12..=19].copy_from_slice(&self.category.encode().to_be_bytes());
		bytes[20..=23].copy_from_slice(&self.model.unwrap_or(0).to_be_bytes());
		bytes[24..=31].copy_from_slice(&self.data.unwrap_or(0).to_be_bytes());

		bytes
	}

	#[must_use]
	pub fn encode(self) -> [u32; 8] {
		let mut words = [0; 8];
		words.copy_from_slice(&bytes_to_words(&self.to_bytes()));
		words
	}
}
