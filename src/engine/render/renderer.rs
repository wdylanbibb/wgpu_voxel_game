use wgpu::{Backends, Device, DeviceDescriptor, Features, Instance, Limits, PowerPreference, PresentMode, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration, TextureUsages};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::texture::Texture;

pub struct Renderer {
	pub surface: Surface,
	pub device: Device,
	pub queue: Queue,
	pub config: SurfaceConfiguration,
	pub size: PhysicalSize<u32>,

	pub depth_texture: Texture,
}

impl Renderer {
	pub fn new(window: &Window) -> Self {
		let size = window.inner_size();

		// The instance is a handle to our GPU
		// Backends::all() => Vulkan + Metal + DX12 + Browser WebGPU
		let instance = Instance::new(Backends::all());
		let surface = unsafe { instance.create_surface(window) };
		let adapter = pollster::block_on(instance.request_adapter(
			&RequestAdapterOptions {
				power_preference: PowerPreference::default(),
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			}
		)).unwrap();

		let (device, queue) = pollster::block_on(adapter.request_device(
			&DeviceDescriptor {
				label: None,
				features: Features::empty(),
				limits: Limits::default(),
			},
			None,
		)).unwrap();

		let config = SurfaceConfiguration {
			usage: TextureUsages::RENDER_ATTACHMENT,
			format: surface.get_supported_formats(&adapter)[0],
			width: size.width,
			height: size.height,
			present_mode: PresentMode::Fifo,
		};
		surface.configure(&device, &config);

		let depth_texture = Texture::create_depth_texture(&device, &config, "depth texture");

		Self {
			surface,
			device,
			queue,
			config,
			size,
			depth_texture,
		}
	}
}