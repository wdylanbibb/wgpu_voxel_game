extern crate core;

mod mesh;
mod texture;
mod camera;

use std::iter;
use std::path::Path;
use bytemuck::{Pod, Zeroable};
use cgmath::{InnerSpace, Matrix4, Quaternion, Rotation3, SquareMatrix, Vector3, Vector4};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};
use wgpu::util::DeviceExt;

use winit::{
	event::*,
	event_loop::{ControlFlow, EventLoop},
	window::{Window, WindowBuilder},
	dpi::PhysicalSize,
};
use crate::mesh::{DrawMesh, Vertex};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct CameraUniform {
	view_position: Vector4<f32>,
	view_proj: Matrix4<f32>,
}

unsafe impl Pod for CameraUniform {}
unsafe impl Zeroable for CameraUniform {}

impl CameraUniform {
	fn new() -> Self {
		Self {
			view_position: Vector4::new(0.0, 0.0, 0.0, 0.0),
			view_proj: Matrix4::identity(),
		}
	}

	fn update_view_proj(&mut self, camera: &camera::Camera, projection: &camera::Projection) {
		self.view_position = camera.position.to_homogeneous();
		self.view_proj = projection.calc_matrix() * camera.calc_matrix();
	}
}

struct Instance {
	position: Vector3<f32>,
	rotation: Quaternion<f32>,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct InstanceRaw {
	model: Matrix4<f32>,
}

unsafe impl Pod for InstanceRaw {}
unsafe impl Zeroable for InstanceRaw {}

impl Instance {
	fn to_raw(&self) -> InstanceRaw {
		let model = Matrix4::from_translation(self.position) * Matrix4::from(self.rotation);
		InstanceRaw {
			model,
		}
	}
}

impl InstanceRaw {
	fn desc<'a>() -> VertexBufferLayout<'a> {
		static ATTRIBS: [VertexAttribute; 4] = wgpu::vertex_attr_array![5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4];
		use std::mem;
		VertexBufferLayout {
			array_stride: mem::size_of::<InstanceRaw>() as BufferAddress,
			step_mode: VertexStepMode::Instance,
			attributes: &ATTRIBS,
		}
	}
}

const NUM_INSTANCES_PER_ROW: u32 = 1;

struct State {
	surface: wgpu::Surface,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
	size: PhysicalSize<u32>,
	camera: camera::Camera,
	projection: camera::Projection,
	camera_controller: camera::CameraController,
	camera_uniform: CameraUniform,
	camera_buffer: wgpu::Buffer,
	camera_bind_group: wgpu::BindGroup,
	render_pipeline: wgpu::RenderPipeline,
	mesh: mesh::Mesh,
	instances: Vec<Instance>,
	instance_buffer: wgpu::Buffer,
	depth_texture: texture::Texture,
	mouse_pressed: bool,
}

impl State {
	async fn new(window: &Window) -> Self {
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
					limits: wgpu::Limits::default()
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

		let texture_bind_group_layout = device.create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Texture {
							multisampled: false,
							view_dimension: wgpu::TextureViewDimension::D2,
							sample_type: wgpu::TextureSampleType::Float { filterable: true },
						},
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
						count: None,
					},
				],
				label: Some("texture bind group layout"),
			}
		);

		let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
		let projection = camera::Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
		let camera_controller = camera::CameraController::new(4.0, 0.4);

		let mut camera_uniform = CameraUniform::new();
		camera_uniform.update_view_proj(&camera, &projection);

		let camera_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Camera Buffer"),
				contents: bytemuck::cast_slice(&[camera_uniform]),
				usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			}
		);

		let camera_bind_group_layout = device.create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None,
						},
						count: None,
					}
				],
				label: Some("camera bind layout group"),
			}
		);

		let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &camera_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: camera_buffer.as_entire_binding(),
				}
			],
			label: Some("camera bind group"),
		});

		let render_pipeline_layout = device.create_pipeline_layout(
			&wgpu::PipelineLayoutDescriptor {
				bind_group_layouts: &[
					&texture_bind_group_layout,
					&camera_bind_group_layout,
				],
				push_constant_ranges: &[],
				label: Some("render pipeline layout"),
			}
		);

		let render_pipeline = {
			let shader = wgpu::ShaderModuleDescriptor {
				source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
				label: Some("Texture Shader"),
			};
			create_render_pipeline(
				&device,
				&render_pipeline_layout,
				config.format,
				Some(texture::Texture::DEPTH_FORMAT),
				&[mesh::MeshVertex::desc(), InstanceRaw::desc()],
				shader,
			)
		};

		let mesh = {
			let material = mesh::Material::new(
				"Cobble Mat",
				texture::Texture::new(Path::new("cobblestone.png"), false, &device, &queue),
				&device,
				&texture_bind_group_layout,
			);

			mesh::Mesh::cube("Cube", &device, material)
		};

		const SPACE_BETWEEN: f32 = 2.0;
		let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
			(0..NUM_INSTANCES_PER_ROW).map(move |x| {
				let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
				let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

				let position = Vector3 { x, y: 0.0, z };

				let rotation = Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(0.0));

				Instance {
					position,
					rotation
				}
			})
		}).collect::<Vec<_>>();

		let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
		let instance_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Instance Buffer"),
				contents: bytemuck::cast_slice(&instance_data),
				usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			}
		);

		let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth texture");

		Self {
			surface,
			device,
			queue,
			config,
			size,
			camera,
			projection,
			camera_controller,
			camera_uniform,
			camera_buffer,
			camera_bind_group,
			render_pipeline,
			mesh,
			instances,
			instance_buffer,
			depth_texture,
			mouse_pressed: false
		}
	}

	pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
		if new_size.width > 0 && new_size.height > 0 {
			self.size = new_size;

			self.projection.resize(new_size.width, new_size.height);

			self.config.width = new_size.width;
			self.config.height = new_size.height;

			self.surface.configure(&self.device, &self.config);

			self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth texture");
		}
	}

	#[allow(unused_variables)]
	fn input(&mut self, event: &WindowEvent) -> bool {
		match event {
			WindowEvent::KeyboardInput {
				input: KeyboardInput {
					virtual_keycode: Some(key),
					state,
					..
				},
				..
			} => self.camera_controller.process_keyboard(*key, *state),
			WindowEvent::MouseWheel { delta, .. } => {
				self.camera_controller.process_scroll(delta);
				true
			}
			WindowEvent::MouseInput {
				button: MouseButton::Left,
				state,
				..
			} => {
				self.mouse_pressed = *state == ElementState::Pressed;
				true
			}
			_ => false,
		}
	}

	fn update(&mut self, dt: instant::Duration) {
		self.camera_controller.update_camera(&mut self.camera, dt);
		self.camera_uniform.update_view_proj(&self.camera, &self.projection);
		self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));

		// for instance in &mut self.instances {
		// 	let amount = cgmath::Quaternion::from_angle_y(cgmath::Deg(ROTATION_SPEED * dt.as_secs_f32()));
		// 	let current = instance.rotation;
		// 	instance.rotation = amount * current;
		// }
		// let instance_data = self.instances
		// 	.iter()
		// 	.map(Instance::to_raw)
		// 	.collect::<Vec<_>>();
		// self.queue.write_buffer(
		// 	&self.instance_buffer,
		// 	0,
		// 	bytemuck::cast_slice(&instance_data),
		// );
	}

	fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
		let output = self.surface.get_current_texture()?;
		let view = output
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("Render Encoder"),
			});

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.1,
							g: 0.2,
							b: 0.3,
							a: 1.0,
						}),
						store: true,
					},
				})],
				depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
					view: &self.depth_texture.view,
					depth_ops: Some(wgpu::Operations {
						load: wgpu::LoadOp::Clear(1.0),
						store: true,
					}),
					stencil_ops: None,
				}),
			});
			render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

			render_pass.set_pipeline(&self.render_pipeline);
			render_pass.draw_mesh_instanced(
				&self.mesh,
				0..self.instances.len() as u32,
				&self.camera_bind_group,
			);
		}

		self.queue.submit(iter::once(encoder.finish()));
		output.present();

		Ok(())
	}
}

