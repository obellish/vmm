mod config;

use std::{
	borrow::Cow,
	fmt::{Display, Formatter, Result as FmtResult},
};

use vmm::cpu::Cpu;

pub use self::config::*;
use crate::exceptions::NativeException;

#[derive(Debug, Clone, Copy)]
pub struct StoppedState {
	pub cycles: u128,
	pub addr: u32,
	pub ex: Option<ExWithMode>,
}

impl Display for StoppedState {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(&prettify_stop(*self))
	}
}

#[derive(Debug, Clone, Copy)]
pub struct ExWithMode {
	pub raw: u32,
	pub sv_mode: bool,
	pub code: u8,
	pub associated: u16,
}

impl Display for ExWithMode {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(&prettify_ex_with_mode(*self))
	}
}

pub fn run_vm(cpu: &mut Cpu, config: RunConfig) -> StoppedState {
	let mut stop_ex = None;

	let mut was_at = cpu.regs.pc;

	while !cpu.halted() {
		if let Some(cycles_limit) = config.cycles_limit {
			if cpu.cycles() > cycles_limit {
				break;
			}
		}

		was_at = cpu.regs.pc;

		if config.print_cycles {
			println!(
				"[vmm] running cycle {:#010X} at address {:#010X}",
				cpu.cycles(),
				cpu.regs.pc
			);
		}

		cpu.next();

		if !matches!(cpu.regs.et, 0) {
			let exception_bytes = cpu.regs.et.to_be_bytes();

			let ex = ExWithMode {
				raw: cpu.regs.et,
				sv_mode: !matches!(exception_bytes[0], 0),
				code: exception_bytes[1],
				associated: u16::from_be_bytes([exception_bytes[2], exception_bytes[3]]),
			};

			if config.print_exceptions && !(config.halt_on_exception && config.print_finish) {
				println!("[vmm] at address {was_at:#010X} - exception occurred: {ex}");
			}

			if config.halt_on_exception {
				stop_ex = Some(ex);
				break;
			}
		}
	}

	let state = StoppedState {
		cycles: cpu.cycles(),
		addr: was_at,
		ex: stop_ex,
	};

	if config.print_finish {
		if config.newline_on_finish {
			println!();
		}

		println!("[vmm] {state}");
	}

	state
}

#[must_use]
pub fn prettify_ex_with_mode(ex: ExWithMode) -> Cow<'static, str> {
	NativeException::decode_parts(ex.code, Some(ex.associated))
		.map_or(Cow::Borrowed("<invalid exception code or data>"), |ex| {
			Cow::Owned(format!("{ex}"))
		})
}

#[must_use]
pub fn prettify_stop(state: StoppedState) -> String {
	let mut output = format!(
		"cycle {:#010X}: CPU halted at address {:#010X}",
		state.cycles, state.addr
	);

	if let Some(ex) = state.ex {
		output.push_str(&format!(
			" because of exception in {} mode: {}",
			if ex.sv_mode { "supervisor" } else { "userland" },
			prettify_ex_with_mode(ex)
		));
	}

	output
}
