use std::ops::Range;
use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix4, Quaternion, Vector2, Vector3};
use wgpu::util::DeviceExt;
use crate::{One, texture};
use crate::material::Material;

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
				usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
				label: Some(&format!("{:?} Vertex Buffer", name)),
			}
		);
		let index_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				contents: bytemuck::cast_slice(indices),
				usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
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



