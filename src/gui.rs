use imgui::FontSource;
use imgui_wgpu::RendererConfig;

use crate::get_bytes;

pub struct Gui {
	pub imgui: imgui::Context,
	pub platform: imgui_winit_support::WinitPlatform,
	pub gui_renderer: imgui_wgpu::Renderer,

	pub last_cursor: Option<imgui::MouseCursor>,
	pub ui_focus: bool,
}

impl Gui {
	pub fn new(window: &winit::window::Window, config: &wgpu::SurfaceConfiguration, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
		let hidpi_factor = window.scale_factor();

		let mut imgui = imgui::Context::create();
		let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
		platform.attach_window(
			imgui.io_mut(),
			window,
			imgui_winit_support::HiDpiMode::Default,
		);
		imgui.set_ini_filename(None);

		let font_size = (16.0 * hidpi_factor) as f32;
		imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

		imgui.fonts().add_font(&[FontSource::TtfData {
			data: &get_bytes("fonts/Silkscreen-Regular.ttf").unwrap(),
			size_pixels: font_size,
			config: Some(imgui::FontConfig {
				size_pixels: font_size,
				..Default::default()
			}),
		}]);

		imgui.fonts().add_font(&[FontSource::TtfData {
			data: &get_bytes("fonts/Silkscreen-Bold.ttf").unwrap(),
			size_pixels: font_size,
			config: Some(imgui::FontConfig {
				size_pixels: font_size,
				..Default::default()
			}),
		}]);

		let renderer_config = RendererConfig {
			texture_format: config.format,
			..Default::default()
		};

		let gui_renderer = imgui_wgpu::Renderer::new(
			&mut imgui,
			&device,
			&queue,
			renderer_config,
		);

		Gui {
			imgui,
			platform,
			gui_renderer,

			last_cursor: None,
			ui_focus: false,
		}
	}
}