use std::ops::Range;
use cgmath::{Vector2, Vector3};
use wgpu::{BindGroup, VertexAttribute, VertexBufferLayout};
use wgpu::util::DeviceExt;
use crate::texture;

pub trait Vertex {
	fn desc<'a>() -> VertexBufferLayout<'a>;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MeshVertex {
	pub position: Vector3<f32>,
	pub tex_coord: Vector2<f32>,
}

unsafe impl bytemuck::Pod for MeshVertex {}
unsafe impl bytemuck::Zeroable for MeshVertex {}

impl Vertex for MeshVertex {
	fn desc<'a>() -> VertexBufferLayout<'a> {
		static ATTRIBS: [VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];
		VertexBufferLayout {
			array_stride: std::mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &ATTRIBS,
		}
	}
}

pub struct Material {
	pub name: String,
	pub diffuse_texture: texture::Texture,
	pub bind_group: BindGroup,
}

impl Material {
	pub fn new(
		name: &str,
		diffuse_texture: texture::Texture,
		device: &wgpu::Device,
		layout: &wgpu::BindGroupLayout,
	) -> Self {
		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
				},
			],
			label: Some(name),
		});

		Self {
			name: String::from(name),
			diffuse_texture,
			bind_group,
		}
	}
}

pub struct Mesh {
	pub name: String,
	pub vertices: Vec<MeshVertex>,
	pub indices: Vec<u32>,
	pub material: Material,
	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
	num_elements: u32,
}

impl Mesh {
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
		Vector3::new(-1.0, -1.0, 1.0),
		Vector3::new(1.0, -1.0, 1.0),
		Vector3::new(1.0, 1.0, 1.0),
		Vector3::new(-1.0, 1.0, 1.0),

		// Back Face
		Vector3::new(1.0, -1.0, -1.0),
		Vector3::new(-1.0, -1.0, -1.0),
		Vector3::new(-1.0, 1.0, -1.0),
		Vector3::new(1.0, 1.0, -1.0),

		// Top Face
		Vector3::new(-1.0, 1.0, 1.0),
		Vector3::new(1.0, 1.0, 1.0),
		Vector3::new(1.0, 1.0, -1.0),
		Vector3::new(-1.0, 1.0, -1.0),

		// Bottom Face
		Vector3::new(-1.0, -1.0, -1.0),
		Vector3::new(1.0, -1.0, -1.0),
		Vector3::new(1.0, -1.0, 1.0),
		Vector3::new(-1.0, -1.0, 1.0),

		// Left Face
		Vector3::new(-1.0, -1.0, -1.0),
		Vector3::new(-1.0, -1.0, 1.0),
		Vector3::new(-1.0, 1.0, 1.0),
		Vector3::new(-1.0, 1.0, -1.0),

		// Right Face
		Vector3::new(1.0, -1.0, 1.0),
		Vector3::new(1.0, -1.0, -1.0),
		Vector3::new(1.0, 1.0, -1.0),
		Vector3::new(1.0, 1.0, 1.0),
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

	pub fn new(name: &str, vertices: &[Vector3<f32>], tex_coords: &[Vector2<f32>], indices: &[u32], device: &wgpu::Device, material: Material) -> Self {
		let vertices = vertices.iter().zip(tex_coords.iter()).map(|(position, tex_coord)| {
			MeshVertex {
				position: *position,
				tex_coord: *tex_coord,
			}
		}).collect::<Vec<_>>();

		let vertex_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				contents: bytemuck::cast_slice(&vertices),
				usage: wgpu::BufferUsages::VERTEX,
				label: Some(&format!("{:?} Vertex Buffer", name)),
			}
		);
		let index_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				contents: bytemuck::cast_slice(indices),
				usage: wgpu::BufferUsages::INDEX,
				label: Some(&format!("{:?} Index Buffer", name)),
			}
		);

		Mesh {
			name: String::from(name),
			vertices,
			indices: indices.to_vec(),
			material,
			vertex_buffer,
			index_buffer,
			num_elements: indices.len() as u32,
		}
	}

	pub fn update_buffers(&mut self, device: &wgpu::Device) {
		self.vertex_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				contents: bytemuck::cast_slice(&self.vertices),
				usage: wgpu::BufferUsages::VERTEX,
				label: Some(&format!("{:?} Vertex Buffer", self.name)),
			}
		);
		self.index_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				contents: bytemuck::cast_slice(&self.indices),
				usage: wgpu::BufferUsages::INDEX,
				label: Some(&format!("{:?} Index Buffer", self.name)),
			}
		);
		self.num_elements = self.indices.len() as u32;
	}

	pub fn vertex_buffer(&self) -> &wgpu::Buffer {
		&self.vertex_buffer
	}

	pub fn index_buffer(&self) -> &wgpu::Buffer {
		&self.index_buffer
	}

	pub fn num_elements(&self) -> u32 {
		self.num_elements
	}

	// pub fn quad(name: &str, device: &wgpu::Device, material: Material) -> Self {
	// 	const QUAD_VERTS: &[Vector3<f32>; 4] = &[
	// 		Vector3::new(-1.0, -1.0, 0.0),
	// 		Vector3::new(1.0, -1.0, 0.0),
	// 		Vector3::new(1.0, 1.0, 0.0),
	// 		Vector3::new(-1.0, 1.0, 0.0),
	// 	];
	//
	// 	const QUAD_TEX_COORDS: &[Vector2<f32>; 4] = &[
	// 		Vector2::new(0.0, 1.0),
	// 		Vector2::new(1.0, 1.0),
	// 		Vector2::new(1.0, 0.0),
	// 		Vector2::new(0.0, 0.0),
	// 	];
	//
	// 	const QUAD_INDICES: &[u32; 6] = &[
	// 		0, 1, 2,
	// 		2, 3, 0,
	// 	];
	//
	// 	Self::new(name, QUAD_VERTS, QUAD_TEX_COORDS, QUAD_INDICES, device, material)
	// }

	pub fn cube(name: &str, device: &wgpu::Device, material: Material) -> Self {
		Self::new(name, &Mesh::CUBE_VERTS, &Mesh::CUBE_TEX_COORDS, &Mesh::CUBE_INDICES, device, material)
	}
}

pub trait DrawMesh<'a> {
	fn draw_mesh(&mut self, mesh: &'a Mesh, camera_bind_group: &'a BindGroup);
	fn draw_mesh_instanced(&mut self, mesh: &'a Mesh, instances: Range<u32>, camera_bind_group: &'a BindGroup);
}

impl <'a, 'b> DrawMesh<'b> for wgpu::RenderPass<'a> where 'b: 'a {
	fn draw_mesh(&mut self, mesh: &'b Mesh, camera_bind_group: &'b BindGroup) {
		self.draw_mesh_instanced(mesh, 0..1, camera_bind_group);
	}

	fn draw_mesh_instanced(&mut self, mesh: &'b Mesh, instances: Range<u32>, camera_bind_group: &'b BindGroup) {
		self.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
		self.set_index_buffer(mesh.index_buffer().slice(..), wgpu::IndexFormat::Uint32);
		self.set_bind_group(0, &mesh.material.bind_group, &[]);
		self.set_bind_group(1, camera_bind_group, &[]);
		self.draw_indexed(0..mesh.num_elements(), 0, instances);
	}
}