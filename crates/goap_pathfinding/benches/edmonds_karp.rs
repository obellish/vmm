use std::collections::HashMap;

use criterion::{Criterion, criterion_group, criterion_main};
use goap_pathfinding::prelude::*;

fn successors_wikipedia() -> Vec<((char, char), i32)> {
	[
		("AB", 3),
		("AD", 3),
		("BC", 4),
		("CA", 3),
		("CD", 1),
		("CE", 2),
		("DE", 2),
		("DF", 6),
		("EB", 1),
		("EG", 1),
		("FG", 9),
	]
	.into_iter()
	.map(|(s, c)| {
		let mut name = s.chars();
		((name.next().unwrap(), name.next().unwrap()), c)
	})
	.collect()
}

fn check_wikipedia_result(flows: EKFlows<char, i32>) {
	let (caps, total, ..) = flows;
	assert_eq!(caps.len(), 8);
	let caps = caps.into_iter().collect::<HashMap<(char, char), i32>>();
	assert_eq!(caps[&('A', 'B')], 2);
	assert_eq!(caps[&('A', 'D')], 3);
	assert_eq!(caps[&('B', 'C')], 2);
	assert_eq!(caps[&('C', 'D')], 1);
	assert_eq!(caps[&('C', 'E')], 1);
	assert_eq!(caps[&('D', 'F')], 4);
	assert_eq!(caps[&('E', 'G')], 1);
	assert_eq!(caps[&('F', 'G')], 4);
	assert_eq!(total, 5);
}

fn wikipedia_example(c: &mut Criterion) {
	let mut group = c.benchmark_group(stringify!(wikipedia_example));

	group.bench_function("sparse", |b| {
		b.iter(|| {
			check_wikipedia_result(edmonds_karp_sparse(
				&"ABCDEFGH".chars().collect::<Vec<_>>(),
				&'A',
				&'G',
				successors_wikipedia(),
			));
		});
	});

	group.bench_function("dense", |b| {
		b.iter(|| {
			check_wikipedia_result(edmonds_karp_dense(
				&"ABCDEFGH".chars().collect::<Vec<_>>(),
				&'A',
				&'G',
				successors_wikipedia(),
			));
		});
	});
}

criterion_group!(benches, wikipedia_example);
criterion_main!(benches);
