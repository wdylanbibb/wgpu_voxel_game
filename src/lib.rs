extern crate core;

use std::iter;
use std::path::Path;

use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix4, SquareMatrix, Vector2, Vector3, Vector4, Zero};
use imgui::{Condition, FontSource, MouseCursor};
use imgui_wgpu::{RendererConfig};
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::chunk::{DrawChunk, Vertex};
use crate::renderer::{Renderer};
use crate::resources::get_bytes;

mod block;
mod camera;
mod chunk;
mod material;
mod texture;
mod trait_enum;
mod renderer;
mod resources;

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

struct State {
    renderer: Renderer,
    imgui: imgui::Context,
    platform: imgui_winit_support::WinitPlatform,
    gui_renderer: imgui_wgpu::Renderer,
    camera: camera::Camera,
    projection: camera::Projection,
    camera_controller: camera::CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    chunk: chunk::Chunk,
    fps_counter: renderer::FPSCounter,
    last_cursor: Option<MouseCursor>,
    mouse_pressed: bool,
    ui_focus: bool,
}

impl State {
    async fn new(window: &Window) -> Self {
        let renderer = Renderer::new(window).await;

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

        imgui.fonts().add_font(&[
            // FontSource::DefaultFontData {
            //     config: Some(imgui::FontConfig {
            //         size_pixels: font_size,
            //         ..Default::default()
            //     })
            // },
            FontSource::TtfData {
                data: &get_bytes("fonts/Silkscreen-Regular.ttf").unwrap(),
                size_pixels: font_size,
                config: Some(imgui::FontConfig {
                    size_pixels: font_size,
                    ..Default::default()
                })
            },
        ]);

        imgui.fonts().add_font(&[
            FontSource::TtfData {
                data: &get_bytes("fonts/Silkscreen-Bold.ttf").unwrap(),
                size_pixels: font_size,
                config: Some(imgui::FontConfig {
                    size_pixels: font_size,
                    ..Default::default()
                })
            },
        ]);

        let renderer_config = RendererConfig {
            texture_format: renderer.config.format,
            ..Default::default()
        };

        let gui_renderer = imgui_wgpu::Renderer::new(&mut imgui, &renderer.device, &renderer.queue, renderer_config);

        let texture_bind_group_layout =
            renderer.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            });

        let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection =
            camera::Projection::new(renderer.config.width, renderer.config.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = camera::CameraController::new(16.0, 0.4);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            renderer.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let camera_bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera bind group"),
        });

        let render_pipeline_layout =
            renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
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

        let mut chunk = {
            let material = material::Material::new(
                "Atlas Mat",
                texture::Texture::new(Path::new("sprite_atlas.png"), false, &renderer.device, &renderer.queue),
                &renderer.device,
                &texture_bind_group_layout,
            );

            chunk::Chunk::new(material, &renderer.device)
        };

        for x in 0..10 {
            for z in 0..10 {
                chunk.set_block(Vector3::new(x, 0, z), block::Block::Grass(block::Grass), &renderer.queue);
            }
        }

        chunk.set_block(Vector3::new(5, 1, 5), block::Block::Stone(block::Stone), &renderer.queue);

        chunk.set_block(Vector3::new(3, 0, 3), block::Block::Air(block::Air), &renderer.queue);

        let fps_counter = renderer::FPSCounter::new();

        Self {
            renderer,
            imgui,
            platform,
            gui_renderer,
            camera,
            projection,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            render_pipeline,
            chunk,
            fps_counter,
            last_cursor: None,
            mouse_pressed: false,
            ui_focus: false,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.renderer.size = new_size;

            self.projection.resize(new_size.width, new_size.height);

            self.renderer.config.width = new_size.width;
            self.renderer.config.height = new_size.height;

            self.renderer.surface.configure(&self.renderer.device, &self.renderer.config);

            self.renderer.depth_texture =
                texture::Texture::create_depth_texture(&self.renderer.device, &self.renderer.config, "depth texture");
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

        self.fps_counter.tick();
    }

    fn render(&mut self, window: &Window) -> Result<(), wgpu::SurfaceError> {
        let output = self.renderer.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.render_world(&view)?;

        self.render_gui(&view, window)?;

        output.present();

        Ok(())
    }

    fn render_world(&mut self, view: &wgpu::TextureView) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self
            .renderer.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
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
                    view: &self.renderer.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_pipeline(&self.render_pipeline);

            // self.chunk.mesh.draw(&mut render_pass, &self.camera_bind_group);
            render_pass.draw_chunk(&self.chunk, &self.camera_bind_group);
        }

        self.renderer.queue.submit(iter::once(encoder.finish()));

        Ok(())
    }

    fn render_gui(&mut self, view: &wgpu::TextureView, window: &Window) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self
            .renderer.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.platform
            .prepare_frame(self.imgui.io_mut(), window)
            .expect("Failed to prepare frame");

        let bold_font = self.imgui.fonts().fonts()[1];

        let ui: imgui::Ui = self.imgui.frame();

        self.ui_focus = ui.io().want_capture_mouse;

        if self.last_cursor != ui.mouse_cursor() {
            self.last_cursor = ui.mouse_cursor();
            self.platform.prepare_render(&ui, window);
        }

        let _ = imgui::Window::new("Game Info")
            .size(Vector2::zero().into(), Condition::FirstUseEver)
            .position(Vector2::new(0.0, 0.0).into(), Condition::FirstUseEver)
            .resizable(false)
            // .movable(false)
            .title_bar(false)
            .always_auto_resize(true)
            .build(&ui, || {
                let bold = ui.push_font(bold_font);
                ui.text("Debug Info");
                bold.pop();
                ui.separator();
                ui.text(format!("FPS: {:?}", self.fps_counter.last_second_frames.len()));
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.gui_renderer
                .render(ui.render(), &self.renderer.queue, &self.renderer.device, &mut render_pass)
                .expect("Rendering failed");
        }

        self.renderer.queue.submit(iter::once(encoder.finish()));

        Ok(())
    }
}

pub async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Voxel Game")
        .with_inner_size(PhysicalSize::new(1280, 720))
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
                if state.mouse_pressed && !state.ui_focus {
                    state.camera_controller.process_mouse(delta.0, delta.1)
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let now = instant::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;

                state.imgui.io_mut().update_delta_time(dt);

                state.update(dt.as_secs_f32());
                match state.render(&window) {
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

        state.platform.handle_event(state.imgui.io_mut(), &window, &event);
    });
}
