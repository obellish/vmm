use std::collections::VecDeque;

use bevy_ecs::prelude::*;
use bevy_reflect::prelude::*;

use super::Behavior;

#[derive(Component, Reflect)]
#[reflect(Component)]
#[repr(transparent)]
pub struct Sequence<T: Behavior> {
	queue: VecDeque<T>,
}

impl<T: Behavior> Sequence<T> {
	#[must_use]
	pub const fn new() -> Self {
		Self {
			queue: VecDeque::new(),
		}
	}

	pub fn start(next: T) -> Self {
		let mut sequence = Self::new();
		sequence.push(next);
		sequence
	}

	#[must_use]
	pub fn then(mut self, next: T) -> Self {
		self.push(next);
		self
	}

	pub fn push(&mut self, next: T) {
		self.queue.push_back(next);
	}

	pub(crate) fn pop(&mut self) -> Option<T> {
		self.queue.pop_front()
	}
}

impl<T: Behavior> Default for Sequence<T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T: Behavior> FromIterator<T> for Sequence<T> {
	fn from_iter<I>(iter: I) -> Self
	where
		I: IntoIterator<Item = T>,
	{
		Self {
			queue: VecDeque::from_iter(iter),
		}
	}
}
