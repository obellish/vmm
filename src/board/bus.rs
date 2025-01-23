pub trait Bus: std::fmt::Debug {
	fn name(&self) -> &'static str;

	fn metadata(&self) -> [u32; 8];

	fn read(&mut self, addr: u32, ex: &mut u16) -> u32;

	fn write(&mut self, addr: u32, word: u32, ex: &mut u16);

	fn reset(&mut self);
}
