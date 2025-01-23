use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::{
	vmm::board::Bus,
	vmm_tools::{
		exceptions::AuxHwException,
		metadata::{ClockType, DeviceCategory, DeviceMetadata},
	},
};

#[derive(Debug)]
pub struct RealtimeClock {
	reset_at: Instant,
	hw_id: u64,
}

impl RealtimeClock {
	#[must_use]
	pub fn new(hw_id: u64) -> Self {
		Self {
			reset_at: Instant::now(),
			hw_id,
		}
	}
}

impl Bus for RealtimeClock {
	fn name(&self) -> &'static str {
		"Realtime Clock"
	}

	fn metadata(&self) -> [u32; 8] {
		DeviceMetadata::new(
			self.hw_id,
			24,
			DeviceCategory::Clock(ClockType::Realtime),
			None,
			None,
		)
		.encode()
	}

	fn read(&mut self, addr: u32, ex: &mut u16) -> u32 {
		let time = if addr < 14 {
			if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
				duration
			} else {
				*ex = AuxHwException::TimeSynchronizationError.encode();
				return 0;
			}
		} else {
			self.reset_at.elapsed()
		};

		match addr % 3 {
			0x00 => (time.as_secs() >> 32) as u32,
			0x01 => (time.as_secs() & 0xFFFF_FFFF) as u32,
			0x02 => {
				(time.subsec_millis() * 1_000_000)
					+ (time.subsec_micros() * 1000)
					+ time.subsec_nanos()
			}
			_ => unreachable!(),
		}
	}

	fn write(&mut self, _: u32, _: u32, ex: &mut u16) {
		*ex = AuxHwException::MemoryNotWritable.encode();
	}

	fn reset(&mut self) {
		self.reset_at = Instant::now();
	}
}
