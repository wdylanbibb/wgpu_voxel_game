use crate::block;
use crate::material::Material;
use cgmath::{ElementWise, Vector2, Vector3};
use ndarray::Array3;
use std::ops::Deref;
use wgpu::util::DeviceExt;
use crate::renderer::Draw;

/*
       (-1, 1, -1) /-------------------| (1, 1, -1)
                 / |                  /|
               /   |                /  |
 (-1, 1, 1)  /     |    (1, 1, 1) /    |
            |------|------------|      |
            |      |            |      |
            |      |            |      |
            |      |------------|------| (1, -1, -1)
            |     /(-1, -1, -1) |     /
            |   /               |   /
            | /                 | /
(-1, -1, 1) |-------------------| (1, -1, 1)
   */

#[derive(Debug)]
pub enum Direction {
    FRONT, // 0, 0, 1
    BACK, // 0, 0, -1
    TOP, // 0, 1, 0
    BOTTOM, // 0, -1, 0
    LEFT, // -1, 0, 0
    RIGHT, // 1, 0, 0
}

impl Direction {
    fn cube_verts(&self) -> [Vector3<f32>; 4] {
        match self {
            Direction::FRONT => [
                Vector3::new(-0.5, -0.5, 0.5),
                Vector3::new(0.5, -0.5, 0.5),
                Vector3::new(0.5, 0.5, 0.5),
                Vector3::new(-0.5, 0.5, 0.5),
            ],
            Direction::BACK => [
                Vector3::new(0.5, -0.5, -0.5),
                Vector3::new(-0.5, -0.5, -0.5),
                Vector3::new(-0.5, 0.5, -0.5),
                Vector3::new(0.5, 0.5, -0.5),
            ],
            Direction::TOP => [
                Vector3::new(-0.5, 0.5, 0.5),
                Vector3::new(0.5, 0.5, 0.5),
                Vector3::new(0.5, 0.5, -0.5),
                Vector3::new(-0.5, 0.5, -0.5),
            ],
            Direction::BOTTOM => [
                Vector3::new(-0.5, -0.5, -0.5),
                Vector3::new(0.5, -0.5, -0.5),
                Vector3::new(0.5, -0.5, 0.5),
                Vector3::new(-0.5, -0.5, 0.5),
            ],
            Direction::LEFT => [
                Vector3::new(-0.5, -0.5, -0.5),
                Vector3::new(-0.5, -0.5, 0.5),
                Vector3::new(-0.5, 0.5, 0.5),
                Vector3::new(-0.5, 0.5, -0.5),
            ],
            Direction::RIGHT => [
                Vector3::new(0.5, -0.5, 0.5),
                Vector3::new(0.5, -0.5, -0.5),
                Vector3::new(0.5, 0.5, -0.5),
                Vector3::new(0.5, 0.5, 0.5),
            ],
        }
    }

    fn cube_indices(&self) -> [u32; 6] {
        match self {
            Direction::FRONT => [0, 1, 2, 2, 3, 0],
            Direction::BACK => [4, 5, 6, 6, 7, 4],
            Direction::TOP => [8, 9, 10, 10, 11, 8],
            Direction::BOTTOM => [12, 13, 14, 14, 15, 12],
            Direction::LEFT => [16, 17, 18, 18, 19, 16],
            Direction::RIGHT => [20, 21, 22, 22, 23, 20],
        }
    }

    fn to_vec3(&self) -> Vector3<i32> {
        match self {
            Direction::FRONT => Vector3::new(0, 0, 1),
            Direction::BACK => Vector3::new(0, 0, -1),
            Direction::TOP => Vector3::new(0, 1, 0),
            Direction::BOTTOM => Vector3::new(0, -1, 0),
            Direction::LEFT => Vector3::new(-1, 0, 0),
            Direction::RIGHT => Vector3::new(1, 0, 0),
        }
    }

    fn index(&self) -> u32 {
        match self {
            Direction::FRONT => 0,
            Direction::BACK => 1,
            Direction::TOP => 2,
            Direction::BOTTOM => 3,
            Direction::LEFT => 4,
            Direction::RIGHT => 5,
        }
    }

