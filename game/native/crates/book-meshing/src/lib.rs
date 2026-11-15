//! CPU-side voxel meshing for Book Edition chunk sections.

use book_core::CHUNK_EDGE;
use book_world::{BlockId, Chunk};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

#[derive(Debug, Clone, Default)]
pub struct ChunkMesh {
    pub vertices: Vec<MeshVertex>,
}

impl ChunkMesh {
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

#[derive(Clone, Copy)]
struct Face {
    neighbor: [i32; 3],
    corners: [[f32; 3]; 4],
    shade: f32,
}

const FACES: [Face; 6] = [
    Face {
        neighbor: [0, 1, 0],
        corners: [
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ],
        shade: 1.0,
    },
    Face {
        neighbor: [0, -1, 0],
        corners: [
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        ],
        shade: 0.48,
    },
    Face {
        neighbor: [0, 0, 1],
        corners: [
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ],
        shade: 0.78,
    },
    Face {
        neighbor: [0, 0, -1],
        corners: [
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ],
        shade: 0.68,
    },
    Face {
        neighbor: [1, 0, 0],
        corners: [
            [1.0, 0.0, 1.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
        ],
        shade: 0.72,
    },
    Face {
        neighbor: [-1, 0, 0],
        corners: [
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 1.0, 0.0],
        ],
        shade: 0.62,
    },
];

/// Builds a chunk mesh and removes faces shared by two solid blocks.
///
/// At the moment neighbor queries are limited to the supplied chunk. The
/// world-level boundary provider will be added when multi-chunk meshing lands.
pub fn build_chunk_mesh(chunk: &Chunk) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();

    for y in 0..CHUNK_EDGE {
        for z in 0..CHUNK_EDGE {
            for x in 0..CHUNK_EDGE {
                let block = chunk.get_block(x, y, z);
                if block.is_air() {
                    continue;
                }

                for face in FACES {
                    let neighbor = chunk.get_block(
                        x + face.neighbor[0],
                        y + face.neighbor[1],
                        z + face.neighbor[2],
                    );

                    if neighbor.is_air() {
                        append_face(&mut mesh.vertices, [x, y, z], block, face);
                    }
                }
            }
        }
    }

    mesh
}

fn append_face(output: &mut Vec<MeshVertex>, block_position: [i32; 3], block: BlockId, face: Face) {
    let base_color = block.debug_color();
    let indices = [0_usize, 1, 2, 0, 2, 3];

    for index in indices {
        let corner = face.corners[index];
        output.push(MeshVertex {
            position: [
                block_position[0] as f32 + corner[0],
                block_position[1] as f32 + corner[1],
                block_position[2] as f32 + corner[2],
            ],
            color: [
                base_color[0] * face.shade,
                base_color[1] * face.shade,
                base_color[2] * face.shade,
            ],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_solid_block_has_six_visible_faces() {
        let mut chunk = Chunk::new();
        assert!(chunk.set_block(0, 0, 0, BlockId::STONE));

        let mesh = build_chunk_mesh(&chunk);
        assert_eq!(mesh.vertex_count(), 36);
    }

    #[test]
    fn adjacent_blocks_hide_the_shared_face() {
        let mut chunk = Chunk::new();
        assert!(chunk.set_block(0, 0, 0, BlockId::STONE));
        assert!(chunk.set_block(1, 0, 0, BlockId::STONE));

        let mesh = build_chunk_mesh(&chunk);
        assert_eq!(mesh.vertex_count(), 60);
    }

    #[test]
    fn generated_terrain_produces_a_non_empty_triangle_mesh() {
        let chunk = Chunk::generate_test_terrain();
        let mesh = build_chunk_mesh(&chunk);

        assert!(!mesh.is_empty());
        assert_eq!(mesh.vertex_count() % 3, 0);
    }
}
