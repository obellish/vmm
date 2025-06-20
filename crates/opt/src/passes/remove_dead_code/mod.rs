mod empty_loops;
mod infinite_loops;
mod pointless_instr;
mod redundancies;
mod unreachable_loops;
mod unused_boundary_instr;

pub use self::{
	empty_loops::*, infinite_loops::*, pointless_instr::*, redundancies::*, unreachable_loops::*,
	unused_boundary_instr::*,
};
