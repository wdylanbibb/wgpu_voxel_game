use cgmath::{Vector2, Vector3};
use ndarray::Array3;
use wgpu::util::{DeviceExt};
use crate::material::Material;

/*
		    (-1, 1, -1) /-------------------| (1, 1, -1)
				      / |                  /|
				    /   |                /  |
	  (-1, 1, 1)  /     |    (1, 1, 1) /    |
				 |------|------------|      |
				 |      |            |      |
				 |      |            |      |
				 |      |------------|------| (1, -1,- 1)
				 |     /(-1, -1, -1) |     /
				 |   /               |   /
				 | /                 | /
	 (-1, -1, 1) |-------------------| (1, -1, 1)
		*/

pub const CUBE_VERTS: [Vector3<f32>; 24] = [
	// Front Face
	Vector3::new(-0.5, -0.5, 0.5),
	Vector3::new(0.5, -0.5, 0.5),
	Vector3::new(0.5, 0.5, 0.5),
	Vector3::new(-0.5, 0.5, 0.5),

	// Back Face
	Vector3::new(0.5, -0.5, -0.5),
	Vector3::new(-0.5, -0.5, -0.5),
	Vector3::new(-0.5, 0.5, -0.5),
	Vector3::new(0.5, 0.5, -0.5),

	// Top Face
	Vector3::new(-0.5, 0.5, 0.5),
	Vector3::new(0.5, 0.5, 0.5),
	Vector3::new(0.5, 0.5, -0.5),
	Vector3::new(-0.5, 0.5, -0.5),

	// Bottom Face
	Vector3::new(-0.5, -0.5, -0.5),
	Vector3::new(0.5, -0.5, -0.5),
	Vector3::new(0.5, -0.5, 0.5),
	Vector3::new(-0.5, -0.5, 0.5),

	// Left Face
	Vector3::new(-0.5, -0.5, -0.5),
	Vector3::new(-0.5, -0.5, 0.5),
	Vector3::new(-0.5, 0.5, 0.5),
	Vector3::new(-0.5, 0.5, -0.5),

	// Right Face
	Vector3::new(0.5, -0.5, 0.5),
	Vector3::new(0.5, -0.5, -0.5),
	Vector3::new(0.5, 0.5, -0.5),
	Vector3::new(0.5, 0.5, 0.5),
];

pub const CUBE_TEX_COORDS: [Vector2<f32>; 24] = [
	// Front Face
	Vector2::new(0.0, 1.0),
	Vector2::new(1.0, 1.0),
	Vector2::new(1.0, 0.0),
	Vector2::new(0.0, 0.0),

	// Back Face
	Vector2::new(1.0, 1.0),
	Vector2::new(0.0, 1.0),
	Vector2::new(0.0, 0.0),
	Vector2::new(1.0, 0.0),

	// Top Face
	Vector2::new(0.0, 1.0),
	Vector2::new(1.0, 1.0),
	Vector2::new(1.0, 0.0),
	Vector2::new(0.0, 0.0),

	// Bottom Face
	Vector2::new(1.0, 1.0),
	Vector2::new(0.0, 1.0),
	Vector2::new(0.0, 0.0),
	Vector2::new(1.0, 0.0),

	// Left Face
	Vector2::new(0.0, 1.0),
	Vector2::new(1.0, 1.0),
	Vector2::new(1.0, 0.0),
	Vector2::new(0.0, 0.0),

	// Right Face
	Vector2::new(1.0, 1.0),
	Vector2::new(0.0, 1.0),
	Vector2::new(0.0, 0.0),
	Vector2::new(1.0, 0.0),
];

pub const CUBE_INDICES: [u32; 36] = [
	// Front Face
	0, 1, 2,
	2, 3, 0,

	// Back Face
	4, 5, 6,
	6, 7, 4,

	// Top Face
	8, 9, 10,
	10, 11, 8,

	// Bottom Face
	12, 13, 14,
	14, 15, 12,

	// Left Face
	16, 17, 18,
	18, 19, 16,

	// Right Face
	20, 21, 22,
	22, 23, 20,
];

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
		static ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<ChunkVertex>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &ATTRIBS,
		}
	}
}

pub struct ChunkMesh {
	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
	num_elements: u32,
	material: Material,
}

impl ChunkMesh {
	pub fn new(material: Material, device: &wgpu::Device) -> Self {
		let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: None,
			contents: bytemuck::cast_slice(&[0 as u32; std::mem::size_of::<ChunkVertex>() * 24 * CHUNK_SIZE]),
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

	pub fn add_block(&self, position: Vector3<i32>, queue: &wgpu::Queue) {
		let vertices = {
			let position = Vector3::new(position.x as f32, position.y as f32, position.z as f32);

			CUBE_VERTS.iter().zip(CUBE_TEX_COORDS.iter()).map(|(p, t)| {
				ChunkVertex {
					position: *p + position,
					tex_coord: *t,
				}
			}).collect::<Vec<_>>()
		};

		// CHUNK_HEIGHT >> 1 is added to the y position to allow for y values of -127 to 128
		let flattened = (position.x + CHUNK_WIDTH as i32 * (position.y + (CHUNK_HEIGHT >> 1) as i32 + CHUNK_DEPTH as i32 * position.z)) as u64;

		queue.write_buffer(
			&self.vertex_buffer,
			flattened * std::mem::size_of::<ChunkVertex>() as u64 * 24,
			bytemuck::cast_slice(&vertices),
		);

		queue.write_buffer(
			&self.index_buffer,
			flattened * 36 * 4, // each index is 4 bytes, and there are 36 indicies per cube
			bytemuck::cast_slice(&CUBE_INDICES.map(|i| i + 24 * flattened as u32)),
		);
	}
}

const CHUNK_DIMS: (usize, usize, usize) = (16, 256, 16);
const CHUNK_WIDTH: usize = CHUNK_DIMS.0;
const CHUNK_HEIGHT: usize = CHUNK_DIMS.1;
const CHUNK_DEPTH: usize = CHUNK_DIMS.2;
const CHUNK_SIZE: usize = CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH;

pub struct Chunk {
	pub blocks: Array3<bool>,
	pub mesh: ChunkMesh,
}

impl Chunk {
	pub fn new(material: Material, device: &wgpu::Device) -> Self {
		let blocks = Array3::from_elem(CHUNK_DIMS, false);

		let mesh = ChunkMesh::new(material, &device);

		Self {
			blocks,
			mesh,
		}
	}

	pub fn add_block(&mut self, position: Vector3<i32>, queue: &wgpu::Queue) {
		self.blocks[[position.x as usize, (position.y + (CHUNK_HEIGHT >> 1) as i32) as usize, position.z as usize]] = true;

		self.mesh.add_block(Vector3::new(position.x, position.y, position.z), queue);
	}
}

pub trait DrawChunk<'a> {
	fn draw_chunk(&mut self, chunk: &'a Chunk, camera_bind_group: &'a wgpu::BindGroup);
}

impl <'a, 'b> DrawChunk<'b> for wgpu::RenderPass<'a> where 'b: 'a {
	fn draw_chunk(&mut self, chunk: &'b Chunk, camera_bind_group: &'b wgpu::BindGroup) {
		self.set_vertex_buffer(0, chunk.mesh.vertex_buffer.slice(..));
		self.set_index_buffer(chunk.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
		self.set_bind_group(0, &chunk.mesh.material.bind_group, &[]);
		self.set_bind_group(1, camera_bind_group, &[]);
		self.draw_indexed(0..chunk.mesh.num_elements, 0, 0..1);
	}
}