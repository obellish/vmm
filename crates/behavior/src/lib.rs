#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod events;
mod plugin;
pub mod prelude;
pub mod sequence;
pub mod transition;

use std::{
	fmt::Debug,
	mem::{replace, swap},
	ops::{Deref, DerefMut},
};

use bevy_ecs::{prelude::*, query::QueryData};
use bevy_framework_kind::prelude::*;
use bevy_reflect::prelude::*;
use bevy_utils::tracing::{debug, warn};

pub use self::plugin::{BehaviorPlugin, RegisterableBehavior};
use self::{
	events::BehaviorEventsMut,
	transition::{Transition, TransitionError, TransitionResponse, TransitionResult},
};

#[derive(QueryData)]
pub struct BehaviorRef<T: Behavior> {
	current: &'static T,
	memory: &'static Memory<T>,
}

impl<T: Behavior> BehaviorRefItem<'_, T> {
	#[must_use]
	pub const fn current(&self) -> &T {
		self.current
	}

	#[expect(clippy::missing_const_for_fn)]
	#[must_use]
	pub fn previous(&self) -> Option<&T> {
		self.memory.last()
	}
}

impl<T: Behavior> Deref for BehaviorRefItem<'_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.current()
	}
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct BehaviorMut<T: Behavior> {
	current: Mut<'static, T>,
	memory: &'static mut Memory<T>,
	transition: &'static mut Transition<T>,
	response: Option<&'static mut TransitionResponse<T>>,
}

#[expect(clippy::missing_const_for_fn)]
impl<T: Behavior> BehaviorMutItem<'_, T> {
	#[must_use]
	pub fn current(&self) -> &T {
		&self.current
	}

	pub fn current_mut(&mut self) -> &mut T {
		self.current.as_mut()
	}

	#[must_use]
	pub fn previous(&self) -> Option<&T> {
		self.memory.last()
	}

	pub fn stop(&mut self) {
		self.set_transition(Transition::Previous);
	}

	pub fn reset(&mut self) {
		self.set_transition(Transition::Reset);
	}

	fn set_transition(&mut self, transition: Transition<T>) {
		let previous = replace(self.transition.as_mut(), transition);
		if !previous.is_none() {
			warn!("transition override: {previous:?} -> {:?}", self.transition);
		}
	}

	fn set_result(&mut self, result: TransitionResult<T>) {
		if let Some(response) = self.response.as_mut() {
			response.set(result);
		}
	}

	fn push(&mut self, instance: Instance<T>, mut next: T, events: &mut BehaviorEventsMut<'_, T>) {
		if self.allows_next(&next) {
			let previous = {
				swap(self.current.as_mut(), &mut next);
				next
			};
			if previous.is_resumable() {
				events.pause(instance);
				self.memory.push(previous);
			} else {
				events.stop(instance, previous);
			}

			events.start(instance);
			self.set_result(Ok(()));
		} else {
			warn!(
				"{instance:?}: transition {:?} -> {next:?} is not allowed",
				*self.current
			);
			self.set_result(Err(TransitionError::RejectedNext(next)));
		}
	}

	fn pop(&mut self, instance: Instance<T>, events: &mut BehaviorEventsMut<'_, T>) {
		if let Some(mut previous) = self.memory.pop() {
			debug!("{instance:?}: {:?} -> {previous:?}", *self.current);
			let previous = {
				swap(self.current.as_mut(), &mut previous);
				previous
			};
			events.resume(instance);
			events.stop(instance, previous);
			self.set_result(Ok(()));
		} else {
			warn!(
				"{instance:?}: transition {:?} -> None is not allowed",
				*self.current
			);
			self.set_result(Err(TransitionError::NoPrevious));
		}
	}

	fn clear(&mut self, instance: Instance<T>, events: &mut BehaviorEventsMut<'_, T>) {
		while self.memory.len() > 1 {
			let previous = self.memory.pop().unwrap();
			events.stop(instance, previous);
		}

		self.pop(instance, events);
	}
}

impl<T: Behavior> Deref for BehaviorMutItem<'_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.current()
	}
}

impl<T: Behavior> DerefMut for BehaviorMutItem<'_, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.current_mut()
	}
}

#[expect(clippy::missing_const_for_fn)]
impl<T: Behavior> BehaviorMutReadOnlyItem<'_, T> {
	#[must_use]
	pub fn current(&self) -> &T {
		&self.current
	}

	#[must_use]
	pub fn previous(&self) -> Option<&T> {
		self.memory.last()
	}
}

impl<T: Behavior> Deref for BehaviorMutReadOnlyItem<'_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.current()
	}
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Memory<T: Behavior> {
	stack: Vec<T>,
}

impl<T: Behavior> Default for Memory<T> {
	fn default() -> Self {
		Self { stack: Vec::new() }
	}
}

impl<T: Behavior> Deref for Memory<T> {
	type Target = Vec<T>;

	fn deref(&self) -> &Self::Target {
		&self.stack
	}
}

impl<T: Behavior> DerefMut for Memory<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.stack
	}
}

pub trait Behavior: Component + Debug {
	fn allows_next(&self, next: &Self) -> bool {
		let _ = next;
		true
	}

	fn is_resumable(&self) -> bool {
		true
	}
}
