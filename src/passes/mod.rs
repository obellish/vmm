mod combine_instr;
mod find_zero;
mod instr_scan;
mod move_value;
mod remove_dead_code;
mod set_untouched_cells;
mod set_zero;
#[cfg(test)]
mod tests;
mod unroll_constant_loops;

pub use self::{
	combine_instr::*, find_zero::*, instr_scan::*, move_value::*, remove_dead_code::*,
	set_untouched_cells::*, set_zero::*, unroll_constant_loops::*,
};
