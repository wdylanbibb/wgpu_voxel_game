use std::{collections::HashMap, sync::Arc};

use cgmath::InnerSpace;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::{
    camera,
    renderer::model::{DrawChunk, GpuChunkMesh, Vertex},
    resources,
    world::meshing::CpuChunkMesh,
};

const OUTLINE_VERTICES: [[f32; 3]; 8] = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [1.0, 1.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
    [1.0, 0.0, 1.0],
    [1.0, 1.0, 1.0],
    [0.0, 1.0, 1.0],
];
const OUTLINE_INDICES: [u16; 24] = [
    0, 1, 1, 2, 2, 3, 3, 0, 4, 5, 5, 6, 6, 7, 7, 4, 0, 4, 1, 5, 2, 6, 3, 7,
];
const OUTLINE_ATTRIBUTES: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x3];

pub mod gpu;
pub mod hdr;
pub mod model;
pub mod pipeline;
pub mod texture;

pub struct RenderLayouts {
    pub chunk_material: wgpu::BindGroupLayout,
    pub camera: wgpu::BindGroupLayout,
    pub environment: wgpu::BindGroupLayout,
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    pub(crate) layouts: RenderLayouts,

    chunk_pipeline: wgpu::RenderPipeline,
    sky_pipeline: wgpu::RenderPipeline,
    outline_pipeline: wgpu::RenderPipeline,
    crosshair_pipeline: wgpu::RenderPipeline,

    depth_texture: texture::Texture,
    hdr: hdr::HdrPipeline,
    environment_bind_group: wgpu::BindGroup,
    chunk_material: model::ChunkMaterial,
    chunk_meshes: HashMap<cgmath::Vector3<i32>, model::GpuChunkMesh>,
    outline_vertex_buffer: wgpu::Buffer,
    outline_index_buffer: wgpu::Buffer,
    outline_target_buffer: wgpu::Buffer,
    outline_target_bind_group: wgpu::BindGroup,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        #[cfg(target_arch = "wasm32")]
        anyhow::ensure!(
            wgpu::util::is_browser_webgpu_supported().await,
            "WebGPU is unavailable in this browser. Use a WebGPU-capable browser and serve the app from localhost or HTTPS"
        );

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            // The HDR environment-map conversion uses a compute shader.
            // WebGL has no compute support, so its compute limits are zero.
            backends: wgpu::Backends::BROWSER_WEBGPU,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: true,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        const MAX_RENDER_DIMENSION: u32 = 2048;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.clamp(1, MAX_RENDER_DIMENSION),
            height: size.height.clamp(1, MAX_RENDER_DIMENSION),
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![surface_format.add_srgb_suffix()],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        let is_surface_configured = size.width > 0 && size.height > 0;

        if is_surface_configured {
            surface.configure(&device, &config);
        }

