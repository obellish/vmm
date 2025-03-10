use bevy_ecs::prelude::*;
use bevy_framework_kind::prelude::*;
use bevy_reflect::prelude::*;

use super::{Behavior, BehaviorMut, Memory, events::BehaviorEventsMut, sequence::Sequence};

#[derive(Component, Reflect)]
#[reflect(Component)]
#[repr(transparent)]
pub struct TransitionResponse<T: Behavior> {
	result: Option<TransitionResult<T>>,
}

impl<T: Behavior> TransitionResponse<T> {
	#[expect(clippy::missing_const_for_fn)]
	#[must_use]
	pub fn take(mut this: Mut<'_, Self>) -> Option<TransitionResult<T>> {
		if this.result.is_some() {
			this.result.take()
		} else {
			None
		}
	}

	pub(crate) fn set(&mut self, result: TransitionResult<T>) {
		self.result = Some(result);
	}
}

impl<T: Behavior> Default for TransitionResponse<T> {
	fn default() -> Self {
		Self { result: None }
	}
}

#[derive(Debug, Default, Component, Reflect)]
#[require(Memory<T>)]
#[reflect(Component)]
pub enum Transition<T: Behavior> {
	#[default]
	None,
	Next(T),
	Previous,
	Reset,
}

impl<T: Behavior> Transition<T> {
	pub const fn is_none(&self) -> bool {
		matches!(self, Self::None)
	}

	fn take(&mut self) -> Self {
		std::mem::take(self)
	}
}

impl<T> Clone for Transition<T>
where
	T: Behavior + Clone,
{
	fn clone(&self) -> Self {
		match self {
			Self::None => Self::None,
			Self::Next(next) => Self::Next(next.clone()),
			Self::Previous => Self::Previous,
			Self::Reset => Self::Reset,
		}
	}
}

#[derive(Debug, Reflect)]
pub enum TransitionError<T: Behavior> {
	RejectedNext(T),
	NoPrevious,
}

pub type TransitionChanged<T> = Or<(Changed<Transition<T>>, Changed<Sequence<T>>)>;

pub type TransitionResult<T> = Result<(), TransitionError<T>>;

pub fn transition<T: Behavior>(
	mut events: BehaviorEventsMut<'_, T>,
	mut query: Query<
		'_,
		'_,
		(Instance<T>, BehaviorMut<T>, Option<&mut Sequence<T>>),
		TransitionChanged<T>,
	>,
) {
	for (instance, mut behavior, sequence_opt) in &mut query {
		match behavior.transition.take() {
			Transition::Next(next) => {
				behavior.push(instance, next, &mut events);
				continue;
			}
			Transition::Previous => behavior.pop(instance, &mut events),
			Transition::Reset => behavior.clear(instance, &mut events),
			Transition::None => {
				if behavior.current.is_added() {
					events.start(instance);
				}
			}
		}

		if let Some(next) = sequence_opt.and_then(|mut sequence| sequence.pop()) {
			behavior.push(instance, next, &mut events);
		}
	}
}
