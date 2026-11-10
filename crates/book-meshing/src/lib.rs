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
/// Faces at section borders stay visible until adjacent-section data is available.
pub fn mesh_section(
    section: &ChunkSection,
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
                    if neighbor_is_solid(section, registry, x, y, z, face.neighbor) {
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
    registry: &BlockRegistry,
    x: usize,
    y: usize,
    z: usize,
    offset: [i32; 3],
) -> bool {
    let nx = x as i32 + offset[0];
    let ny = y as i32 + offset[1];
    let nz = z as i32 + offset[2];

    if nx < 0 || ny < 0 || nz < 0 {
        return false;
    }

    let Some(neighbor) = section.get(nx as usize, ny as usize, nz as usize) else {
        return false;
    };

    registry
        .get(neighbor)
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
    fn zero_sized_atlas_returns_empty_mesh() {
        let mut section = ChunkSection::new();
        section.set(1, 1, 1, BlockId(1));
        let mesh = mesh_section(&section, &registry(), 0, 16);
        assert!(mesh.is_empty());
    }
}
