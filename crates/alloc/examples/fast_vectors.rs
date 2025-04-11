use std::{mem, time::Instant};

use vmm_alloc::UnsafeStalloc;

#[global_allocator]
static ALLOC: UnsafeStalloc<1000, 4> = unsafe { UnsafeStalloc::new() };

fn main() {
	let start = Instant::now();
	for _ in 0..10_000_000 {
		let mut a = Vec::new();
		let mut b = Vec::new();
		for i in 0..10 {
			a.push(i);
			b.push(i);
		}

		mem::forget(a);
		mem::forget(b);

		unsafe {
			ALLOC.clear();
		}
	}

	println!("Elapsed: {}ms", start.elapsed().as_millis());
}
