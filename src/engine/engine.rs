use std::default::Default;

use bevy_ecs::event::{Event, Events};
use bevy_ecs::schedule::{IntoSystemDescriptor, Schedule, ShouldRun, Stage, StageLabel, SystemStage};
use bevy_ecs::system::{IntoExclusiveSystem, Resource};
use bevy_ecs::world::{FromWorld, World};

pub trait Module {
	fn build(&self, engine: &mut Engine);
}

#[derive(StageLabel)]
pub enum CoreStage {
	First,
	PreUpdate,
	Update,
	PostUpdate,
	Render,
	PostRender,
	Last,
}

#[derive(StageLabel)]
pub struct StartupSchedule;

#[derive(StageLabel)]
pub enum StartupStage {
	PreStartup,
	Startup,
	PostStartup,
}

pub struct Engine {
	pub world: World,
	pub schedule: Schedule,
	/// The runner function is responsible for the engine's event loop,
	/// as to allow for the Engine's schedule being run outside of itself
	pub runner: Box<dyn Fn(Engine)>,
}

impl Default for Engine {
	fn default() -> Self {
		let mut engine = Engine::empty();
		engine.add_default_stages()
			.add_event::<EngineExit>()
			.add_system_to_stage(CoreStage::Last, World::clear_trackers.exclusive_system());
		engine
	}
}

impl Engine {
	pub fn new() -> Self {
		Engine::default()
	}

	pub fn empty() -> Self {
		Self {
			world: Default::default(),
			schedule: Default::default(),
			runner: Box::new(run_once),
		}
	}

	pub fn update(&mut self) {
		self.schedule.run(&mut self.world);
	}

	pub fn run(&mut self) {
		let mut engine = std::mem::replace(self, Engine::empty());
		let runner = std::mem::replace(&mut engine.runner, Box::new(run_once));
		(runner)(engine)
	}

	pub fn add_stage<S: Stage>(&mut self, label: impl StageLabel, stage: S) -> &mut Self {
		self.schedule.add_stage(label, stage);
		self
	}

	pub fn add_default_stages(&mut self) -> &mut Self {
		self.add_stage(CoreStage::First, SystemStage::parallel())
			.add_stage(
				StartupSchedule,
				Schedule::default()
					.with_run_criteria(ShouldRun::once)
					.with_stage(StartupStage::PreStartup, SystemStage::parallel())
					.with_stage(StartupStage::Startup, SystemStage::parallel())
					.with_stage(StartupStage::PostStartup, SystemStage::parallel()),
			)
			.add_stage(CoreStage::PreUpdate, SystemStage::parallel())
			.add_stage(CoreStage::Update, SystemStage::parallel())
			.add_stage(CoreStage::PostUpdate, SystemStage::parallel())
			.add_stage(CoreStage::Render, SystemStage::parallel())
			.add_stage(CoreStage::PostRender, SystemStage::parallel())
			.add_stage(CoreStage::Last, SystemStage::parallel())
	}

	pub fn set_runner(&mut self, run_fn: impl Fn(Engine) + 'static) -> &mut Self {
		self.runner = Box::new(run_fn);
		self
	}

	pub fn add_module<T: Module>(&mut self, module: T) -> &mut Self {
		module.build(self);
		self
	}

	pub fn add_system<Params>(&mut self, system: impl IntoSystemDescriptor<Params>) -> &mut Self {
		self.add_system_to_stage(CoreStage::Update, system)
	}

	pub fn add_startup_system<Params>(&mut self, system: impl IntoSystemDescriptor<Params>) -> &mut Self {
		self.add_startup_system_to_stage(StartupStage::Startup, system)
	}

	pub fn add_startup_system_to_stage<Params>(
		&mut self,
		stage_label: impl StageLabel,
		system: impl IntoSystemDescriptor<Params>,
	) -> &mut Self {
		self.schedule.stage(StartupSchedule, |schedule: &mut Schedule| {
			schedule.add_system_to_stage(stage_label, system)
		});
		self
	}

	pub fn add_system_to_stage<Params>(
		&mut self,
		stage_label: impl StageLabel,
		system: impl IntoSystemDescriptor<Params>,
	) -> &mut Self {
		self.schedule.add_system_to_stage(stage_label, system);
		self
	}

	pub fn add_event<T: Event>(&mut self) -> &mut Self {
		if !self.world.contains_resource::<Events<T>>() {
			self.init_resource::<Events<T>>()
				.add_system_to_stage(CoreStage::First, Events::<T>::update_system);
		}
		self
	}

	pub fn init_resource<R: Resource + FromWorld>(&mut self) -> &mut Self {
		self.world.init_resource::<R>();
		self
	}

	pub fn init_non_send_resource<R: 'static + FromWorld>(&mut self) -> &mut Self {
		self.world.init_non_send_resource::<R>();
		self
	}

	pub fn insert_resource(&mut self, resource: impl Resource) -> &mut Self {
		self.world.insert_resource(resource);
		self
	}

	pub fn insert_non_send_resource<R: 'static>(&mut self, resource: R) -> &mut Self {
		self.world.insert_non_send_resource(resource);
		self
	}
}

fn run_once(mut engine: Engine) {
	engine.update();
}

pub struct EngineExit;