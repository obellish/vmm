mod combine_instr;
mod instr_scan;
mod remove_empty_loops;
mod search_for_zero;
mod set_untouched_cells;
mod unroll_constant_loops;

pub use self::{
	combine_instr::*, instr_scan::*, remove_empty_loops::*, search_for_zero::*,
	set_untouched_cells::*, unroll_constant_loops::*,
};
