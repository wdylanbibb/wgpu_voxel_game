use cgmath::{Vector2, ElementWise, Vector3};
use crate::{chunk::{Chunk, ChunkMesh, Direction}, block::Block};

pub struct World {
    chunks: Vec<Chunk>,
    chunk_meshes: Vec<ChunkMesh>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            chunk_meshes: Vec::new(),
        }
    }

    pub fn new_chunk(&mut self, chunk_offset: Vector2<i32>, uniform_offset: u32, device: &wgpu::Device) -> usize {
        let chunk = Chunk::new(chunk_offset);
        let chunk_mesh = ChunkMesh::new(uniform_offset, device);

        self.chunks.push(chunk);
        self.chunk_meshes.push(chunk_mesh);

        if self.chunks.len() != self.chunk_meshes.len() {
            eprintln!("chunk vec and chunk mesh vec have different sizes!");
        }

        self.chunks.len() - 1
    }

    pub fn get_chunk(&self, chunk_index: usize) -> (&Chunk, &ChunkMesh) {
        (&self.chunks[chunk_index], &self.chunk_meshes[chunk_index])
    }

    pub fn get_chunk_mut(&mut self, chunk_index: usize) -> (&mut Chunk, &mut ChunkMesh) {
        (&mut self.chunks[chunk_index], &mut self.chunk_meshes[chunk_index])
    }

    pub fn set_block(&mut self, chunk_index: usize, position: Vector3<i32>, block: Block) {
        let (chunk, mesh) = self.get_chunk_mut(chunk_index);

        chunk.set_block(position, block);

        let _flattened = ChunkMesh::flatten_3d(position.into());

        let faces = [
            Direction::FRONT,
            Direction::BACK,
            Direction::TOP,
            Direction::BOTTOM,
            Direction::LEFT,
            Direction::RIGHT,
        ];

        if let Block::Air(_) = block {
            for face in faces {
                mesh.remove_face(position, &face);
                let v = face.to_vec3().add_element_wise(position);

                let neighbor = chunk.get_block(v);
                if let Some(neighbor) = neighbor {
                    match neighbor {
                        Block::Air(_) => (),
                        _ => mesh.add_face(position, &face.get_opposite(), neighbor),
                    }
                }
            }
        } else {
            for face in faces {
                let v = face.to_vec3().add_element_wise(position);

                let neighbor = chunk.get_block(v);
                if let Some(neighbor) = neighbor {
                    match neighbor {
                        Block::Air(_) => mesh.add_face(position, &face, &block),
                        _ => {
                            mesh.remove_face(position, &face);
                            mesh.remove_face(v, &face.get_opposite());
                        }
                    }
                } else {
                    mesh.add_face(position, &face, &block);
                }
            }
        }
    }

    pub fn update_buffers(&self, queue: &wgpu::Queue) {
        for chunk_mesh in self.chunk_meshes.iter() {
            chunk_mesh.buffer_write(queue);
        }
    }

    pub fn chunks_iter(&self) -> std::slice::Iter<Chunk> {
        self.chunks.iter()
    }

    pub fn chunks_iter_mut(&mut self) -> std::slice::IterMut<Chunk> {
        self.chunks.iter_mut()
    }

    pub fn chunk_mesh_iter(&self) -> std::slice::Iter<ChunkMesh> {
        self.chunk_meshes.iter()
    }

    pub fn chunk_mesh_iter_mut(&mut self) -> std::slice::IterMut<ChunkMesh> {
        self.chunk_meshes.iter_mut()
    }
}
