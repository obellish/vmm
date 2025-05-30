mod clear_cell;
mod clear_loop;
mod collapse_relative_instr;
mod collapse_stacked_instr;
mod combine_move_change;
mod fetch_value;
mod find_zero;
mod if_nz;
mod move_value;
mod remove_dead_code;
mod reorder_instr;
mod scale_val;
mod set_scale;
mod simd_inc;
mod take_val;
mod unroll_constant_loops;
mod unroll_increment_loops;
mod zeroed_cell_inc;

pub use self::{
	clear_cell::*, clear_loop::*, collapse_relative_instr::*, collapse_stacked_instr::*,
	combine_move_change::*, fetch_value::*, find_zero::*, if_nz::*, move_value::*,
	remove_dead_code::*, reorder_instr::*, scale_val::*, set_scale::*, simd_inc::*, take_val::*,
	unroll_constant_loops::*, unroll_increment_loops::*, zeroed_cell_inc::*,
};
