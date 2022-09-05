use bevy_ecs::event::EventReader;
use bevy_ecs::system::ResMut;
use cgmath::Vector2;

use crate::engine::input::ButtonState;
use crate::engine::input::input::Input;

#[derive(Debug, Clone)]
pub struct MouseButtonInput {
	pub button: MouseButton,
	pub state: ButtonState,
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum MouseButton {
	Left,
	Right,
	Middle,
	Other(u16),
}

impl From<winit::event::MouseButton> for MouseButton {
	fn from(button: winit::event::MouseButton) -> Self {
		match button {
			winit::event::MouseButton::Left => MouseButton::Left,
			winit::event::MouseButton::Right => MouseButton::Right,
			winit::event::MouseButton::Middle => MouseButton::Middle,
			winit::event::MouseButton::Other(n) => MouseButton::Other(n),
		}
	}
}

#[derive(Debug, Clone)]
pub struct MouseMotion {
	pub delta: Vector2<f32>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MouseScrollUnit {
	Line,
	Pixel,
}

#[derive(Debug, Clone)]
pub struct MouseWheel {
	pub unit: MouseScrollUnit,
	pub x: f32,
	pub y: f32,
}

pub fn mouse_button_input_system(
	mut mouse_button_input: ResMut<Input<MouseButton>>,
	mut mouse_button_input_events: EventReader<MouseButtonInput>,
) {
	mouse_button_input.clear();
	for event in mouse_button_input_events.iter() {
		match event.state {
			ButtonState::Pressed => mouse_button_input.press(event.button),
			ButtonState::Released => mouse_button_input.release(event.button),
		}
	}
}