extern crate core;


use std::mem;
use std::path::Path;

use cgmath::{Vector2, Vector3};
use wgpu::util::{align_to, DeviceExt};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::block::Block;
use crate::chunk::{Chunk, CHUNK_DEPTH, CHUNK_WIDTH, ChunkUniform, Vertex};
use crate::gui::Gui;
use crate::renderer::Renderer;
use crate::resources::get_bytes;

mod block;
mod camera;
mod chunk;
mod material;
mod renderer;
mod resources;
mod texture;
mod trait_enum;
mod gui;

struct State {
    renderer: Renderer,
    gui: Gui,
    camera: camera::Camera,
    projection: camera::Projection,

    camera_controller: camera::CameraController,
    camera_uniform: renderer::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    // chunk_uniform_buffer: wgpu::Buffer,
    chunk_uniform_bind_group: wgpu::BindGroup,

    render_pipeline: wgpu::RenderPipeline,
    chunks: Vec<Chunk>,
    mouse_pressed: bool,
}

impl State {
    fn new(window: &Window) -> Self {
        let renderer = Renderer::new(window);

        let gui = Gui::new(window, &renderer.config, &renderer.device, &renderer.queue);

        let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection = camera::Projection::new(
            renderer.config.width,
            renderer.config.height,
            cgmath::Deg(45.0),
            0.1,
            100.0,
        );
        let camera_controller = camera::CameraController::new(16.0, 0.4);

        let mut camera_uniform = renderer::CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_bind_group_layout =
            renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("camera bind layout group"),
                });

        let camera_bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
                label: Some("camera bind group"),
            });

        let chunk_uniform_size = mem::size_of::<ChunkUniform>().next_power_of_two() as wgpu::BufferAddress;
        // Make the `uniform_alignment` >= `chunk_uniform_size` and aligned to `min_uniform_buffer_offset_alignment`
        let uniform_alignment = {
            let alignment = renderer
                .device
                .limits()
                .min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
            align_to(chunk_uniform_size, alignment)
        };

        // Create array of chunks and fill them with blocks
        let chunks = {
            let mut chunks = Vec::new();

            for chunk_x in -1..=1 {
                for chunk_y in -1..=1 {
                    let uniform_offset = (((3 * chunk_x + chunk_y) + 4) as u64 * uniform_alignment) as _;

                    chunks.push(
                        Chunk::new(Vector2::new(chunk_x, chunk_y), uniform_offset, &renderer.device)
                            .with_blocks(
                                (0..16).map(|x| {
                                    (0..16).map(move |z| (Vector3::new(x, (chunk_x+1)+(chunk_y+1), z), Block::grass()))
                                }).flatten().collect::<Vec<(Vector3<i32>, Block)>>(),
                                &renderer.queue
                            ),
                    );
                }
            }

            chunks
        };

        let mut local_buf = encase::DynamicUniformBuffer::new_with_alignment(Vec::new(), uniform_alignment);

        for (i, chunk) in chunks.iter().enumerate() {
            let data = ChunkUniform::new(
                Vector3::new(
                    (chunk.world_offset.x * CHUNK_WIDTH as i32) as f32,
                    0.0,
                    (chunk.world_offset.y * CHUNK_DEPTH as i32) as f32,
                ),
            );

            local_buf.write(&data).unwrap();
        }

        // Note: dynamic uniform offsets also have to be aligned to `Limits::min_uniform_buffer_offset_alignment`.
        let chunk_uniform_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk Uniform Buffer"),
            contents: local_buf.as_ref(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let local_bind_group_layout = renderer.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: wgpu::BufferSize::new(chunk_uniform_size),
                        },
                        count: None,
                    },
                ],
                label: None,
            });

        let diffuse_texture = texture::Texture::new(
            Path::new("sprite_atlas.png"),
            false,
            &renderer.device,
            &renderer.queue,
        );

        let chunk_uniform_bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &local_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &chunk_uniform_buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(chunk_uniform_size),
                    }),
                },
            ],
            label: None,
        });

        let render_pipeline_layout =
            renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    bind_group_layouts: &[&camera_bind_group_layout, &local_bind_group_layout],
                    push_constant_ranges: &[],
                    label: Some("render pipeline layout"),
                });

        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
                label: Some("Texture Shader"),
            };
            renderer::create_render_pipeline(
                &renderer.device,
                &render_pipeline_layout,
                renderer.config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[chunk::ChunkVertex::desc()],
                shader,
            )
        };

        Self {
            renderer,
            gui,
            camera,
            projection,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            // chunk_uniform_buffer,
            chunk_uniform_bind_group,
            render_pipeline,
            chunks,
            mouse_pressed: false,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.renderer.size = new_size;

            self.projection.resize(new_size.width, new_size.height);

            self.renderer.config.width = new_size.width;
            self.renderer.config.height = new_size.height;

            self.renderer
                .surface
                .configure(&self.renderer.device, &self.renderer.config);

            self.renderer.depth_texture = texture::Texture::create_depth_texture(
                &self.renderer.device,
                &self.renderer.config,
                "depth texture",
            );
        }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
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

    fn update(&mut self, dt: f32) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform
            .update_view_proj(&self.camera, &self.projection);
        self.renderer.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        self.renderer.fps_counter.tick();
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // let fps = self.renderer.fps_counter.last_second_frames.len();
        // let bold_font = self.gui.imgui.fonts().fonts()[1];

        // update uniforms
        // for chunk in self.chunks.iter() {
        //     let data = ChunkUniform::new(
        //         Vector3::new(
        //             (chunk.world_offset.x * CHUNK_WIDTH as i32) as f32,
        //             0.0,
        //             (chunk.world_offset.y * CHUNK_DEPTH as i32) as f32,
        //         ),
        //     );
        //
        //     self.renderer.queue.write_buffer(
        //         &self.chunk_uniform_buffer,
        //         chunk.mesh.uniform_offset as wgpu::BufferAddress,
        //         bytemuck::bytes_of(&data),
        //     );
        // }

        self.renderer.render(
            &self.render_pipeline,
            &self.camera_bind_group,
            &self
                .chunks
                .iter()
                .map(|chunk| (&chunk.mesh, &self.chunk_uniform_bind_group))
                .collect::<Vec<_>>(),
        )?;

        Ok(())
    }
}

pub fn run() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Voxel Game")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();
    let mut state = State::new(&window);

    let mut last_render_time = instant::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        state
            .gui.platform
            .handle_event(state.gui.imgui.io_mut(), &window, &event);
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() && !state.input(event) => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
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
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.resize(**new_inner_size);
                }
                _ => {}
            },
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                if state.mouse_pressed && !state.gui.ui_focus {
                    state.camera_controller.process_mouse(delta.0, delta.1)
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let now = instant::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;

                state.gui.imgui.io_mut().update_delta_time(dt);

                state.update(dt.as_secs_f32());
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.renderer.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually request it
                window.request_redraw();
            }
            _ => {}
        }
    });
}
