use std::path::PathBuf;

use cgmath::Vector2;

pub struct WindowResized {
	pub width: f32,
	pub height: f32,
}

pub struct WindowCloseRequested;

pub struct RequestRedraw;

pub struct WindowClosed;

pub struct CursorMoved {
	pub position: Vector2<f32>,
}

pub struct CursorEntered;

pub struct CursorLeft;

pub struct ReceivedCharacter {
	pub char: char,
}

pub struct WindowFocused {
	pub focused: bool,
}

pub struct WindowScaleFactorChanged {
	pub scale_factor: f64,
}

pub struct WindowBackendScaleFactorChanged {
	pub scale_factor: f64,
}

pub enum FileDragAndDrop {
	DroppedFile { path_buf: PathBuf },
	HoveredFile { path_buf: PathBuf },
	HoveredFileCancelled,
}

pub struct WindowMoved {
	pub position: Vector2<i32>,
}