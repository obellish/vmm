use super::{cpu::Registers, mem::MappedMemory};

#[derive(Default)]
pub struct Mmu;

#[allow(clippy::unused_self, clippy::needless_pass_by_ref_mut)]
impl Mmu {
	#[must_use]
	pub const fn new() -> Self {
		Self
	}

	pub fn decode_entry(
		&mut self,
		mem: &mut MappedMemory,
		regs: &Registers,
		entry_addr: u32,
		action: MemAction,
	) -> EntryDecodingResult {
		let mut ex = 0;

		let v_entry = mem.read(entry_addr, &mut ex);

		if !matches!(ex, 0) {
			return EntryDecodingResult::HwException(ex);
		}

		if matches!(
			v_entry & (0b1 << if matches!(regs.smt, 0) { 30 } else { 31 }),
			0b0
		) {
			return EntryDecodingResult::PassThrough;
		}

		let mode_shift = if matches!(regs.smt, 0) { 0u32 } else { 3 };

		let action_shift = match action {
			MemAction::Read => 2,
			MemAction::Write => 1,
			MemAction::Exec => 0,
		};

		if matches!(v_entry & (0b1 << (24 + action_shift + mode_shift)), 1) {
			EntryDecodingResult::Decoded(v_entry & 0b1111_1111_1111_1111_1111_1111)
		} else {
			EntryDecodingResult::PermissionNotSet
		}
	}

	pub fn translate(
		&mut self,
		mem: &mut MappedMemory,
		regs: &Registers,
		v_addr: u32,
		action: MemAction,
	) -> Result<u32, Option<u16>> {
		if matches!(regs.mtt, 0) {
			return Ok(v_addr);
		}

		let vpi_entry_number = v_addr & 0b11_1111_1111;

		let vpi_entry_addr = regs.pda + (vpi_entry_number * 4);

		let v_page_number = match self.decode_entry(mem, regs, vpi_entry_addr, action) {
			EntryDecodingResult::Decoded(value) => value,
			EntryDecodingResult::PassThrough => return Ok(v_addr),
			EntryDecodingResult::PermissionNotSet => return Err(None),
			EntryDecodingResult::HwException(ex) => return Err(Some(ex)),
		};

		let v_page_addr = v_page_number * 16384;

		let v_page_entry_addr = v_page_addr + (v_addr.wrapping_shl(10) >> 22) * 4;

		let p_page_number = match self.decode_entry(mem, regs, v_page_entry_addr, action) {
			EntryDecodingResult::Decoded(value) => value,
			EntryDecodingResult::PassThrough => return Ok(v_addr),
			EntryDecodingResult::PermissionNotSet => return Err(None),
			EntryDecodingResult::HwException(ex) => return Err(Some(ex)),
		};

		Ok(p_page_number * 1024 + (v_addr & 0b11_1111_1111))
	}
}

#[derive(Debug, Clone, Copy)]
pub enum EntryDecodingResult {
	Decoded(u32),
	PassThrough,
	PermissionNotSet,
	HwException(u16),
}

#[derive(Debug, Clone, Copy)]
pub enum MemAction {
	Read,
	Write,
	Exec,
}