        let chunk_material_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                label: Some("Chunk Material Layout"),
            });

        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("camera_layout"),
        });

        let environment_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("environment_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::Cube,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let outline_target_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("outline_target_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&chunk_material_layout),
                    Some(&camera_layout),
                    Some(&environment_layout),
                ],
                immediate_size: 0,
            });

        let hdr = hdr::HdrPipeline::new(&device, &config);
        let hdr_loader = resources::HdrLoader::new(&device);

        let chunk_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Chunk Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/chunk.wgsl").into()),
            };
            pipeline::create_render_pipeline(
                &device,
                &render_pipeline_layout,
                hdr.format(),
                Some(texture::Texture::DEPTH_FORMAT),
                &[Some(model::ChunkVertex::desc())],
                wgpu::PrimitiveTopology::TriangleList,
                shader,
            )
        };

        let sky_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Sky Pipeline Layout"),
                bind_group_layouts: &[Some(&camera_layout), Some(&environment_layout)],
                immediate_size: 0,
            });
            let shader = wgpu::include_wgsl!("../shaders/sky.wgsl");
            pipeline::create_render_pipeline(
                &device,
                &layout,
                hdr.format(),
                Some(texture::Texture::DEPTH_FORMAT),
                &[],
                wgpu::PrimitiveTopology::TriangleList,
                shader,
            )
        };

        let outline_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Outline Pipeline Layout"),
                bind_group_layouts: &[Some(&camera_layout), Some(&outline_target_layout)],
                immediate_size: 0,
            });
            let shader =
                device.create_shader_module(wgpu::include_wgsl!("../shaders/outline.wgsl"));
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Outline Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &OUTLINE_ATTRIBUTES,
                    })],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: hdr.format(),
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::LineList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: texture::Texture::DEPTH_FORMAT,
                    depth_write_enabled: Some(false),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
        };

        let crosshair_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Crosshair Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });
            let shader = wgpu::include_wgsl!("../shaders/crosshair.wgsl");
            pipeline::create_render_pipeline(
                &device,
                &layout,
                config.format.add_srgb_suffix(),
                None,
                &[],
                wgpu::PrimitiveTopology::TriangleList,
                shader,
            )
        };

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let sky_bytes = resources::load_binary("pure-sky.hdr").await?;
        let sky_texture = hdr_loader.from_equirectangular_bytes(
            &device,
            &queue,
            &sky_bytes,
            512,
            Some("Sky Texture"),
        )?;

        let sun_direction = cgmath::Vector3::new(0.4, 0.8, 0.2).normalize();

        let sun_uniform = gpu::SunUniform {
            direction: sun_direction.into(),
            intensity: 1.0,
            color: [1.0, 0.95, 0.85],
            ambient: 0.25,
        };

        let sun_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sun Uniform Buffer"),
            contents: bytemuck::bytes_of(&sun_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let environment_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("environment_bind_group"),
            layout: &environment_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(sky_texture.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sky_texture.sampler()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: sun_buffer.as_entire_binding(),
                },
            ],
        });

        let layouts = RenderLayouts {
            chunk_material: chunk_material_layout,
            camera: camera_layout,
            environment: environment_layout,
        };

        let chunk_material = model::ChunkMaterial::new(&device, &queue, &layouts.chunk_material)?;
        let outline_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Outline Vertex Buffer"),
            contents: bytemuck::cast_slice(&OUTLINE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let outline_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Outline Index Buffer"),
            contents: bytemuck::cast_slice(&OUTLINE_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let outline_target_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Outline Target Buffer"),
            contents: bytemuck::cast_slice(&[[0.0_f32; 4]]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let outline_target_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("outline_target_bind_group"),
            layout: &outline_target_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: outline_target_buffer.as_entire_binding(),
            }],
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured,
            layouts,

            chunk_pipeline,
            sky_pipeline,
            outline_pipeline,
            crosshair_pipeline,

            depth_texture,
            hdr,
            environment_bind_group,
            chunk_material,
            chunk_meshes: HashMap::new(),
            outline_vertex_buffer,
            outline_index_buffer,
            outline_target_buffer,
            outline_target_bind_group,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            self.is_surface_configured = false;
            return;
        }

        const MAX_RENDER_DIMENSION: u32 = 2048;

        let width = width.min(MAX_RENDER_DIMENSION);
        let height = height.min(MAX_RENDER_DIMENSION);

        if self.is_surface_configured && self.config.width == width && self.config.height == height
        {
            return;
        }

        self.config.width = width;
        self.config.height = height;

        self.surface.configure(&self.device, &self.config);

        self.hdr.resize(&self.device, width, height);
        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");

        self.is_surface_configured = true;
    }

    pub fn update(&mut self, _dt: std::time::Duration) {}

    pub fn render(
        &mut self,
        camera: &camera::CameraState,
        targeted_block: Option<cgmath::Vector3<i32>>,
        show_crosshair: bool,
    ) -> anyhow::Result<()> {
        if !self.is_surface_configured {
            return Ok(());
        }

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                anyhow::bail!("Lost device");
            }
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.config.format.add_srgb_suffix()),
            ..Default::default()
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        if let Some(position) = targeted_block {
            self.queue.write_buffer(
                &self.outline_target_buffer,
                0,
                bytemuck::cast_slice(&[[
                    position.x as f32,
                    position.y as f32,
                    position.z as f32,
                    0.0,
                ]]),
            );
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: self.hdr.view(),
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.chunk_pipeline);
            render_pass.set_bind_group(0, &self.chunk_material.bind_group, &[]);
            render_pass.set_bind_group(1, &camera.bind_group, &[]);
            render_pass.set_bind_group(2, &self.environment_bind_group, &[]);

            for chunk in self.chunk_meshes.values() {
                render_pass.draw_chunk(chunk);
            }

            render_pass.set_pipeline(&self.sky_pipeline);
            render_pass.set_bind_group(0, &camera.bind_group, &[]);
            render_pass.set_bind_group(1, &self.environment_bind_group, &[]);
            render_pass.draw(0..3, 0..1);

            if targeted_block.is_some() {
                render_pass.set_pipeline(&self.outline_pipeline);
                render_pass.set_bind_group(0, &camera.bind_group, &[]);
                render_pass.set_bind_group(1, &self.outline_target_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.outline_vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    self.outline_index_buffer.slice(..),
                    wgpu::IndexFormat::Uint16,
                );
                render_pass.draw_indexed(0..OUTLINE_INDICES.len() as u32, 0, 0..1);
            }
        }

        self.hdr.process(&mut encoder, &view);

        if show_crosshair {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Crosshair Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.crosshair_pipeline);
            pass.draw(0..12, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(output);

        Ok(())
    }

    pub fn upload_chunk_mesh(&mut self, chunk_pos: cgmath::Vector3<i32>, mesh: CpuChunkMesh) {
        if mesh.indices.is_empty() {
            self.chunk_meshes.remove(&chunk_pos);
            return;
        }

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Index Buffer"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        self.chunk_meshes.insert(
            chunk_pos,
            GpuChunkMesh {
                vertex_buffer,
                index_buffer,
                num_elements: mesh.indices.len() as u32,
            },
        );
    }

    pub fn remove_chunk_mesh(&mut self, chunk_pos: cgmath::Vector3<i32>) {
        self.chunk_meshes.remove(&chunk_pos);
    }
}
