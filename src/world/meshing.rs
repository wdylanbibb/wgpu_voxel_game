use crate::{
    renderer::model::ChunkVertex,
    world::{
        World,
        block::{AtlasTile, BlockId, BlockTextures},
        chunk::CHUNK_SIZE,
    },
};

const ATLAS_SIZE: f32 = 256.0;
const TILE_SIZE: f32 = 16.0;

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
) -> ChunkVertex {
    ChunkVertex {
        position: [
            origin[0] + block_position[0] as f32 + face_corner[0],
            origin[1] + block_position[1] as f32 + face_corner[1],
            origin[2] + block_position[2] as f32 + face_corner[2],
        ],
        tex_coord,
    }
}

fn tile_for_face(textures: BlockTextures, face: FaceTexture) -> AtlasTile {
    match face {
        FaceTexture::Top => textures.top,
        FaceTexture::Side => textures.side,
        FaceTexture::Bottom => textures.bottom,
    }
}

fn atlas_uvs(tile: AtlasTile) -> [[f32; 2]; 4] {
    // Keep samples half a texel inside the tile to avoid bleeding into an
    // adjacent sprite at face edges.
    let inset = 0.5;
    let u_min = (tile.x as f32 * TILE_SIZE + inset) / ATLAS_SIZE;
    let v_min = (tile.y as f32 * TILE_SIZE + inset) / ATLAS_SIZE;
    let u_max = ((tile.x as f32 + 1.0) * TILE_SIZE - inset) / ATLAS_SIZE;
    let v_max = ((tile.y as f32 + 1.0) * TILE_SIZE - inset) / ATLAS_SIZE;

    [
        [u_min, v_max],
        [u_min, v_min],
        [u_max, v_min],
        [u_max, v_max],
    ]
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

fn emit_face(
    vertices: &mut Vec<ChunkVertex>,
    indices: &mut Vec<u32>,
    block_position: [usize; 3],
    origin: [f32; 3],
    face: &Face,
    tile: AtlasTile,
) {
    let first_vertex = vertices.len() as u32;
    let uvs = atlas_uvs(tile);

    for (corner, tex_coord) in face.corners.iter().zip(uvs) {
        vertices.push(make_vertex(block_position, *corner, origin, tex_coord));
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
    let chunk = world
        .chunks
        .get(&chunk_pos)
        .unwrap_or_else(|| panic!("mesh_chunk: chunk {chunk_pos:?} not found"));

    let origin = chunk_origin(chunk_pos);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let block = chunk.get(x, y, z);

                if block == BlockId::Air {
                    continue;
                }

                let Some(textures) = block.definition().textures else {
                    continue;
                };

                for face in &FACES {
                    let neighbor_position = [
                        x as i32 + face.neighbor[0],
                        y as i32 + face.neighbor[1],
                        z as i32 + face.neighbor[2],
                    ];

                    if block_at(world, chunk_pos, neighbor_position)
                        .definition()
                        .opaque
                    {
                        continue;
                    }

                    emit_face(
                        &mut vertices,
                        &mut indices,
                        [x, y, z],
                        origin,
                        face,
                        tile_for_face(textures, face.texture),
                    );
                }
            }
        }
    }

    CpuChunkMesh { vertices, indices }
}
