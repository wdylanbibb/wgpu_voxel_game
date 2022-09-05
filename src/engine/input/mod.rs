use bevy_ecs::schedule::{ParallelSystemDescriptorCoercion, SystemLabel};
use winit::event::ElementState;

use crate::engine::engine::{CoreStage, Engine, Module};
use crate::engine::input::input::Input;
use crate::engine::input::keyboard::{keyboard_input_system, KeyboardInput, KeyCode, ScanCode};
use crate::engine::input::mouse::{mouse_button_input_system, MouseButton, MouseButtonInput, MouseMotion, MouseWheel};

pub mod keyboard;
pub mod input;
pub mod mouse;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ButtonState {
	Pressed,
	Released,
}

impl ButtonState {
	pub fn is_pressed(&self) -> bool {
		matches!(self, ButtonState::Pressed)
	}
}

impl From<winit::event::ElementState> for ButtonState {
	fn from(state: ElementState) -> Self {
		match state {
			ElementState::Pressed => ButtonState::Pressed,
			ElementState::Released => ButtonState::Released,
		}
	}
}

#[derive(Default)]
pub struct InputModule;

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemLabel)]
pub struct InputSystem;

impl Module for InputModule {
	fn build(&self, engine: &mut Engine) {
		engine
			// keyboard
			.add_event::<KeyboardInput>()
			.init_resource::<Input<KeyCode>>()
			.init_resource::<Input<ScanCode>>()
			.add_system_to_stage(
				CoreStage::PreUpdate,
				keyboard_input_system.label(InputSystem),
			)
			// mouse
			.add_event::<MouseButtonInput>()
			.add_event::<MouseMotion>()
			.add_event::<MouseWheel>()
			.init_resource::<Input<MouseButton>>()
			.add_system_to_stage(
				CoreStage::PreUpdate,
				mouse_button_input_system.label(InputSystem),
			);
	}
}