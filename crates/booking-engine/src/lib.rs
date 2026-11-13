//! Booking is the world and simulation engine for Minecraft Book Edition.
//!
//! This crate owns block registries, loaded world state, terrain creation, and
//! CPU-side chunk mesh preparation. Graphics APIs consume the prepared meshes;
//! they do not own or generate gameplay world data.

use book_core::{AtlasTile, BlockDefinition, BlockId, BlockRegistry};
use book_meshing::{mesh_chunk_section, ChunkMesh};
use book_world::{ChunkColumn, ChunkManager, ChunkPos, SECTION_EDGE};

/// CPU mesh for one world section, ready to be uploaded by a renderer backend.
#[derive(Clone, Debug)]
pub struct WorldSectionMesh {
    pub position: ChunkPos,
    pub section_y: i32,
    pub mesh: ChunkMesh,
}

/// Main world-engine state owned by Booking.
#[derive(Clone, Debug)]
pub struct BookingEngine {
    blocks: BlockRegistry,
    world: ChunkManager,
}

impl Default for BookingEngine {
    fn default() -> Self { Self::new() }
}

impl BookingEngine {
    /// Creates an empty world and registers the built-in prototype blocks.
    pub fn new() -> Self {
        let mut blocks = BlockRegistry::new();
        for (id, name, tile) in [
            (1, "book:grass_block", AtlasTile { x: 0, y: 0 }),
            (2, "book:dirt", AtlasTile { x: 1, y: 0 }),
            (3, "book:stone", AtlasTile { x: 2, y: 0 }),
        ] {
            blocks.register(BlockDefinition {
                id: BlockId(id), name: name.to_owned(), atlas_tile: tile,
                solid: true, opaque: true,
            });
        }
        Self { blocks, world: ChunkManager::new() }
    }

    pub fn block_registry(&self) -> &BlockRegistry { &self.blocks }
    pub fn block_registry_mut(&mut self) -> &mut BlockRegistry { &mut self.blocks }
    pub fn world(&self) -> &ChunkManager { &self.world }
    pub fn world_mut(&mut self) -> &mut ChunkManager { &mut self.world }

    /// Generates deterministic prototype terrain around the origin.
    /// A radius of 1 creates a 3x3 chunk area. This is not yet the Bedrock
    /// compatible noise, biome, or structure generator.
    pub fn generate_prototype_terrain(&mut self, chunk_radius: i32) {
        let radius = chunk_radius.max(0);
        for chunk_z in -radius..=radius {
            for chunk_x in -radius..=radius {
                let position = ChunkPos { x: chunk_x, z: chunk_z };
                let mut column = ChunkColumn::new(position);
                let section = column.section_mut(0);

                for local_z in 0..SECTION_EDGE {
                    for local_x in 0..SECTION_EDGE {
                        let world_x = chunk_x * SECTION_EDGE as i32 + local_x as i32;
                        let world_z = chunk_z * SECTION_EDGE as i32 + local_z as i32;
                        let wave = ((world_x as f32 * 0.12).sin()
                            + (world_z as f32 * 0.11).cos()) * 1.35;
                        let distance = ((world_x as f32).powi(2)
                            + (world_z as f32).powi(2)).sqrt();
                        let ridge = distance * -0.045 + 1.8;
                        let height = (3.0 + wave + ridge).round().clamp(1.0, 8.0) as usize;

                        for y in 0..=height {
                            let block = if y == height { BlockId(1) }
                                else if y + 2 >= height { BlockId(2) }
                                else { BlockId(3) };
                            section.set(local_x, y, local_z, block);
                        }
                    }
                }
                self.world.insert_column(column);
            }
        }
    }

    /// Builds CPU meshes for all loaded section-zero chunks.
    /// Loaded neighbors are used to cull hidden faces at chunk boundaries.
    pub fn build_loaded_world_meshes(&self) -> Vec<WorldSectionMesh> {
        self.world.iter().filter_map(|(position, _)| {
            mesh_chunk_section(&self.world, *position, 0, &self.blocks, 16, 16)
                .map(|mesh| WorldSectionMesh { position: *position, section_y: 0, mesh })
        }).collect()
    }

    pub fn loaded_chunk_count(&self) -> usize { self.world.loaded_count() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prototype_world_generates_expected_chunk_area() {
        let mut engine = BookingEngine::new();
        engine.generate_prototype_terrain(1);
        assert_eq!(engine.loaded_chunk_count(), 9);
        assert!(engine.world().get_block(0, 0, 0).is_some());
    }

    #[test]
    fn prototype_world_builds_non_empty_meshes() {
        let mut engine = BookingEngine::new();
        engine.generate_prototype_terrain(1);
        let meshes = engine.build_loaded_world_meshes();
        assert_eq!(meshes.len(), 9);
        assert!(meshes.iter().all(|section| !section.mesh.is_empty()));
    }

    #[test]
    fn prototype_world_supports_negative_chunk_coordinates() {
        let mut engine = BookingEngine::new();
        engine.generate_prototype_terrain(1);
        assert!(engine.world().column(ChunkPos { x: -1, z: -1 }).is_some());
    }
}
