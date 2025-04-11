#![feature(allocator_api)]

use std::{hint::black_box, thread, time::Instant};
use vmm_alloc::SyncStalloc;

const THREAD_COUNT: usize = 6;

fn main() {
    let start = Instant::now();

    for _ in 0..5000 {
        let alloc = SyncStalloc::<THREAD_COUNT, 4>::new();

        thread::scope(|s| {
            for _ in 0..THREAD_COUNT {
                s.spawn(|| {
                    let mut total = 0;
                    for i in 0..1000 {
                        let lock = alloc.acquire_locked();
                        total += *black_box(Box::new_in(i, &*lock));
                    }

                    assert_eq!(total, 499_500);
                });
            }
        });
    }

    println!("Elapsed: {}ms", start.elapsed().as_millis());
}