fn create_render_pipeline(
	device: &wgpu::Device,
	layout: &wgpu::PipelineLayout,
	color_format: wgpu::TextureFormat,
	depth_format: Option<wgpu::TextureFormat>,
	vertex_layouts: &[VertexBufferLayout],
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
					alpha: wgpu::BlendComponent::REPLACE,
					color: wgpu::BlendComponent::REPLACE,
				}),
				write_mask: wgpu::ColorWrites::ALL,
			})],
		}),
		primitive: wgpu::PrimitiveState {
			topology: wgpu::PrimitiveTopology::TriangleList,
			strip_index_format: None,
			front_face: wgpu::FrontFace::Ccw,
			cull_mode: Some(wgpu::Face::Back),
			polygon_mode: wgpu::PolygonMode::Fill,
			unclipped_depth: false,
			conservative: false,
		},
		depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
			format,
			depth_write_enabled: true,
			depth_compare: wgpu::CompareFunction::Less,
			stencil: wgpu::StencilState::default(),
			bias: wgpu::DepthBiasState::default(),
		}),
		multisample: wgpu::MultisampleState {
			count: 1,
			mask: !0,
			alpha_to_coverage_enabled: false,
		},
		multiview: None
	})
}


pub async fn run() {
	env_logger::init();

	let event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.with_title("Voxel Game")
		.with_inner_size(PhysicalSize::new(1080, 720))
		.build(&event_loop)
		.unwrap();

	// State::new uses async code, so we're going to wait for it to finish
	let mut state = State::new(&window).await;
	let mut last_render_time = instant::Instant::now();

	event_loop.run(move |event, _, control_flow| {
		match event {
			Event::WindowEvent {
				ref event,
				window_id,
			} if window_id == window.id() && !state.input(event) => {
				match event {
					WindowEvent::CloseRequested |
					WindowEvent::KeyboardInput {
						input:
						KeyboardInput {
							state: ElementState::Pressed,
							virtual_keycode: Some(VirtualKeyCode::Escape),
							..
						},
						..
					} => *control_flow = ControlFlow::Exit,
					WindowEvent::Resized(size) => {
						state.resize(*size);
					},
					WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
						state.resize(**new_inner_size);
					},
					_ => {}
				}
			},
			Event::DeviceEvent {
				event: DeviceEvent::MouseMotion { delta, },
				..
			} => if state.mouse_pressed {
				state.camera_controller.process_mouse(delta.0, delta.1)
			}
			Event::RedrawRequested(window_id) if window_id == window.id() => {
				let now = instant::Instant::now();
				let dt = now - last_render_time;
				last_render_time = now;
				state.update(dt);
				match state.render() {
					Ok(_) => {}
					// Reconfigure the surface if lost
					Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
					// The system is out of memory, we should probably quit
					Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
					// All other errors (Outdated, Timeout) should be resolved by the next frame
					Err(e) => eprintln!("{:?}", e),
				}
			},
			Event::MainEventsCleared => {
				// RedrawRequested will only trigger once, unless we manually request it
				window.request_redraw();
			}
			_ => {}
		}
	});
}