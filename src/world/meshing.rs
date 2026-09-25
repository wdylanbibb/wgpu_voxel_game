use crate::{
    renderer::model::ChunkVertex,
    world::{
        World,
        block::{AtlasTile, BlockId, BlockTextures},
        chunk::CHUNK_SIZE,
    },
};

#[derive(Clone, Copy)]
enum FaceTexture {
    Top,
    Side,
    Bottom,
}

struct Face {
    neighbor: [i32; 3],
    corners: [[f32; 3]; 4],
    texture: FaceTexture,
}

// Vertices for each face are counter-clockwise when viewed from outside the
// block, matching the chunk pipeline's CCW front face.
const FACES: [Face; 6] = [
    Face {
        neighbor: [1, 0, 0],
        corners: [
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
            [1.0, 0.0, 1.0],
        ],
        texture: FaceTexture::Side,
    },
    Face {
        neighbor: [-1, 0, 0],
        corners: [
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ],
        texture: FaceTexture::Side,
    },
    Face {
        neighbor: [0, 1, 0],
        corners: [
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 0.0],
        ],
        texture: FaceTexture::Top,
    },
    Face {
        neighbor: [0, -1, 0],
        corners: [
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
        ],
        texture: FaceTexture::Bottom,
    },
    Face {
        neighbor: [0, 0, 1],
        corners: [
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
        texture: FaceTexture::Side,
    },
    Face {
        neighbor: [0, 0, -1],
        corners: [
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
        ],
        texture: FaceTexture::Side,
    },
];

pub struct CpuChunkMesh {
    pub vertices: Vec<ChunkVertex>,
    pub indices: Vec<u32>,
}

fn chunk_origin(chunk_pos: cgmath::Vector3<i32>) -> [f32; 3] {
    let size = CHUNK_SIZE as f32;

    [
        chunk_pos.x as f32 * size,
        chunk_pos.y as f32 * size,
        chunk_pos.z as f32 * size,
    ]
}

fn make_vertex(
    block_position: [usize; 3],
    face_corner: [f32; 3],
    origin: [f32; 3],
    tex_coord: [f32; 2],
    normal: [f32; 3],
    atlas_tile: AtlasTile,
) -> ChunkVertex {
    ChunkVertex {
        position: [
            origin[0] + block_position[0] as f32 + face_corner[0],
            origin[1] + block_position[1] as f32 + face_corner[1],
            origin[2] + block_position[2] as f32 + face_corner[2],
        ],
        tex_coord,
        normal,
        atlas_tile: [atlas_tile.x as f32, atlas_tile.y as f32],
    }
}

fn tile_for_face(textures: BlockTextures, face: FaceTexture) -> AtlasTile {
    match face {
        FaceTexture::Top => textures.top,
        FaceTexture::Side => textures.side,
        FaceTexture::Bottom => textures.bottom,
    }
}

fn repeating_uvs(width: usize, height: usize) -> [[f32; 2]; 4] {
    let width = width as f32;
    let height = height as f32;
    [[0.0, height], [0.0, 0.0], [width, 0.0], [width, height]]
}

fn block_at(world: &World, chunk_pos: cgmath::Vector3<i32>, local_position: [i32; 3]) -> BlockId {
    let size = CHUNK_SIZE as i32;
    let containing_chunk = chunk_pos
        + cgmath::Vector3::new(
            local_position[0].div_euclid(size),
            local_position[1].div_euclid(size),
            local_position[2].div_euclid(size),
        );

    let local_x = local_position[0].rem_euclid(size) as usize;
    let local_y = local_position[1].rem_euclid(size) as usize;
    let local_z = local_position[2].rem_euclid(size) as usize;

    world
        .chunks
        .get(&containing_chunk)
        .map(|chunk| chunk.get(local_x, local_y, local_z))
        .unwrap_or(BlockId::Air)
}

fn greedy_cell_position(direction: usize, plane: usize, u: usize, v: usize) -> [usize; 3] {
    match direction {
        // X faces: horizontal mask axis is Z, vertical mask axis is Y.
        0 | 1 => [plane, v, u],
        // Y faces: horizontal mask axis is X, vertical mask axis is Z.
        2 | 3 => [u, plane, v],
        // Z faces: horizontal mask axis is X, vertical mask axis is Y.
        4 | 5 => [u, v, plane],
        _ => unreachable!("invalid face direction"),
    }
}

fn greedy_quad_start(
    direction: usize,
    plane: usize,
    u: usize,
    v: usize,
    width: usize,
    height: usize,
) -> [usize; 3] {
    // Each face's first corner points along a different pair of local axes.
    // Move the starting block for negatively directed axes so the scaled quad
    // still covers the rectangle beginning at (u, v).
    match direction {
        0 => [plane, v, u],
        1 => [plane, v, u + width - 1],
        2 => [u, plane, v],
        3 => [u, plane, v + height - 1],
        4 => [u + width - 1, v, plane],
        5 => [u, v, plane],
        _ => unreachable!("invalid face direction"),
    }
}