    fn get_opposite(&self) -> Self {
        match self {
            Direction::FRONT => Direction::BACK,
            Direction::BACK => Direction::FRONT,
            Direction::TOP => Direction::BOTTOM,
            Direction::BOTTOM => Direction::TOP,
            Direction::LEFT => Direction::RIGHT,
            Direction::RIGHT => Direction::LEFT,
        }
    }
}

pub trait Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

// Perhaps a more apt name would be BlockVertex but it's fine
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ChunkVertex {
    pub position: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
}

unsafe impl bytemuck::Pod for ChunkVertex {}
unsafe impl bytemuck::Zeroable for ChunkVertex {}

impl Vertex for ChunkVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        static ATTRIBS: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ChunkVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}

pub const ATLAS_SIZE: usize = 256;
pub const TEXTURE_SIZE: usize = 16;

pub struct ChunkMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_elements: u32,
    material: Material,
}

impl ChunkMesh {
    fn new(material: Material, device: &wgpu::Device) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(
                &[0 as u32; std::mem::size_of::<ChunkVertex>() * 24 * CHUNK_SIZE],
            ),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[0 as u32; 36 * CHUNK_SIZE]),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        ChunkMesh {
            vertex_buffer,
            index_buffer,
            num_elements: 36 * CHUNK_SIZE as u32,
            material,
        }
    }

    fn flatten_3d(v: (i32, i32, i32)) -> u64 {
        // CHUNK_HEIGHT >> 1 is added to the y position to allow for y values of -127 to 128
        let (x, y, z) = v;
        (x
            + CHUNK_WIDTH as i32
            * (y + (CHUNK_HEIGHT >> 1) as i32 + CHUNK_DEPTH as i32 * z))
            as u64
    }

    fn add_block(&self, chunk: &Chunk, position: Vector3<i32>, block: &block::Block, queue: &wgpu::Queue) {
        let flattened = ChunkMesh::flatten_3d(position.into());

        if let block::Block::Air(_) = block {
            queue.write_buffer(
                &self.vertex_buffer,
                flattened * std::mem::size_of::<ChunkVertex>() as u64 * 24,
                bytemuck::cast_slice(&[0 as u8; std::mem::size_of::<ChunkVertex>() * 24]),
            );

            queue.write_buffer(
                &self.index_buffer,
                flattened * std::mem::size_of::<u32>() as u64 * 36,
                bytemuck::cast_slice(&[0 as u32; 36]),
            );

            return;
        }

        for face in [
            Direction::FRONT,
            Direction::BACK,
            Direction::TOP,
            Direction::BOTTOM,
            Direction::LEFT,
            Direction::RIGHT,
        ] {
            let v = face.to_vec3().add_element_wise(position);

            let neighbor = chunk.get_block(v);
            match neighbor {
                Some(neighbor) => {
                    if let block::Block::Air(_) = neighbor {
                        // The face is touching air
                        self.add_face(position, &face, block, queue);
                    } else {
                        // The face is touching a neighbor
                        self.remove_face(position, &face, queue);
                        chunk.mesh.remove_face(v, &face.get_opposite(), queue);
                    }
                },
                None => {
                    // The face is on the edge of a chunk
                    self.add_face(position, &face, block, queue)
                }
            }
        }
    }

    fn get_buf_offset(position: Vector3<i32>, face: &Direction) -> (u64, u64) {
        let flattened = ChunkMesh::flatten_3d(position.into());

        let v_off =
            flattened * std::mem::size_of::<ChunkVertex>() as u64 * 24 + face.index() as u64 * std::mem::size_of::<ChunkVertex>() as u64 * 4;

        let i_off =
            flattened * std::mem::size_of::<u32>() as u64 * 36 + face.index() as u64 * std::mem::size_of::<u32>() as u64 * 6;

        (v_off, i_off)
    }

    fn add_face(&self, position: Vector3<i32>, face: &Direction, block: &block::Block, queue: &wgpu::Queue) {
        let flattened = ChunkMesh::flatten_3d(position.into());

        let vertices = {
            let position = Vector3::new(position.x as f32, position.y as f32, position.z as f32);

            face.cube_verts()
                .iter()
                .zip(&block.deref().texture_coordinates().to_vec()[(face.index() * 4) as usize..(face.index() * 4 + 4) as usize])
                .map(|(p, t)| {
                    ChunkVertex {
                        position: *p + position,
                        tex_coord: *t,
                    }
                })
                .collect::<Vec<_>>()
        };

        let (v_off, i_off) = ChunkMesh::get_buf_offset(position, &face);

        queue.write_buffer(
            &self.vertex_buffer,
            v_off,
            bytemuck::cast_slice(&vertices),
        );

        queue.write_buffer(
            &self.index_buffer,
            i_off,
            bytemuck::cast_slice(&face.cube_indices().map(|i| i + 24 * flattened as u32)),
        );
    }

    fn remove_face(&self, position: Vector3<i32>, face: &Direction, queue: &wgpu::Queue) {
        let (v_off, i_off) = ChunkMesh::get_buf_offset(position, &face);

        queue.write_buffer(
            &self.vertex_buffer,
            v_off,
            bytemuck::cast_slice(&[0 as u8; std::mem::size_of::<ChunkVertex>() * 4]),
        );

        queue.write_buffer(
            &self.index_buffer,
            i_off,
            bytemuck::cast_slice(&[0 as u32; 6]),
        );
    }
}

