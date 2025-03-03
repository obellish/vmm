#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![allow(clippy::trivially_copy_pass_by_ref)]

mod action;
mod compare;
mod datum;
mod effect;
mod goal;
mod local_state;
mod mutator;

pub use self::{
	compare::Compare,
	datum::Datum,
	effect::Effect,
	local_state::{InternalData, LocalState},
	mutator::Mutator,
};
