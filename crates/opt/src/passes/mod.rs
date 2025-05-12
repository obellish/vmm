mod clear_cell;
mod collapse_stacked_instr;
mod find_zero;
mod move_value;
mod remove_dead_code;
mod set_untouched_cells;
mod unroll_constant_loops;

pub use self::{
	clear_cell::*, collapse_stacked_instr::*, find_zero::*, move_value::*, remove_dead_code::*,
	set_untouched_cells::*, unroll_constant_loops::*,
};
