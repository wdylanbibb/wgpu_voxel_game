use std::ops::Deref;
use std::rc::Rc;

use bytemuck::{Pod, Zeroable};
use cgmath::{Vector2, Vector3, Zero};
use encase::ShaderType;
use ndarray::Array3;
use wgpu::{BindGroup, DynamicOffset, RenderPass};
use wgpu::util::DeviceExt;

use crate::{block, renderer};

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
/// An enum for the different faces of a cube to allow for easy toggling
pub enum Direction {
    FRONT, // 0, 0, 1
    BACK, // 0, 0, -1
    TOP, // 0, 1, 0
    BOTTOM, // 0, -1, 0
    LEFT, // -1, 0, 0
    RIGHT,  // 1, 0, 0
}

impl Direction {
    /// Returns the vertices that make up the face in a cube.
    pub fn cube_verts(&self) -> [Vector3<f32>; 4] {
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

    /// Returns the indices that make up the face in a cube.
    pub fn cube_indices(&self) -> [u32; 6] {
        match self {
            Direction::FRONT => [0, 1, 2, 2, 3, 0],
            Direction::BACK => [4, 5, 6, 6, 7, 4],
            Direction::TOP => [8, 9, 10, 10, 11, 8],
            Direction::BOTTOM => [12, 13, 14, 14, 15, 12],
            Direction::LEFT => [16, 17, 18, 18, 19, 16],
            Direction::RIGHT => [20, 21, 22, 22, 23, 20],
        }
    }

    /// Returns the normal vector of the face.
    pub fn to_vec3(&self) -> Vector3<i32> {
        match self {
            Direction::FRONT => Vector3::new(0, 0, 1),
            Direction::BACK => Vector3::new(0, 0, -1),
            Direction::TOP => Vector3::new(0, 1, 0),
            Direction::BOTTOM => Vector3::new(0, -1, 0),
            Direction::LEFT => Vector3::new(-1, 0, 0),
            Direction::RIGHT => Vector3::new(1, 0, 0),
        }
    }

    pub fn index(&self) -> u32 {
        match self {
            Direction::FRONT => 0,
            Direction::BACK => 1,
            Direction::TOP => 2,
            Direction::BOTTOM => 3,
            Direction::LEFT => 4,
            Direction::RIGHT => 5,
        }
    }

    pub fn get_opposite(&self) -> Self {
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

unsafe impl Pod for ChunkVertex {}

unsafe impl Zeroable for ChunkVertex {}

impl Vertex for ChunkVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        static ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ChunkVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(ShaderType, Debug, Copy, Clone)]
pub struct ChunkUniform {
    pub chunk_offset: Vector3<f32>,
}

impl ChunkUniform {
    pub fn new(chunk_offset: Vector3<f32>) -> Self {
        Self {
            chunk_offset,
        }
    }
}

unsafe impl Pod for ChunkUniform {}
unsafe impl Zeroable for ChunkUniform {}

pub const ATLAS_SIZE: usize = 256;
pub const TEXTURE_SIZE: usize = 16;

#[derive(Clone)]
pub struct ChunkMesh {
    vertex_buffer: Rc<wgpu::Buffer>,
    index_buffer: Rc<wgpu::Buffer>,
    num_elements: u32,
    pub uniform_offset: DynamicOffset,
    pub vertices: Vec<ChunkVertex>,
    pub indices: Vec<u32>,
}

impl ChunkMesh {
    pub fn new(uniform_offset: DynamicOffset, device: &wgpu::Device) -> Self {
        let vertices = vec![
            ChunkVertex { position: Vector3::zero(), tex_coord: Vector2::zero() }; 24 * CHUNK_SIZE
        ];

        let indices = vec![0u32; 36 * CHUNK_SIZE];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        ChunkMesh {
            vertex_buffer: Rc::new(vertex_buffer),
            index_buffer: Rc::new(index_buffer),
            num_elements: indices.len() as u32,
            uniform_offset,
            vertices,
            indices,
        }
    }

    pub fn flatten_3d(v: (i32, i32, i32)) -> u64 {
        // CHUNK_HEIGHT >> 1 is added to the y position to allow for y values of -127 to 128
        let (x, y, z) = v;
        (x + CHUNK_WIDTH as i32 * (y + (CHUNK_HEIGHT >> 1) as i32 + CHUNK_HEIGHT as i32 * z)) as u64
    }

    pub fn buffer_write(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));
    }