const CHUNK_DIMS: (usize, usize, usize) = (16, 256, 16);
const CHUNK_WIDTH: usize = CHUNK_DIMS.0;
const CHUNK_HEIGHT: usize = CHUNK_DIMS.1;
const CHUNK_DEPTH: usize = CHUNK_DIMS.2;
const CHUNK_SIZE: usize = CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH;

pub struct Chunk {
    pub blocks: Array3<block::Block>,
    pub mesh: ChunkMesh,
}

impl Chunk {
    pub fn new(material: Material, device: &wgpu::Device) -> Self {
        let blocks = Array3::<block::Block>::from_shape_fn(CHUNK_DIMS, |_| block::Block::Air(block::Air));

        let mesh = ChunkMesh::new(material, &device);

        Self { blocks, mesh }
    }

    pub fn set_block(&mut self, position: Vector3<i32>, block: block::Block, queue: &wgpu::Queue) {
        self.mesh.add_block(
            self,
            Vector3::new(position.x, position.y, position.z),
            &block,
            queue,
        );

        self.blocks[[
            position.x as usize,
            (position.y + (CHUNK_HEIGHT >> 1) as i32) as usize,
            position.z as usize,
        ]] = block;
    }

    pub fn get_block(&self, mut position: Vector3<i32>) -> Option<&block::Block> {
        // let mut position: Option<Vector3<usize>> = position.cast();
        position.y = position.y + (CHUNK_HEIGHT >> 1) as i32;
        self.blocks.get((position.x as usize, position.y as usize, position.z as usize))
    }
}

impl Draw for ChunkMesh {
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, uniforms: &'a wgpu::BindGroup) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_bind_group(0, &self.material.bind_group, &[]);
        render_pass.set_bind_group(1, uniforms, &[]);
        render_pass.draw_indexed(0..self.num_elements, 0, 0..1);
    }
}

// pub trait DrawChunk<'a> {
//     fn draw_chunk(&mut self, chunk: &'a Chunk, camera_bind_group: &'a wgpu::BindGroup);
// }
// 
// impl<'a, 'b> DrawChunk<'b> for wgpu::RenderPass<'a>
// where
//     'b: 'a,
// {
//     fn draw_chunk(&mut self, chunk: &'b Chunk, camera_bind_group: &'b wgpu::BindGroup) {
//         self.push_debug_group("Prepare chunk data for draw");
//         self.set_vertex_buffer(0, chunk.mesh.vertex_buffer.slice(..));
//         self.set_index_buffer(chunk.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
//         self.set_bind_group(0, &chunk.mesh.material.bind_group, &[]);
//         self.set_bind_group(1, camera_bind_group, &[]);
//         self.pop_debug_group();
//         self.insert_debug_marker("Draw!");
//         self.draw_indexed(0..chunk.mesh.num_elements, 0, 0..1);
//     }
// }
