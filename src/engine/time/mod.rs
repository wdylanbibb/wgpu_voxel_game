use bevy_ecs::schedule::{ExclusiveSystemDescriptorCoercion, SystemLabel};
use bevy_ecs::system::{IntoExclusiveSystem, ResMut};

use crate::engine::engine::{CoreStage, Engine, Module};
use crate::engine::time::time::Time;

pub mod time;
pub mod timer;
pub mod stopwatch;

pub struct TimeModule;

#[derive(Debug, Eq, PartialEq, Clone, Hash, SystemLabel)]
pub struct TimeSystem;

impl Module for TimeModule {
	fn build(&self, engine: &mut Engine) {
		engine.init_resource::<Time>()
			.add_system_to_stage(
				CoreStage::First,
				time_system.exclusive_system().at_start().label(TimeSystem),
			);
	}
}

fn time_system(
	mut time: ResMut<Time>,
) {
	time.update();
}