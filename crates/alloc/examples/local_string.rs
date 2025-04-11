use std::{hint::black_box, mem, ptr::NonNull, time::Instant};

use vmm_alloc::Stalloc;

fn main() {
	let start = Instant::now();

	let s = Stalloc::<200, 4>::new();

	for _ in 0..100_000_000 {
		let mut message = unsafe {
			String::from_raw_parts(s.allocate_blocks(50, 1).unwrap().as_ptr(), 0, 50 * 4)
		};

		message.push_str("Hello, ");
		message.push_str("world!");

		message = black_box(message);

		unsafe {
			s.deallocate_blocks(NonNull::new_unchecked(message.as_mut_ptr()), 50);
		}

        mem::forget(message);
	}

	println!("Elapsed: {}ms", start.elapsed().as_millis());
}
