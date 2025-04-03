#![allow(clippy::all)]

use std::{
	fmt::{Debug, Display, Formatter, Result as FmtResult, Write as _},
	iter::FusedIterator,
};

use vmm_ident::{Ident, ident};

use super::item::ItemKind;

include!(concat!(env!("OUT_DIR"), "/block.rs"));

impl Debug for BlockState {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		fmt_block_state(*self, f)
	}
}

impl Display for BlockState {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Debug::fmt(&self, f)
	}
}

fn fmt_block_state(bs: BlockState, f: &mut Formatter<'_>) -> FmtResult {
	let kind = bs.to_kind();

	f.write_str(kind.to_str())?;

	let props = kind.props();

	if !props.is_empty() {
		let mut list = f.debug_list();
		for &p in kind.props() {
			struct KeyVal<'a>(&'a str, &'a str);

			impl<'a> Debug for KeyVal<'a> {
				fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
					f.write_str(&self.0)?;
					f.write_char('=')?;
					f.write_str(&self.1)
				}
			}

			list.entry(&KeyVal(p.to_str(), bs.get(p).unwrap().to_str()));
		}

		list.finish()
	} else {
		Ok(())
	}
}
