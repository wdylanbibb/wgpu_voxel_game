use bevy_ecs::event::{EventReader, Events, EventWriter, ManualEventReader};
use cgmath::Vector2;
use winit::event::{DeviceEvent, Event, MouseScrollDelta};
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};

use crate::engine::engine::{Engine, EngineExit, Module};
use crate::engine::input::keyboard::KeyboardInput;
use crate::engine::input::mouse::{MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel};
use crate::engine::window::event::{CursorEntered, CursorLeft, CursorMoved, FileDragAndDrop, ReceivedCharacter, RequestRedraw, WindowBackendScaleFactorChanged, WindowClosed, WindowCloseRequested, WindowFocused, WindowMoved, WindowResized, WindowScaleFactorChanged};
use crate::engine::window::window::WindowContainer;

pub mod event;
pub mod window;

pub struct WindowModule;

impl Module for WindowModule {
	fn build(&self, engine: &mut Engine) {
		engine
			.add_event::<WindowResized>()
			.add_event::<WindowClosed>()
			.add_event::<WindowCloseRequested>()
			.add_event::<RequestRedraw>()
			.add_event::<CursorMoved>()
			.add_event::<CursorEntered>()
			.add_event::<CursorLeft>()
			.add_event::<ReceivedCharacter>()
			.add_event::<WindowFocused>()
			.add_event::<WindowScaleFactorChanged>()
			.add_event::<WindowBackendScaleFactorChanged>()
			.add_event::<FileDragAndDrop>()
			.add_event::<WindowMoved>()
			.set_runner(window_runner)
			.add_system(close_when_requested);

		let event_loop = EventLoop::new();

		engine.insert_non_send_resource(event_loop);
	}
}

pub fn close_when_requested(
	mut closed: EventReader<WindowCloseRequested>,
	mut event_writer: EventWriter<EngineExit>,
) {
	for _ in closed.iter() {
		event_writer.send(EngineExit);
	}
}

fn window_runner(mut engine: Engine) {
	env_logger::init();

	let event_loop = engine
		.world
		.remove_non_send_resource::<EventLoop<()>>()
		.unwrap();

	let window = WindowContainer::create_window(&event_loop);

	engine.insert_non_send_resource(window);

	let mut last_render_time = instant::Instant::now();

	let mut redraw_event_reader = ManualEventReader::<RequestRedraw>::default();
	let mut engine_exit_event_reader = ManualEventReader::<EngineExit>::default();

	event_loop.run(move |event, _, control_flow| {
		match event {
			Event::WindowEvent {
				event,
				window_id,
				..
			} => {
				let world = engine.world.cell();

				let window = world.non_send_resource_mut::<WindowContainer>();

				match event {
					WindowEvent::Resized(size) => {
						let mut resize_events = world.resource_mut::<Events<WindowResized>>();
						resize_events.send(WindowResized {
							width: size.width as f32,
							height: size.height as f32,
						});
					}
					WindowEvent::CloseRequested => {
						let mut window_close_request_events =
							world.resource_mut::<Events<WindowCloseRequested>>();
						window_close_request_events.send(WindowCloseRequested);
					}
					WindowEvent::KeyboardInput { input, .. } => {
						let mut keyboard_input_events = world.resource_mut::<Events<KeyboardInput>>();
						keyboard_input_events.send(input.into())
					}
					WindowEvent::CursorMoved { position, .. } => {
						let mut cursor_moved_events = world.resource_mut::<Events<CursorMoved>>();

						let physical_position = Vector2::new(position.x as f32, position.y as f32);

						cursor_moved_events.send(CursorMoved {
							position: physical_position / window.scale_factor() as f32,
						});
					}
					WindowEvent::CursorEntered { .. } => {
						let mut cursor_entered_events = world.resource_mut::<Events<CursorEntered>>();
						cursor_entered_events.send(CursorEntered);
					}
					WindowEvent::CursorLeft { .. } => {
						let mut cursor_left_events = world.resource_mut::<Events<CursorLeft>>();
						cursor_left_events.send(CursorLeft);
					}
					WindowEvent::MouseInput { state, button, .. } => {
						let mut mouse_button_input_events = world.resource_mut::<Events<MouseButtonInput>>();
						mouse_button_input_events.send(MouseButtonInput {
							button: button.into(),
							state: state.into(),
						})
					}
					WindowEvent::MouseWheel { delta, .. } => match delta {
						MouseScrollDelta::LineDelta(x, y) => {
							let mut mouse_whell_input_events = world.resource_mut::<Events<MouseWheel>>();
							mouse_whell_input_events.send(MouseWheel {
								unit: MouseScrollUnit::Line,
								x,
								y,
							});
						}
						MouseScrollDelta::PixelDelta(p) => {
							let mut mouse_wheel_input_events = world.resource_mut::<Events<MouseWheel>>();
							mouse_wheel_input_events.send(MouseWheel {
								unit: MouseScrollUnit::Pixel,
								x: p.x as f32,
								y: p.y as f32,
							});
						}
					}
					WindowEvent::ReceivedCharacter(c) => {
						let mut char_input_events = world.resource_mut::<Events<ReceivedCharacter>>();
						char_input_events.send(ReceivedCharacter {
							char: c,
						});
					}
					WindowEvent::ScaleFactorChanged {
						scale_factor,
						new_inner_size,
					} => {
						let mut backend_scale_factor_change_events =
							world.resource_mut::<Events<WindowBackendScaleFactorChanged>>();
						backend_scale_factor_change_events.send(WindowBackendScaleFactorChanged {
							scale_factor
						});
					}
					WindowEvent::Focused(focused) => {
						let mut focused_events = world.resource_mut::<Events<WindowFocused>>();
						focused_events.send(WindowFocused {
							focused,
						});
					}
					WindowEvent::DroppedFile(path_buf) => {
						let mut events = world.resource_mut::<Events<FileDragAndDrop>>();
						events.send(FileDragAndDrop::DroppedFile { path_buf });
					}
					WindowEvent::HoveredFile(path_buf) => {
						let mut events = world.resource_mut::<Events<FileDragAndDrop>>();
						events.send(FileDragAndDrop::HoveredFile { path_buf });
					}
					WindowEvent::HoveredFileCancelled => {
						let mut events = world.resource_mut::<Events<FileDragAndDrop>>();
						events.send(FileDragAndDrop::HoveredFileCancelled);
					}
					WindowEvent::Moved(position) => {
						let position = Vector2::new(position.x, position.y);
						let mut events = world.resource_mut::<Events<WindowMoved>>();
						events.send(WindowMoved {
							position,
						});
					}
					_ => (),
				}
			}
			Event::DeviceEvent {
				event: DeviceEvent::MouseMotion { delta },
				..
			} => {
				let mut mouse_motion_events = engine.world.resource_mut::<Events<MouseMotion>>();
				mouse_motion_events.send(MouseMotion {
					delta: Vector2::new(delta.0 as f32, delta.1 as f32),
				});
			}
			Event::MainEventsCleared => {
				engine.update();
			}
			Event::RedrawEventsCleared => {
				if let Some(redraw_events) = engine.world.get_resource::<Events<RequestRedraw>>() {
					if redraw_event_reader.iter(redraw_events).last().is_some() {
						*control_flow = ControlFlow::Poll;
					}
				}
				if let Some(exit_events) = engine.world.get_resource::<Events<EngineExit>>() {
					if engine_exit_event_reader.iter(exit_events).last().is_some() {
						*control_flow = ControlFlow::Exit;
					}
				}
			}
			_ => (),
		}
	});
}
