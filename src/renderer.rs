//! A Frames Per Second counter.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

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
