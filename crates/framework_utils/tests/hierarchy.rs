use bevy_ecs::{prelude::*, system::RunSystemOnce};
use bevy_framework_utils::hierarchy::HierarchyQuery;
use bevy_hierarchy::prelude::*;

#[derive(Component)]
#[repr(transparent)]
struct A(usize);

#[derive(Component)]
#[repr(transparent)]
struct X;

#[test]
fn ancestors() {
	let mut w = World::new();
	w.spawn(A(0)).with_children(|a| {
		a.spawn(A(1)).with_children(|b| {
			b.spawn(A(2)).with_children(|c| {
				c.spawn(A(3));
				c.spawn(A(4)).with_children(|d| {
					d.spawn(A(5));
				});
			});
			b.spawn(A(6)).with_children(|c| {
				c.spawn((A(7), X));
			});
		});
	});

	let r = w
		.run_system_once(
			move |q: HierarchyQuery<'_, '_>,
			      qa: Query<'_, '_, &A>,
			      qx: Single<'_, Entity, With<X>>| {
				let entity = *qx;
				let mut r = Vec::new();
				for e in q.ancestors(entity) {
					r.push(qa.get(e).unwrap().0);
				}

				r
			},
		)
		.unwrap();

	assert_eq!(r, [6, 1, 0]);
}

#[test]
fn descendants_wide() {
	let mut w = World::new();
	let entity = w
		.spawn(A(0))
		.with_children(|a| {
			a.spawn(A(1)).with_children(|b| {
				b.spawn(A(2)).with_children(|c| {
					c.spawn(A(3));
					c.spawn(A(4)).with_children(|d| {
						d.spawn(A(5));
					});
				});

				b.spawn(A(6)).with_children(|c| {
					c.spawn(A(7));
				});
			});
		})
		.id();

	let r = w
		.run_system_once(move |q: HierarchyQuery<'_, '_>, qa: Query<'_, '_, &A>| {
			let mut r = Vec::new();
			for e in q.descendants_wide(entity) {
				r.push(qa.get(e).unwrap().0);
			}

			r
		})
		.unwrap();

	assert_eq!(r, [1, 2, 6, 3, 4, 7, 5]);
}

#[test]
fn descendants_deep() {
	let mut w = World::new();
	let entity = w
		.spawn(A(0))
		.with_children(|a| {
			a.spawn(A(1)).with_children(|b| {
				b.spawn(A(2)).with_children(|c| {
					c.spawn(A(3));
					c.spawn(A(4)).with_children(|d| {
						d.spawn(A(5));
					});
				});

				b.spawn(A(6)).with_children(|c| {
					c.spawn(A(7));
				});
			});
		})
		.id();

	let r = w
		.run_system_once(move |q: HierarchyQuery<'_, '_>, qa: Query<'_, '_, &A>| {
			let mut r = Vec::new();
			for e in q.descendants_deep(entity) {
				r.push(qa.get(e).unwrap().0);
			}
			r
		})
		.unwrap();

	assert_eq!(r, [1, 2, 3, 4, 5, 6, 7]);
}

#[test]
fn parent_child() {
	let mut w = World::new();

	let e = w
		.spawn_empty()
		.with_children(|parent| {
			parent.spawn_empty();
		})
		.id();

	let pass = w.run_system_once(move |q: HierarchyQuery<'_, '_>| {
		assert_eq!(q.children(e).count(), 1);
		true
	});

	assert!(pass.unwrap());
}
