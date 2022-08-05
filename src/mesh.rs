use std::ops::Range;
use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix4, Quaternion, Vector2, Vector3};
use wgpu::util::DeviceExt;
use crate::{One, texture};

pub trait Vertex {
	fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
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
	fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
		static ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &ATTRIBS,
		}
	}
}

pub struct Material {
	pub name: String,
	pub diffuse_texture: texture::Texture,
	pub bind_group: wgpu::BindGroup,
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

pub struct Instance {
	pub position: Vector3<f32>,
	pub rotation: Quaternion<f32>,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct InstanceRaw {
	pub model: Matrix4<f32>,
}

unsafe impl Pod for InstanceRaw {}
unsafe impl Zeroable for InstanceRaw {}

impl Instance {
	pub fn new(position: Vector3<f32>) -> Self {
		Self {
			position,
			rotation: Quaternion::one(),
		}
	}

	pub fn to_raw(&self) -> InstanceRaw {
		let model = Matrix4::from_translation(self.position) * Matrix4::from(self.rotation);
		InstanceRaw {
			model,
		}
	}
}

impl InstanceRaw {
	pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
		static ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4];
		use std::mem;
		wgpu::VertexBufferLayout {
			array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Instance,
			attributes: &ATTRIBS,
		}
	}
}

pub struct Mesh {
	pub name: String,
	pub vertex_buffer: wgpu::Buffer,
	pub index_buffer: wgpu::Buffer,
	pub num_elements: u32,
	pub material: Material,

	pub instances: Vec<Instance>,
	pub instance_buffer: wgpu::Buffer,
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

	pub fn new(name: &str, vertices: &[Vector3<f32>], tex_coords: &[Vector2<f32>], indices: &[u32], material: Material, instances: Vec<Instance>, device: &wgpu::Device) -> Self {
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

		let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
		let instance_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Instance Buffer"),
				contents: bytemuck::cast_slice(&instance_data),
				usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			}
		);

		Mesh {
			name: String::from(name),
			vertex_buffer,
			index_buffer,
			num_elements: indices.len() as u32,
			material,
			instances,
			instance_buffer,
		}
	}

	pub fn cube(name: &str, material: Material, instances: Vec<Instance>, device: &wgpu::Device) -> Self {
		Self::new(name, &Mesh::CUBE_VERTS, &Mesh::CUBE_TEX_COORDS, &Mesh::CUBE_INDICES, material, instances, device)
	}

	pub fn add_instance(&mut self, instance: Instance, device: &wgpu::Device) {
		self.instances.push(instance);

		self.update_instance_buffer(device);
	}

	fn update_instance_buffer(&mut self, device: &wgpu::Device) {
		let instance_data = self.instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
		self.instance_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Instance Buffer"),
				contents: bytemuck::cast_slice(&instance_data),
				usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			}
		);
	}
}

// pub const SIZE: usize = 16;

// In the future, a hashmap with the different block types and their respective meshes would be better.
pub struct Chunk {
	pub blocks: Mesh,
}

impl Chunk {
	pub fn new(material: Material, device: &wgpu::Device) -> Self {
		let blocks = Mesh::cube("Chunk", material, Vec::new(), device);

		Self {
			blocks,
		}
	}

	pub fn add_block(&mut self, position: Vector3<f32>, device: &wgpu::Device) {
		self.blocks.add_instance(Instance::new(position), device);
	}
}

pub trait DrawMesh<'a> {
	fn draw_mesh(&mut self, mesh: &'a Mesh, camera_bind_group: &'a wgpu::BindGroup);
	fn draw_mesh_instanced(&mut self, mesh: &'a Mesh, instances: Range<u32>, camera_bind_group: &'a wgpu::BindGroup);
}

impl <'a, 'b> DrawMesh<'b> for wgpu::RenderPass<'a> where 'b: 'a {
	fn draw_mesh(&mut self, mesh: &'b Mesh, camera_bind_group: &'b wgpu::BindGroup) {
		self.draw_mesh_instanced(mesh, 0..1, camera_bind_group);
	}

	fn draw_mesh_instanced(&mut self, mesh: &'b Mesh, instances: Range<u32>, camera_bind_group: &'b wgpu::BindGroup) {
		self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
		self.set_vertex_buffer(1, mesh.instance_buffer.slice(..));
		self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
		self.set_bind_group(0, &mesh.material.bind_group, &[]);
		self.set_bind_group(1, camera_bind_group, &[]);
		self.draw_indexed(0..mesh.num_elements, 0, instances);
	}
}

pub trait DrawChunk<'a> {
	fn draw_chunk(&mut self, chunk: &'a Chunk, camera_bind_group: &'a wgpu::BindGroup);
}

impl <'a, 'b> DrawChunk<'b> for wgpu::RenderPass<'a> where 'b: 'a {
	fn draw_chunk(&mut self, chunk: &'b Chunk, camera_bind_group: &'b wgpu::BindGroup) {
		self.draw_mesh_instanced(&chunk.blocks, 0..1, camera_bind_group);
	}
}

