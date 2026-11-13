mod player;
pub use player::{BlockHit, PlayerController, PlayerInput};

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
            (4, "book:bedrock", AtlasTile { x: 2, y: 0 }),
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

    /// Generates deterministic multi-section terrain around the origin.
    ///
    /// This is a temporary Booking terrain generator, not Bedrock's production
    /// noise/biome/structure pipeline. It creates a 96-block vertical range
    /// with a bedrock floor, stone, dirt, and a grass surface.
    pub fn generate_prototype_terrain(&mut self, chunk_radius: i32) {
        let radius = chunk_radius.max(0);
        const MIN_Y: i32 = -32;

        for chunk_z in -radius..=radius {
            for chunk_x in -radius..=radius {
                let position = ChunkPos { x: chunk_x, z: chunk_z };
                let mut column = ChunkColumn::new(position);

                for local_z in 0..SECTION_EDGE {
                    for local_x in 0..SECTION_EDGE {
                        let world_x = chunk_x * SECTION_EDGE as i32 + local_x as i32;
                        let world_z = chunk_z * SECTION_EDGE as i32 + local_z as i32;
                        let broad = value_noise(world_x as f32 / 48.0, world_z as f32 / 48.0);
                        let detail = value_noise(world_x as f32 / 17.0, world_z as f32 / 17.0);
                        let surface_y = (27.0 + broad * 9.0 + detail * 3.0)
                            .round()
                            .clamp(16.0, 40.0) as i32;

                        column.set_block(local_x, MIN_Y, local_z, BlockId(4));
                        for world_y in (MIN_Y + 1)..=surface_y {
                            let block = if world_y == surface_y {
                                BlockId(1)
                            } else if world_y >= surface_y - 3 {
                                BlockId(2)
                            } else {
                                BlockId(3)
                            };
                            column.set_block(local_x, world_y, local_z, block);
                        }
                    }
                }

                self.world.insert_column(column);
            }
        }
    }

    /// Builds CPU meshes for every loaded vertical section.
    /// Neighbor sections are included so internal section faces are culled.
    pub fn build_loaded_world_meshes(&self) -> Vec<WorldSectionMesh> {
        let mut meshes = Vec::new();
        for (position, column) in self.world.iter() {
            for (section_y, _) in column.iter_sections() {
                if let Some(mesh) = mesh_chunk_section(
                    &self.world, *position, section_y, &self.blocks, 16, 16,
                ) {
                    meshes.push(WorldSectionMesh {
                        position: *position,
                        section_y,
                        mesh,
                    });
                }
            }
        }
        meshes
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
        assert!(engine.world().get_block(0, -32, 0).is_some());
        assert!(engine.world().get_block(0, 24, 0).is_some());
        assert!(engine.world().column(ChunkPos { x: 0, z: 0 }).unwrap().section_count() >= 4);
    }

    #[test]
    fn prototype_world_builds_non_empty_meshes() {
        let mut engine = BookingEngine::new();
        engine.generate_prototype_terrain(1);
        let meshes = engine.build_loaded_world_meshes();
        assert!(meshes.len() > 9);
        assert!(meshes.iter().all(|section| !section.mesh.is_empty()));
        assert!(meshes.iter().any(|section| section.section_y < 0));
        assert!(meshes.iter().any(|section| section.section_y > 0));
    }

    #[test]
    fn prototype_world_supports_negative_chunk_coordinates() {
        let mut engine = BookingEngine::new();
        engine.generate_prototype_terrain(1);
        assert!(engine.world().column(ChunkPos { x: -1, z: -1 }).is_some());
    }
}

/// Smooth deterministic value noise used by the prototype terrain generator.
fn value_noise(x: f32, z: f32) -> f32 {
    let x0 = x.floor() as i32;
    let z0 = z.floor() as i32;
    let tx = smoothstep(x - x.floor());
    let tz = smoothstep(z - z.floor());

    let a = lattice_value(x0, z0);
    let b = lattice_value(x0 + 1, z0);
    let c = lattice_value(x0, z0 + 1);
    let d = lattice_value(x0 + 1, z0 + 1);
    let top = lerp(a, b, tx);
    let bottom = lerp(c, d, tx);
    lerp(top, bottom, tz)
}

fn lattice_value(x: i32, z: i32) -> f32 {
    let mut value = (x as u32).wrapping_mul(0x8da6_b343)
        ^ (z as u32).wrapping_mul(0xd816_3841);
    value ^= value >> 13;
    value = value.wrapping_mul(0x85eb_ca6b);
    value ^= value >> 16;
    (value as f32 / u32::MAX as f32) * 2.0 - 1.0
}

fn smoothstep(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
