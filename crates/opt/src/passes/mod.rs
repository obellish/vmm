mod clear_cell;
mod clear_loop;
mod clear_until_zero;
mod collapse_relative_instr;
mod collapse_stacked_instr;
mod combine_move_change;
mod constant_sub;
mod dupe_and_scale;
mod dupe_val;
mod fetch_and_scale_val;
mod fetch_val;
mod find_zero;
mod if_nz;
mod remove_dead_code;
mod reorder_instr;
mod scale_and_move_val;
mod scale_and_take_val;
mod scale_val;
mod set_scale;
mod set_zero;
mod simd;
mod sub_cell;
mod take_to_fetch;
mod take_val;
mod unroll_constant_loops;
mod unroll_increment_loops;
mod unroll_super_scale;
mod zeroed_cell_inc;

pub use self::{
	clear_cell::*, clear_loop::*, clear_until_zero::*, collapse_relative_instr::*,
	collapse_stacked_instr::*, combine_move_change::*, constant_sub::*, dupe_and_scale::*,
	dupe_val::*, fetch_and_scale_val::*, fetch_val::*, find_zero::*, if_nz::*, remove_dead_code::*,
	reorder_instr::*, scale_and_move_val::*, scale_and_take_val::*, scale_val::*, set_scale::*,
	set_zero::*, simd::*, sub_cell::*, take_to_fetch::*, take_val::*, unroll_constant_loops::*,
	unroll_increment_loops::*, unroll_super_scale::*, zeroed_cell_inc::*,
};
