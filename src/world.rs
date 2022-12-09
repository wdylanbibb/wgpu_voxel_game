use cgmath::{Vector2, ElementWise, Vector3};
use hashbrown::HashMap;
use crate::{chunk::{Chunk, ChunkMesh, Direction, self}, block::Block};

#[derive(Clone)]
pub struct World {
    chunk_map: HashMap<Vector2<i32>, usize>,
    chunks: Vec<Chunk>,
    chunk_meshes: Vec<ChunkMesh>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunk_map: HashMap::new(),
            chunks: Vec::new(),
            chunk_meshes: Vec::new(),
        }
    }

    pub fn new_chunk(&mut self, chunk_location: Vector2<i32>, uniform_offset: u32, device: &wgpu::Device) -> usize {
        let chunk = Chunk::new(chunk_location);
        let chunk_mesh = ChunkMesh::new(uniform_offset, device);

        self.chunks.push(chunk);
        self.chunk_meshes.push(chunk_mesh);

        if self.chunks.len() != self.chunk_meshes.len() {
            eprintln!("chunk vec and chunk mesh vec have different sizes!");
        }

        let index = self.chunks.len() - 1;

        self.chunk_map.insert(chunk_location, index);

        index
    }

    pub fn get_chunk_index_by_offset(&self, offset: Vector2<i32>) -> Option<usize> {
        self.chunk_map.get(&offset).copied()
    }

    pub fn get_chunk_by_offset(&self, offset: Vector2<i32>) -> Option<(&Chunk, &ChunkMesh)> {
        match self.get_chunk_index_by_offset(offset) {
            Some(expr) => self.get_chunk(expr),
            None => None,
        }
    }

    pub fn get_chunk(&self, chunk_index: usize) -> Option<(&Chunk, &ChunkMesh)> {
        match (self.chunks.get(chunk_index), self.chunk_meshes.get(chunk_index)) {
            (None, None) | (None, Some(_)) | (Some(_), None) => None,
            (Some(chunk), Some(mesh)) => Some((chunk, mesh)),
        }
    }

    pub fn get_chunk_mut(&mut self, chunk_index: usize) -> Option<(&mut Chunk, &mut ChunkMesh)> {
        match (self.chunks.get_mut(chunk_index), self.chunk_meshes.get_mut(chunk_index)) {
            (None, None) | (None, Some(_)) | (Some(_), None) => None,
            (Some(chunk), Some(mesh)) => Some((chunk, mesh))
        }
    }

    pub fn set_block(&mut self, chunk_index: usize, position: Vector3<i32>, block: Block) {
        let chunk = match self.chunks.get_mut(chunk_index) {
            Some(chunk) => chunk,
            None => return,
        };

        chunk.set_block(position, block);

        let chunks = self.chunks.clone();

        let chunk = match chunks.get(chunk_index) {
            Some(chunk) => chunk,
            None => return,
        };

        let _flattened = ChunkMesh::flatten_3d(position.into());

        let faces = [
            Direction::FRONT,
            Direction::BACK,
            Direction::TOP,
            Direction::BOTTOM,
            Direction::LEFT,
            Direction::RIGHT,
        ];

        let is_air = if let Block::Air(_) = block { true } else { false };

        for face in faces {
            let face_vec = face.to_vec3();
            let v = face_vec.add_element_wise(position);

            let neighbor = chunk.get_block(v);
            match neighbor {
                Some(neighbor) => {
                    let mesh = match self.chunk_meshes.get_mut(chunk_index) {
                        Some(mesh) => mesh,
                        None => continue, // The current chunk's mesh is unavailable
                    };

                    match neighbor {
                        Block::Air(..) => if !is_air {
                            mesh.add_face(position, &face, &block);
                        },
                        _ => if is_air {
                            mesh.add_face(position, &face.get_opposite(), neighbor);
                        } else {
                            mesh.remove_face(position, &face);
                            mesh.remove_face(v, &face.get_opposite());
                        }
                    }
                },
                None => {
                    let (neighbor_chunk, neighbor_mesh) = match self.chunk_map.get(&Vector2::new(face_vec.x, face_vec.z).add_element_wise(chunk.world_offset)) {
                        Some(index) => match (self.chunks.get(*index), self.chunk_meshes.get_mut(*index)) {
                            (Some(chunk), Some(mesh)) => (chunk, mesh),
                            // Either the neighbor chunk or the chunk's mesh couldn't be found, but
                            // the chunk has an index in the map.
                            (None, None) | (None, Some(_)) | (Some(_), None) => continue,
                        },
                        None => {
                            match self.chunk_meshes.get_mut(chunk_index) {
                                Some(mesh) => {
                                    mesh.add_face(position, &face, &block);
                                    continue
                                },
                                None => continue,
                            }
                        },
                    };

                    let mut neighbor_chunk_block = None;
                    let neighbor_chunk_block_position = Vector3::new(v.x.rem_euclid(chunk::CHUNK_WIDTH as i32), v.y, v.z.rem_euclid(chunk::CHUNK_DEPTH as i32));
                    if !(0..16).contains(&v.x) || !(0..16).contains(&v.z) {
                        neighbor_chunk_block = neighbor_chunk.get_block(neighbor_chunk_block_position);
                    }

                    if !is_air {
                        if let Some(b) = neighbor_chunk_block {
                            match b {
                                Block::Air(..) => { 
                                    match self.chunk_meshes.get_mut(chunk_index) {
                                        Some(mesh) => mesh.add_face(position, &face, &block),
                                        None => continue,
                                    }
                                },
                                _ => neighbor_mesh.remove_face(neighbor_chunk_block_position, &face.get_opposite()),
                            }
                        } else {
                            match self.chunk_meshes.get_mut(chunk_index) {
                                Some(mesh) => mesh.add_face(position, &face, &block),
                                None => continue,
                            }
                        }
                    }
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

    pub fn chunk_map_iter(&mut self) -> hashbrown::hash_map::Iter<Vector2<i32>, usize> {
        self.chunk_map.iter()
    }

    pub fn chunk_map_iter_mut(&mut self) -> hashbrown::hash_map::IterMut<Vector2<i32>, usize> {
        self.chunk_map.iter_mut()
    }
}
