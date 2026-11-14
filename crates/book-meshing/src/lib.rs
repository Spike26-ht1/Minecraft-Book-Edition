use book_core::{BlockId, BlockRegistry};
use book_world::{ChunkManager, ChunkPos, ChunkSection, SECTION_EDGE};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
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

/// Neighbor sections surrounding a section in the six cardinal directions.
#[derive(Clone, Copy, Debug, Default)]
pub struct SectionNeighbors<'a> {
    pub west: Option<&'a ChunkSection>,
    pub east: Option<&'a ChunkSection>,
    pub north: Option<&'a ChunkSection>,
    pub south: Option<&'a ChunkSection>,
    pub below: Option<&'a ChunkSection>,
    pub above: Option<&'a ChunkSection>,
}

/// Builds a CPU mesh and removes faces hidden by solid blocks in this section.
/// Faces at section borders stay visible when no adjacent section is supplied.
pub fn mesh_section(
    section: &ChunkSection,
    registry: &BlockRegistry,
    atlas_columns: u16,
    atlas_rows: u16,
) -> ChunkMesh {
    mesh_section_with_neighbors(section, SectionNeighbors::default(), registry, atlas_columns, atlas_rows)
}

/// Builds a section mesh using all available cardinal neighbor sections.
pub fn mesh_section_with_neighbors(
    section: &ChunkSection,
    neighbors: SectionNeighbors<'_>,
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
                    if neighbor_is_solid(section, neighbors, registry, x, y, z, face.neighbor) {
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
                            normal: face.normal,
                            block_id,
                        });
                    }
                }
            }
        }
    }

    mesh
}

/// Builds a mesh directly from a loaded world chunk and its loaded neighbors.
/// Returns None when the requested chunk column or section is not loaded.
pub fn mesh_chunk_section(
    manager: &ChunkManager,
    position: ChunkPos,
    section_y: i32,
    registry: &BlockRegistry,
    atlas_columns: u16,
    atlas_rows: u16,
) -> Option<ChunkMesh> {
    let column = manager.column(position)?;
    let section = column.section(section_y)?;

    let neighbors = SectionNeighbors {
        west: position.x.checked_sub(1)
            .and_then(|x| manager.column(ChunkPos { x, z: position.z }))
            .and_then(|column| column.section(section_y)),
        east: position.x.checked_add(1)
            .and_then(|x| manager.column(ChunkPos { x, z: position.z }))
            .and_then(|column| column.section(section_y)),
        north: position.z.checked_sub(1)
            .and_then(|z| manager.column(ChunkPos { x: position.x, z }))
            .and_then(|column| column.section(section_y)),
        south: position.z.checked_add(1)
            .and_then(|z| manager.column(ChunkPos { x: position.x, z }))
            .and_then(|column| column.section(section_y)),
        below: section_y.checked_sub(1).and_then(|y| column.section(y)),
        above: section_y.checked_add(1).and_then(|y| column.section(y)),
    };

    Some(mesh_section_with_neighbors(
        section, neighbors, registry, atlas_columns, atlas_rows,
    ))
}