fn emit_greedy_face(
    vertices: &mut Vec<ChunkVertex>,
    indices: &mut Vec<u32>,
    block_position: [usize; 3],
    origin: [f32; 3],
    face: &Face,
    tile: AtlasTile,
    width: usize,
    height: usize,
) {
    let first_vertex = vertices.len() as u32;
    let uvs = repeating_uvs(width, height);
    let normal = [
        face.neighbor[0] as f32,
        face.neighbor[1] as f32,
        face.neighbor[2] as f32,
    ];
    let first = face.corners[0];
    let width_direction = [
        face.corners[3][0] - first[0],
        face.corners[3][1] - first[1],
        face.corners[3][2] - first[2],
    ];
    let height_direction = [
        face.corners[1][0] - first[0],
        face.corners[1][1] - first[1],
        face.corners[1][2] - first[2],
    ];
    let width = width as f32;
    let height = height as f32;
    let corners = [
        first,
        [
            first[0] + height_direction[0] * height,
            first[1] + height_direction[1] * height,
            first[2] + height_direction[2] * height,
        ],
        [
            first[0] + height_direction[0] * height + width_direction[0] * width,
            first[1] + height_direction[1] * height + width_direction[1] * width,
            first[2] + height_direction[2] * height + width_direction[2] * width,
        ],
        [
            first[0] + width_direction[0] * width,
            first[1] + width_direction[1] * width,
            first[2] + width_direction[2] * width,
        ],
    ];

    for (corner, tex_coord) in corners.into_iter().zip(uvs) {
        vertices.push(make_vertex(
            block_position,
            corner,
            origin,
            tex_coord,
            normal,
            tile,
        ));
    }

    indices.extend_from_slice(&[
        first_vertex,
        first_vertex + 1,
        first_vertex + 2,
        first_vertex,
        first_vertex + 2,
        first_vertex + 3,
    ]);
}

pub fn mesh_chunk(world: &World, chunk_pos: cgmath::Vector3<i32>) -> CpuChunkMesh {
    mesh_chunk_greedy(world, chunk_pos)
}

/// Mesh a chunk by merging coplanar, visible faces that use the same atlas tile.
///
/// This emits one quad per maximal rectangle instead of one per block face.
/// The chunk shader wraps each quad's tile-local UVs, so atlas tiles repeat
/// across merged faces instead of stretching.
pub fn mesh_chunk_greedy(world: &World, chunk_pos: cgmath::Vector3<i32>) -> CpuChunkMesh {
    let chunk = world
        .chunks
        .get(&chunk_pos)
        .unwrap_or_else(|| panic!("mesh_chunk_greedy: chunk {chunk_pos:?} not found"));

    let origin = chunk_origin(chunk_pos);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for (direction, face) in FACES.iter().enumerate() {
        for plane in 0..CHUNK_SIZE {
            let mut mask = vec![None; CHUNK_SIZE * CHUNK_SIZE];

            for v in 0..CHUNK_SIZE {
                for u in 0..CHUNK_SIZE {
                    let position = greedy_cell_position(direction, plane, u, v);
                    let block = chunk.get(position[0], position[1], position[2]);
                    let Some(textures) = block.definition().textures else {
                        continue;
                    };

                    let neighbor_position = [
                        position[0] as i32 + face.neighbor[0],
                        position[1] as i32 + face.neighbor[1],
                        position[2] as i32 + face.neighbor[2],
                    ];
                    if !block_at(world, chunk_pos, neighbor_position)
                        .definition()
                        .opaque
                    {
                        mask[u + CHUNK_SIZE * v] = Some(tile_for_face(textures, face.texture));
                    }
                }
            }

            for v in 0..CHUNK_SIZE {
                let mut u = 0;
                while u < CHUNK_SIZE {
                    let Some(tile) = mask[u + CHUNK_SIZE * v] else {
                        u += 1;
                        continue;
                    };

                    let mut width = 1;
                    while u + width < CHUNK_SIZE && mask[u + width + CHUNK_SIZE * v] == Some(tile) {
                        width += 1;
                    }

                    let mut height = 1;
                    'grow: while v + height < CHUNK_SIZE {
                        for column in u..u + width {
                            if mask[column + CHUNK_SIZE * (v + height)] != Some(tile) {
                                break 'grow;
                            }
                        }
                        height += 1;
                    }

                    for row in v..v + height {
                        for column in u..u + width {
                            mask[column + CHUNK_SIZE * row] = None;
                        }
                    }

                    emit_greedy_face(
                        &mut vertices,
                        &mut indices,
                        greedy_quad_start(direction, plane, u, v, width, height),
                        origin,
                        face,
                        tile,
                        width,
                        height,
                    );
                    u += width;
                }
            }
        }
    }

    CpuChunkMesh { vertices, indices }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use cgmath::Vector3;

    use super::{CHUNK_SIZE, ChunkVertex, mesh_chunk_greedy};
    use crate::world::{World, block::BlockId, chunk::Chunk};

    #[test]
    fn merges_a_solid_chunk_into_six_quads() {
        let mut chunk = Chunk::new();
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    chunk.set(x, y, z, BlockId::Dirt);
                }
            }
        }

        let chunk_pos = Vector3::new(0, 0, 0);
        let world = World::from_chunks(HashMap::from([(chunk_pos, chunk)]));
        let mesh = mesh_chunk_greedy(&world, chunk_pos);

        assert_eq!(mesh.vertices.len(), 6 * 4);
        assert_eq!(mesh.indices.len(), 6 * 6);
        assert!(mesh.vertices.iter().all(|vertex: &ChunkVertex| {
            vertex
                .position
                .iter()
                .all(|coordinate| (0.0..=16.0).contains(coordinate))
        }));
    }
}
