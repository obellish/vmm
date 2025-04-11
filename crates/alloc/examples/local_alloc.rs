#![feature(allocator_api)]

use std::{mem, time::Instant};

use vmm_alloc::Stalloc;

fn main() {
	let start = Instant::now();
	for _ in 0..10_000_000 {
		let alloc = Stalloc::<200, 4>::new();

		let mut a = Vec::new_in(&alloc);
		let mut b = Vec::new_in(&alloc);
		for i in 0..10 {
			a.push(i);
			b.push(i);
		}

		mem::forget(a);
		mem::forget(b);
	}

	println!("Elapsed: {}ms", start.elapsed().as_millis());
}
