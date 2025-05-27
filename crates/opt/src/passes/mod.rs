mod clear_cell;
mod clear_loop;
mod collapse_relative_instr;
mod collapse_stacked_instr;
mod combine_move_change;
mod fetch_value;
mod find_zero;
mod inspect_instr;
mod move_value;
mod remove_dead_code;
mod reorder_instr;
mod scale_val;
mod unroll_constant_loops;
mod unroll_increment_loops;
mod unroll_one_off_loops;

pub use self::{
	clear_cell::*, clear_loop::*, collapse_relative_instr::*, collapse_stacked_instr::*,
	combine_move_change::*, fetch_value::*, find_zero::*, inspect_instr::*, move_value::*,
	remove_dead_code::*, reorder_instr::*, scale_val::*, unroll_constant_loops::*,
	unroll_increment_loops::*, unroll_one_off_loops::*,
};
