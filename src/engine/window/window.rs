pub struct WindowContainer(pub winit::window::Window);

impl WindowContainer {
	pub fn create_window(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
		let window_builder = winit::window::WindowBuilder::new();

		// TODO: Create WindowSettings resource and add it to engine.world
		let window_builder = window_builder
			.with_title("Voxel Game")
			.with_inner_size(winit::dpi::PhysicalSize::new(1280, 720));

		let window = window_builder.build(event_loop).unwrap();

		Self(window)
	}

	pub fn inner_size(&self) -> winit::dpi::PhysicalSize<u32> {
		self.0.inner_size()
	}

	pub fn scale_factor(&self) -> f64 {
		self.0.scale_factor()
	}
}