    pub fn get_buf_offset(chunk_position: Vector3<i32>, face: &Direction) -> (u64, u64) {
        let flattened = ChunkMesh::flatten_3d(chunk_position.into());

        let v_off = flattened * 24
            + face.index() as u64 * 4;

        let i_off = flattened * 36
            + face.index() as u64 * 6;

        (v_off, i_off)
    }

    pub fn add_face(
        &mut self,
        block_position: Vector3<i32>,
        face: &Direction,
        block: &block::Block,
    ) {
        let flattened = ChunkMesh::flatten_3d(block_position.into());

        let vertices = {
            let position = block_position.cast::<f32>().unwrap();

            face.cube_verts()
                .iter()
                .zip(
                    &block.deref().texture_coordinates().to_vec()
                        [(face.index() * 4) as usize..(face.index() * 4 + 4) as usize],
                )
                .map(|(p, t)| {
                    ChunkVertex {
                        position: *p + position,
                        tex_coord: *t,
                    }
                })
                .collect::<Vec<_>>()
        };

        let indices = face.cube_indices().map(|i| i + 24 * flattened as u32);

        let (v_off, i_off) = ChunkMesh::get_buf_offset(block_position, &face);

        self.vertices.splice(v_off as usize..(v_off as usize + vertices.len()), vertices);
        self.indices.splice(i_off as usize..(i_off as usize + indices.len()), indices);
    }

    pub fn remove_face(&mut self, position: Vector3<i32>, face: &Direction) {
        let (v_off, i_off) = ChunkMesh::get_buf_offset(position, &face);

        self.vertices.splice(
            v_off as usize..(v_off as usize + 4),
            vec![ChunkVertex { position: Vector3::zero(), tex_coord: Vector2::zero() }; 4]
        );

        self.indices.splice(i_off as usize..(i_off as usize + 6), vec![0u32; 6]);
    }
}

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 256;
pub const CHUNK_DEPTH: usize = 16;
pub const CHUNK_DIMS: (usize, usize, usize) = (CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_DEPTH);
pub const CHUNK_SIZE: usize = CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH;

#[derive(Clone)]
pub struct Chunk {
    pub blocks: Array3<block::Block>,
    pub world_offset: Vector2<i32>,
}

impl Chunk {
    pub fn new(world_offset: Vector2<i32>) -> Self {
        let blocks =
            Array3::<block::Block>::from_shape_fn(CHUNK_DIMS, |_| block::Block::Air(block::Air));

        Self {
            blocks,
            world_offset,
        }
    }

    pub fn set_block(&mut self, position: Vector3<i32>, block: block::Block) {

        self.blocks[[
            position.x as usize,
            (position.y + (CHUNK_HEIGHT >> 1) as i32) as usize,
            position.z as usize,
        ]] = block;
    }

    pub fn get_block(&self, mut position: Vector3<i32>) -> Option<&block::Block> {
        // let mut position: Option<Vector3<usize>> = position.cast();
        position.y = position.y + (CHUNK_HEIGHT >> 1) as i32;
        self.blocks.get((
            position.x as usize,
            position.y as usize,
            position.z as usize,
        ))
    }
}

impl renderer::Draw for ChunkMesh {
    fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>, camera_bind_group: &'a BindGroup, uniforms: &'a BindGroup) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(1, uniforms, &[self.uniform_offset]);
        render_pass.draw_indexed(0..self.num_elements, 0, 0..1);
    }
}