fn neighbor_is_solid(
    section: &ChunkSection,
    neighbors: SectionNeighbors<'_>,
    registry: &BlockRegistry,
    x: usize,
    y: usize,
    z: usize,
    offset: [i32; 3],
) -> bool {
    let nx = x as i32 + offset[0];
    let ny = y as i32 + offset[1];
    let nz = z as i32 + offset[2];
    let edge = SECTION_EDGE as i32;

    let neighbor = if nx < 0 {
        neighbors.west.and_then(|n| n.get(SECTION_EDGE - 1, y, z))
    } else if nx >= edge {
        neighbors.east.and_then(|n| n.get(0, y, z))
    } else if nz < 0 {
        neighbors.north.and_then(|n| n.get(x, y, SECTION_EDGE - 1))
    } else if nz >= edge {
        neighbors.south.and_then(|n| n.get(x, y, 0))
    } else if ny < 0 {
        neighbors.below.and_then(|n| n.get(x, SECTION_EDGE - 1, z))
    } else if ny >= edge {
        neighbors.above.and_then(|n| n.get(x, 0, z))
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
    normal: [f32; 3],
    positions: [[f32; 3]; 6],
}

const FACES: [Face; 6] = [
    Face {
        neighbor: [0, 0, -1],
        normal: [0.0, 0.0, -1.0],
        positions: [
            [0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0],
        ],
    },
    Face {
        neighbor: [0, 0, 1],
        normal: [0.0, 0.0, 1.0],
        positions: [
            [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0],
            [0.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0],
        ],
    },
    Face {
        neighbor: [-1, 0, 0],
        normal: [-1.0, 0.0, 0.0],
        positions: [
            [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 1.0],
            [0.0, 0.0, 0.0], [0.0, 1.0, 1.0], [0.0, 1.0, 0.0],
        ],
    },
    Face {
        neighbor: [1, 0, 0],
        normal: [1.0, 0.0, 0.0],
        positions: [
            [1.0, 0.0, 0.0], [1.0, 1.0, 1.0], [1.0, 0.0, 1.0],
            [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0],
        ],
    },
    Face {
        neighbor: [0, -1, 0],
        normal: [0.0, -1.0, 0.0],
        positions: [
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0],
            [0.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0],
        ],
    },
    Face {
        neighbor: [0, 1, 0],
        normal: [0.0, 1.0, 0.0],
        positions: [
            [0.0, 1.0, 0.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0], [0.0, 1.0, 1.0], [1.0, 1.0, 1.0],
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
    fn all_face_windings_match_their_outward_normals() {
        for face in FACES.iter() {
            let edge_a = [
                face.positions[1][0] - face.positions[0][0],
                face.positions[1][1] - face.positions[0][1],
                face.positions[1][2] - face.positions[0][2],
            ];
            let edge_b = [
                face.positions[2][0] - face.positions[0][0],
                face.positions[2][1] - face.positions[0][1],
                face.positions[2][2] - face.positions[0][2],
            ];
            let cross = [
                edge_a[1] * edge_b[2] - edge_a[2] * edge_b[1],
                edge_a[2] * edge_b[0] - edge_a[0] * edge_b[2],
                edge_a[0] * edge_b[1] - edge_a[1] * edge_b[0],
            ];
            let dot = cross[0] * face.normal[0]
                + cross[1] * face.normal[1]
                + cross[2] * face.normal[2];
            assert!(dot > 0.0, "face winding disagrees with normal {:?}", face.normal);
        }
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
        let lower_mesh = mesh_section_with_neighbors(
            &lower, SectionNeighbors { above: Some(&upper), ..Default::default() },
            &registry, 16, 16,
        );
        let upper_mesh = mesh_section_with_neighbors(
            &upper, SectionNeighbors { below: Some(&lower), ..Default::default() },
            &registry, 16, 16,
        );

        assert_eq!(lower_mesh.quad_count(), 5);
        assert_eq!(upper_mesh.quad_count(), 5);
        assert_eq!(lower_mesh.quad_count() + upper_mesh.quad_count(), 10);
    }

    #[test]
    fn horizontal_neighbor_sections_hide_shared_faces() {
        let mut center = ChunkSection::new();
        let mut east = ChunkSection::new();
        center.set(SECTION_EDGE - 1, 3, 6, BlockId(1));
        east.set(0, 3, 6, BlockId(1));

        let registry = registry();
        let center_mesh = mesh_section_with_neighbors(
            &center,
            SectionNeighbors { east: Some(&east), ..Default::default() },
            &registry, 16, 16,
        );
        let east_mesh = mesh_section_with_neighbors(
            &east,
            SectionNeighbors { west: Some(&center), ..Default::default() },
            &registry, 16, 16,
        );

        assert_eq!(center_mesh.quad_count(), 5);
        assert_eq!(east_mesh.quad_count(), 5);
    }

    #[test]
    fn world_mesher_reads_loaded_neighbor_chunks() {
        use book_world::{ChunkColumn, ChunkManager, ChunkPos};

        let mut manager = ChunkManager::new();
        let center_pos = ChunkPos { x: 0, z: 0 };
        let east_pos = ChunkPos { x: 1, z: 0 };
        let mut center = ChunkColumn::new(center_pos);
        let mut east = ChunkColumn::new(east_pos);
        center.section_mut(0).set(SECTION_EDGE - 1, 3, 6, BlockId(1));
        east.section_mut(0).set(0, 3, 6, BlockId(1));
        manager.insert_column(center);
        manager.insert_column(east);

        let mesh = mesh_chunk_section(&manager, center_pos, 0, &registry(), 16, 16)
            .expect("loaded center section should be meshed");
        assert_eq!(mesh.quad_count(), 5);
        assert!(mesh_chunk_section(&manager, ChunkPos { x: 8, z: 8 }, 0, &registry(), 16, 16).is_none());
    }

    #[test]
    fn missing_vertical_neighbor_keeps_border_face_visible() {
        let mut section = ChunkSection::new();
        section.set(4, SECTION_EDGE - 1, 7, BlockId(1));
        let mesh = mesh_section(&section, &registry(), 16, 16);
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
