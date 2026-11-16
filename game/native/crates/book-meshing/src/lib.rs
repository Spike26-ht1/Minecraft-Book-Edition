//! CPU-side voxel meshing for Book Edition chunk sections and columns.

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

/// Builds one section mesh. Faces at section boundaries remain visible because
/// this function has no neighboring-section context.
pub fn build_chunk_mesh(chunk: &Chunk) -> ChunkMesh {
    build_section_mesh(std::slice::from_ref(chunk), 0, 0)
}

/// Builds a vertical chunk column and removes faces shared across section
/// boundaries. Sections must be ordered from the lowest section upward.
pub fn build_chunk_column_mesh(sections: &[Chunk]) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();

    for (section_index, section) in sections.iter().enumerate() {
        let section_mesh = build_section_mesh(sections, section_index, section_index as i32);
        mesh.vertices.extend(section_mesh.vertices);
    }

    mesh
}

fn build_section_mesh(
    sections: &[Chunk],
    section_index: usize,
    vertical_offset_sections: i32,
) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();

    for y in 0..CHUNK_EDGE {
        for z in 0..CHUNK_EDGE {
            for x in 0..CHUNK_EDGE {
                let block = sections[section_index].get_block(x, y, z);
                if block.is_air() {
                    continue;
                }

                for face in FACES {
                    let neighbor = neighbor_block(
                        sections,
                        section_index,
                        x + face.neighbor[0],
                        y + face.neighbor[1],
                        z + face.neighbor[2],
                    );

                    if neighbor.is_air() {
                        append_face(
                            &mut mesh.vertices,
                            [x, y + vertical_offset_sections * CHUNK_EDGE, z],
                            block,
                            face,
                        );
                    }
                }
            }
        }
    }

    mesh
}

fn neighbor_block(
    sections: &[Chunk],
    section_index: usize,
    x: i32,
    y: i32,
    z: i32,
) -> BlockId {
    if y < 0 {
        return section_index
            .checked_sub(1)
            .and_then(|index| sections.get(index))
            .map(|section| section.get_block(x, CHUNK_EDGE - 1, z))
            .unwrap_or(BlockId::AIR);
    }

    if y >= CHUNK_EDGE {
        return sections
            .get(section_index + 1)
            .map(|section| section.get_block(x, 0, z))
            .unwrap_or(BlockId::AIR);
    }

    sections[section_index].get_block(x, y, z)
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
        assert!(mesh.vertices.iter().any(|vertex| vertex.position[1] >= 16.0));
    }

    #[test]
    fn generated_terrain_produces_a_non_empty_triangle_mesh() {
        let chunk = Chunk::generate_test_terrain();
        let mesh = build_chunk_mesh(&chunk);

        assert!(!mesh.is_empty());
        assert_eq!(mesh.vertex_count() % 3, 0);
    }
}
