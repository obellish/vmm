mod clear_cell;
mod clear_loop;
mod collapse_stacked_instr;
mod find_zero;
mod inspect_instr;
mod move_value;
mod remove_dead_code;
mod set_untouched_cells;
mod unroll_constant_loops;
mod unroll_increment_loops;

pub use self::{
	clear_cell::*, clear_loop::*, collapse_stacked_instr::*, find_zero::*, inspect_instr::*,
	move_value::*, remove_dead_code::*, set_untouched_cells::*, unroll_constant_loops::*,
	unroll_increment_loops::*,
};
