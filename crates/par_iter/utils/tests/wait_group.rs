use std::{sync::mpsc, thread, time::Duration};

use vmm_par_utils::sync::WaitGroup;

const THREADS: usize = 10;

#[test]
fn wait() {
	let wg = WaitGroup::new();
	let (tx, rx) = mpsc::channel();

	for _ in 0..THREADS {
		let wg = wg.clone();
		let tx = tx.clone();

		thread::spawn(move || {
			wg.wait();
			tx.send(()).unwrap();
		});
	}

	thread::sleep(Duration::from_millis(100));

	assert!(rx.try_recv().is_err());

	wg.wait();

	for _ in 0..THREADS {
		rx.recv().unwrap();
	}
}

#[test]
fn wait_and_drop() {
	let wg = WaitGroup::new();
	let wg2 = WaitGroup::new();
	let (tx, rx) = mpsc::channel();

	for _ in 0..THREADS {
		let wg = wg.clone();
		let wg2 = wg2.clone();
		let tx = tx.clone();

		thread::spawn(move || {
			wg2.wait();
			tx.send(()).unwrap();
			drop(wg);
		});
	}

	assert!(rx.try_recv().is_err());
	drop(wg2);

	wg.wait();

	for _ in 0..THREADS {
		rx.try_recv().unwrap();
	}
}
