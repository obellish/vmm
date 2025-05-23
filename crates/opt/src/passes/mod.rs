mod clear_cell;
mod clear_loop;
mod collapse_relative_instr;
mod collapse_stacked_instr;
mod combine_move_change;
mod find_zero;
mod inspect_instr;
mod move_to_fetch;
mod move_value;
mod remove_dead_code;
mod reorder_instr;
mod set_untouched_cells;
mod unroll_constant_loops;
mod unroll_increment_loops;

pub use self::{
	clear_cell::*, clear_loop::*, collapse_relative_instr::*, collapse_stacked_instr::*,
	combine_move_change::*, find_zero::*, inspect_instr::*, move_to_fetch::*, move_value::*,
	remove_dead_code::*, reorder_instr::*, set_untouched_cells::*, unroll_constant_loops::*,
	unroll_increment_loops::*,
};
