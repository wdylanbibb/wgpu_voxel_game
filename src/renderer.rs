//! A Frames Per Second counter.

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use winit::dpi::PhysicalSize;
use winit::window::Window;
use crate::texture::Texture;

pub struct Renderer {
	pub surface: wgpu::Surface,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
	pub config: wgpu::SurfaceConfiguration,
	pub size: PhysicalSize<u32>,

	pub depth_texture: Texture,
}

impl Renderer {
	pub async fn new(window: &Window) -> Self {
		let size = window.inner_size();

		// The instance is a handle to our GPU
		// BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
		let instance = wgpu::Instance::new(wgpu::Backends::all());
		let surface = unsafe { instance.create_surface(window) };
		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			})
			.await
			.unwrap();

		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					label: None,
					features: wgpu::Features::empty(),
					limits: wgpu::Limits::default(),
				},
				// Some(&std::path::Path::new("trace")), // Trace path
				None,
			)
			.await
			.unwrap();

		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface.get_supported_formats(&adapter)[0],
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::Fifo,
		};
		surface.configure(&device, &config);

		let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

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

#[derive(Debug)]
pub struct FPSCounter {
	pub last_second_frames: VecDeque<Instant>
}

impl FPSCounter {
	pub fn new() -> FPSCounter {
		FPSCounter {
			last_second_frames: VecDeque::with_capacity(128)
		}
	}

	pub fn tick(&mut self) -> usize {
		let now = Instant::now();
		let a_second_ago = now - Duration::from_secs(1);

		while self.last_second_frames.front().map_or(false, |t| *t < a_second_ago) {
			self.last_second_frames.pop_front();
		}

		self.last_second_frames.push_back(now);
		self.last_second_frames.len()
	}
}


pub(crate) fn create_render_pipeline(
	device: &wgpu::Device,
	layout: &wgpu::PipelineLayout,
	color_format: wgpu::TextureFormat,
	depth_format: Option<wgpu::TextureFormat>,
	vertex_layouts: &[wgpu::VertexBufferLayout],
	shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
	let shader = device.create_shader_module(shader);

	device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: Some("Render Pipeline"),
		layout: Some(layout),
		vertex: wgpu::VertexState {
			module: &shader,
			entry_point: "vs_main",
			buffers: vertex_layouts,
		},
		fragment: Some(wgpu::FragmentState {
			module: &shader,
			entry_point: "fs_main",
			targets: &[Some(wgpu::ColorTargetState {
			    format: color_format,
			    blend: Some(wgpu::BlendState {
			        alpha: wgpu::BlendComponent::OVER,
			        color: wgpu::BlendComponent::OVER,
			    }),
			    write_mask: wgpu::ColorWrites::ALL,
			})],
			// targets: &[Some(color_format.into())],
		}),
		primitive: wgpu::PrimitiveState {
			topology: wgpu::PrimitiveTopology::TriangleList,
			strip_index_format: None,
			front_face: wgpu::FrontFace::Ccw,
			cull_mode: Some(wgpu::Face::Back),
			polygon_mode: wgpu::PolygonMode::Fill,
			unclipped_depth: false,
			conservative: false,
			..Default::default()
		},
		depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
			format,
			depth_write_enabled: true,
			depth_compare: wgpu::CompareFunction::Less,
			stencil: wgpu::StencilState::default(),
			bias: wgpu::DepthBiasState::default(),
		}),
		// multisample: wgpu::MultisampleState {
		//     count: 1,
		//     mask: !0,
		//     alpha_to_coverage_enabled: false,
		// },
		multisample: wgpu::MultisampleState::default(),
		multiview: None,
	})
}
