use book_core::{BlockId, BlockRegistry};
use book_world::{ChunkSection, SECTION_EDGE};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub block_id: BlockId,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChunkMesh {
    pub vertices: Vec<MeshVertex>,
}

impl ChunkMesh {
    pub fn quad_count(&self) -> usize {
        self.vertices.len() / 6
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

/// Builds a CPU mesh and removes faces hidden by solid blocks in this section.
/// Faces at section borders stay visible when no adjacent section is supplied.
pub fn mesh_section(
    section: &ChunkSection,
    registry: &BlockRegistry,
    atlas_columns: u16,
    atlas_rows: u16,
) -> ChunkMesh {
    mesh_section_with_vertical_neighbors(section, None, None, registry, atlas_columns, atlas_rows)
}

/// Builds a section mesh while checking the sections immediately below and above it.
/// This prevents internal horizontal faces from being emitted at vertical section seams.
pub fn mesh_section_with_vertical_neighbors(
    section: &ChunkSection,
    below: Option<&ChunkSection>,
    above: Option<&ChunkSection>,
    registry: &BlockRegistry,
    atlas_columns: u16,
    atlas_rows: u16,
) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();

    if atlas_columns == 0 || atlas_rows == 0 {
        return mesh;
    }

    for y in 0..SECTION_EDGE {
        for z in 0..SECTION_EDGE {
            for x in 0..SECTION_EDGE {
                let Some(block_id) = section.get(x, y, z) else {
                    continue;
                };
                let Some(definition) = registry.get(block_id) else {
                    continue;
                };
                if !definition.solid {
                    continue;
                }

                let tile = definition.atlas_tile;
                let u0 = tile.x as f32 / atlas_columns as f32;
                let v0 = tile.y as f32 / atlas_rows as f32;
                let u1 = (tile.x + 1) as f32 / atlas_columns as f32;
                let v1 = (tile.y + 1) as f32 / atlas_rows as f32;
                let px = x as f32;
                let py = y as f32;
                let pz = z as f32;

                for face in FACES.iter() {
                    if neighbor_is_solid(section, below, above, registry, x, y, z, face.neighbor) {
                        continue;
                    }

                    let uvs = [
                        [u0, v1], [u1, v1], [u1, v0],
                        [u0, v1], [u1, v0], [u0, v0],
                    ];

                    for index in 0..6 {
                        let p = face.positions[index];
                        mesh.vertices.push(MeshVertex {
                            position: [px + p[0], py + p[1], pz + p[2]],
                            uv: uvs[index],
                            block_id,
                        });
                    }
                }
            }
        }
    }

    mesh
}

fn neighbor_is_solid(
    section: &ChunkSection,
    below: Option<&ChunkSection>,
    above: Option<&ChunkSection>,
    registry: &BlockRegistry,
    x: usize,
    y: usize,
    z: usize,
    offset: [i32; 3],
) -> bool {
    let nx = x as i32 + offset[0];
    let ny = y as i32 + offset[1];
    let nz = z as i32 + offset[2];

    // Only vertical section seams are resolved here. Horizontal chunk seams
    // will be handled by the world-aware meshing stage.
    let neighbor = if nx < 0 || nx >= SECTION_EDGE as i32 || nz < 0 || nz >= SECTION_EDGE as i32 {
        return false;
    } else if ny < 0 {
        if ny != -1 {
            return false;
        }
        below.and_then(|neighbor| neighbor.get(nx as usize, SECTION_EDGE - 1, nz as usize))
    } else if ny >= SECTION_EDGE as i32 {
        if ny != SECTION_EDGE as i32 {
            return false;
        }
        above.and_then(|neighbor| neighbor.get(nx as usize, 0, nz as usize))
    } else {
        section.get(nx as usize, ny as usize, nz as usize)
    };

    neighbor
        .and_then(|id| registry.get(id))
        .map(|definition| definition.solid)
        .unwrap_or(false)
}

struct Face {
    neighbor: [i32; 3],
    positions: [[f32; 3]; 6],
}

const FACES: [Face; 6] = [
    Face {
        neighbor: [0, 0, -1],
        positions: [
            [0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0],
        ],
    },
    Face {
        neighbor: [0, 0, 1],
        positions: [
            [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0],
            [0.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0],
        ],
    },
    Face {
        neighbor: [-1, 0, 0],
        positions: [
            [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 1.0],
            [0.0, 0.0, 0.0], [0.0, 1.0, 1.0], [0.0, 1.0, 0.0],
        ],
    },
    Face {
        neighbor: [1, 0, 0],
        positions: [
            [1.0, 0.0, 0.0], [1.0, 1.0, 1.0], [1.0, 0.0, 1.0],
            [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0],
        ],
    },
    Face {
        neighbor: [0, -1, 0],
        positions: [
            [0.0, 0.0, 0.0], [1.0, 0.0, 1.0], [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 1.0],
        ],
    },
    Face {
        neighbor: [0, 1, 0],
        positions: [
            [0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0],
            [0.0, 1.0, 0.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0],
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use book_core::{AtlasTile, BlockDefinition};

    fn registry() -> BlockRegistry {
        let mut registry = BlockRegistry::new();
        registry.register(BlockDefinition {
            id: BlockId(1),
            name: "book:stone".to_owned(),
            atlas_tile: AtlasTile { x: 0, y: 0 },
            solid: true,
            opaque: true,
        });
        registry
    }

    #[test]
    fn one_block_generates_six_quads() {
        let mut section = ChunkSection::new();
        section.set(1, 1, 1, BlockId(1));
        let mesh = mesh_section(&section, &registry(), 16, 16);
        assert_eq!(mesh.quad_count(), 6);
        assert_eq!(mesh.vertices.len(), 36);
    }

    #[test]
    fn adjacent_blocks_hide_shared_faces() {
        let mut section = ChunkSection::new();
        section.set(1, 1, 1, BlockId(1));
        section.set(2, 1, 1, BlockId(1));
        let mesh = mesh_section(&section, &registry(), 16, 16);
        assert_eq!(mesh.quad_count(), 10);
    }

    #[test]
    fn vertical_neighbor_sections_hide_shared_faces() {
        let mut lower = ChunkSection::new();
        let mut upper = ChunkSection::new();
        lower.set(4, SECTION_EDGE - 1, 7, BlockId(1));
        upper.set(4, 0, 7, BlockId(1));

        let registry = registry();
        let lower_mesh = mesh_section_with_vertical_neighbors(
            &lower, None, Some(&upper), &registry, 16, 16,
        );
        let upper_mesh = mesh_section_with_vertical_neighbors(
            &upper, Some(&lower), None, &registry, 16, 16,
        );

        assert_eq!(lower_mesh.quad_count(), 5);
        assert_eq!(upper_mesh.quad_count(), 5);
        assert_eq!(lower_mesh.quad_count() + upper_mesh.quad_count(), 10);
    }

    #[test]
    fn missing_vertical_neighbor_keeps_border_face_visible() {
        let mut section = ChunkSection::new();
        section.set(4, SECTION_EDGE - 1, 7, BlockId(1));
        let mesh = mesh_section_with_vertical_neighbors(
            &section, None, None, &registry(), 16, 16,
        );
        assert_eq!(mesh.quad_count(), 6);
    }

    #[test]
    fn zero_sized_atlas_returns_empty_mesh() {
        let mut section = ChunkSection::new();
        section.set(1, 1, 1, BlockId(1));
        let mesh = mesh_section(&section, &registry(), 0, 16);
        assert!(mesh.is_empty());
    }
}
