use std::{cell::RefCell, collections::HashSet};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use vmm_serde_array::BigArray;

thread_local! {
	static DROPPED: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
}

#[derive(Serialize, Deserialize)]
struct S {
	#[serde(with = "BigArray")]
	arr: [u8; 64],
	#[serde(with = "BigArray")]
	arr2: [u8; 65],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
struct DroppableU32(u32);

impl Drop for DroppableU32 {
	fn drop(&mut self) {
		DROPPED.with(|dropped| dropped.borrow_mut().push(self.0));
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
struct Droppables<const CNT: usize> {
	#[serde(with = "BigArray")]
	arr: [DroppableU32; CNT],
}

impl<const CNT: usize> Droppables<CNT> {
	fn test(value_idx: usize, value: u32) -> Result<()> {
		let mut maybe_init_array = core::mem::MaybeUninit::<[DroppableU32; CNT]>::uninit();
		for i in 0..CNT {
			unsafe {
				let p = maybe_init_array
					.as_mut_ptr()
					.cast::<DroppableU32>()
					.wrapping_add(i);
				core::ptr::write(p, DroppableU32(i as u32 * 3));
			}
		}

		let mut ds = Self {
			arr: unsafe { maybe_init_array.assume_init() },
		};

		clear_dropped_set();
		ds.arr[value_idx] = DroppableU32(value);
		assert_eq!(
			get_dropped_set(),
			std::iter::once(value_idx as u32 * 3).collect()
		);
		clear_dropped_set();

		let j = serde_json::to_string(&ds)?;

		let val_starts = j.find(&value.to_string()).unwrap();
		{
			let ds_back = serde_json::from_str::<Self>(&j)?;
			assert_eq!(&ds.arr[..], &ds_back.arr[..]);
		}

		let mut zero_to_cnt_set = (0..CNT as u32)
			.map(|v| v * 3)
			.into_iter()
			.collect::<HashSet<_>>();

		zero_to_cnt_set.remove(&(value_idx as u32 * 3));
		zero_to_cnt_set.insert(value);

		assert_eq!(get_dropped_set(), zero_to_cnt_set);
		clear_dropped_set();

		let _ds_back_err = serde_json::from_str::<Self>(&j[0..val_starts]).unwrap_err();

		let zero_to_value_idx_set = (0..value_idx as u32)
			.map(|v| v * 3)
			.into_iter()
			.collect::<HashSet<_>>();

		assert_eq!(get_dropped_set(), zero_to_value_idx_set);
		clear_dropped_set();

		Ok(())
	}
}

fn get_dropped_set() -> HashSet<u32> {
	DROPPED.with(|dropped| dropped.borrow().iter().copied().collect())
}

fn clear_dropped_set() {
	DROPPED.with(|dropped| dropped.borrow_mut().clear());
	assert_eq!(get_dropped_set().len(), 0);
}

#[test]
fn works() -> Result<()> {
	let s = S {
		arr: [1; 64],
		arr2: [2; 65],
	};

	let v = serde_value::to_value(&s)?;
	let s_back = v.deserialize_into::<S>()?;

	assert_eq!(&s.arr[..], &s_back.arr[..]);
	assert_eq!(&s.arr2[..], &s_back.arr2[..]);

	Ok(())
}

#[test]
fn dropped_partial() -> Result<()> {
	Droppables::<4>::test(2, 20_220_325)?;
	Droppables::<77>::test(50, 20_220_325)
}
