//! CPU-side voxel meshing for Book Edition chunk sections and columns.

use std::collections::HashMap;

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

/// Builds one section mesh at world origin.
pub fn build_chunk_mesh(chunk: &Chunk) -> ChunkMesh {
    build_chunk_region_mesh(&[(0, 0, vec![chunk.clone()])])
}

/// Builds one vertical chunk column and removes faces shared across sections.
pub fn build_chunk_column_mesh(sections: &[Chunk]) -> ChunkMesh {
    build_chunk_region_mesh(&[(0, 0, sections.to_vec())])
}

/// Builds a horizontal region of chunk columns.
///
/// Each tuple is (chunk_x, chunk_z, vertical_sections). Sections must be
/// ordered from the lowest section upward. Neighbor lookups cross horizontal
/// and vertical section boundaries, so shared faces are never emitted.
pub fn build_chunk_region_mesh(columns: &[(i32, i32, Vec<Chunk>)]) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let lookup = columns
        .iter()
        .enumerate()
        .map(|(index, (chunk_x, chunk_z, _))| ((*chunk_x, *chunk_z), index))
        .collect::<HashMap<_, _>>();

    for (chunk_x, chunk_z, sections) in columns {
        for (section_index, section) in sections.iter().enumerate() {
            for y in 0..CHUNK_EDGE {
                for z in 0..CHUNK_EDGE {
                    for x in 0..CHUNK_EDGE {
                        let block = section.get_block(x, y, z);
                        if block.is_air() {
                            continue;
                        }

                        for face in FACES {
                            let neighbor_x = x + face.neighbor[0];
                            let neighbor_y = y + face.neighbor[1];
                            let neighbor_z = z + face.neighbor[2];

                            let neighbor_section = section_index as i32
                                + if neighbor_y < 0 {
                                    -1
                                } else if neighbor_y >= CHUNK_EDGE {
                                    1
                                } else {
                                    0
                                };
                            let neighbor_chunk_x = *chunk_x + neighbor_x.div_euclid(CHUNK_EDGE);
                            let neighbor_chunk_z = *chunk_z + neighbor_z.div_euclid(CHUNK_EDGE);
                            let local_x = neighbor_x.rem_euclid(CHUNK_EDGE);
                            let local_y = neighbor_y.rem_euclid(CHUNK_EDGE);
                            let local_z = neighbor_z.rem_euclid(CHUNK_EDGE);

                            let neighbor = lookup
                                .get(&(neighbor_chunk_x, neighbor_chunk_z))
                                .and_then(|index| columns[*index].2.get(neighbor_section as usize))
                                .map(|neighbor_section| {
                                    neighbor_section.get_block(local_x, local_y, local_z)
                                })
                                .unwrap_or(BlockId::AIR);

                            if neighbor.is_air() {
                                append_face(
                                    &mut mesh.vertices,
                                    [
                                        chunk_x * CHUNK_EDGE + x,
                                        section_index as i32 * CHUNK_EDGE + y,
                                        chunk_z * CHUNK_EDGE + z,
                                    ],
                                    block,
                                    face,
                                );
                            }
                        }
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
    fn adjacent_vertical_sections_hide_their_shared_face() {
        let mut lower = Chunk::new();
        let mut upper = Chunk::new();
        assert!(lower.set_block(0, CHUNK_EDGE - 1, 0, BlockId::STONE));
        assert!(upper.set_block(0, 0, 0, BlockId::STONE));

        let mesh = build_chunk_column_mesh(&[lower, upper]);
        assert_eq!(mesh.vertex_count(), 60);
        assert!(mesh
            .vertices
            .iter()
            .any(|vertex| vertex.position[1] >= 16.0));
    }

    #[test]
    fn adjacent_horizontal_chunks_hide_their_shared_face() {
        let mut west = Chunk::new();
        let mut east = Chunk::new();
        assert!(west.set_block(CHUNK_EDGE - 1, 4, 7, BlockId::STONE));
        assert!(east.set_block(0, 4, 7, BlockId::STONE));

        let mesh = build_chunk_region_mesh(&[(0, 0, vec![west]), (1, 0, vec![east])]);
        assert_eq!(mesh.vertex_count(), 60);
    }

    #[test]
    fn negative_chunk_coordinates_are_written_in_world_space() {
        let mut chunk = Chunk::new();
        assert!(chunk.set_block(15, 3, 8, BlockId::GRASS));
        let mesh = build_chunk_region_mesh(&[(-1, 0, vec![chunk])]);

        assert!(mesh
            .vertices
            .iter()
            .any(|vertex| vertex.position[0] == -1.0));
    }

    #[test]
    fn generated_terrain_produces_a_non_empty_triangle_mesh() {
        let chunk = Chunk::generate_test_terrain();
        let mesh = build_chunk_mesh(&chunk);

        assert!(!mesh.is_empty());
        assert_eq!(mesh.vertex_count() % 3, 0);
    }
}
