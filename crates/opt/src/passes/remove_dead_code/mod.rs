mod empty_loops;
mod non_movements;
mod pointless_instr;
mod redundancies;
mod unreachable_loops;
mod unused_starting_instr;

pub use self::{
	empty_loops::*, non_movements::*, pointless_instr::*, redundancies::*, unreachable_loops::*,
	unused_starting_instr::*,
};
