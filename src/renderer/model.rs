use std::mem;

use crate::renderer::texture;

pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

pub trait DrawChunk<'a> {
    fn draw_chunk(&mut self, chunk: &'a GpuChunkMesh);
}

pub struct ChunkMaterial {
    pub atlas: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}

impl ChunkMaterial {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
    ) -> anyhow::Result<Self> {
        let atlas = texture::Texture::from_bytes(
            device,
            queue,
            include_bytes!("../../res/sprite_atlas.png"),
            "Block Sprite Atlas",
            false,
        )?;

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Chunk Material Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&atlas.sampler),
                },
            ],
        });

        Ok(Self { atlas, bind_group })
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ChunkVertex {
    pub position: [f32; 3],
    pub tex_coord: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex for ChunkVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        static ATTRIBS: [wgpu::VertexAttribute; 3] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ChunkVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}

pub struct GpuChunkMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

impl<'a, 'b> DrawChunk<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_chunk(&mut self, chunk: &'b GpuChunkMesh) {
        self.set_vertex_buffer(0, chunk.vertex_buffer.slice(..));
        self.set_index_buffer(chunk.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        self.draw_indexed(0..chunk.num_elements, 0, 0..1);
    }
}
