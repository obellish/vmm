mod clean_up_start;
mod empty_loops;
mod no_move_or_change;
mod redundancies;
mod unreachable_loops;

pub use self::{
	clean_up_start::*, empty_loops::*, no_move_or_change::*, redundancies::*, unreachable_loops::*,
};
