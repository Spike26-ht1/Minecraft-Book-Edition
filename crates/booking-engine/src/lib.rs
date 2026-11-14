//! Booking is the world and simulation engine for Minecraft Book Edition.
//!
//! This crate owns block registries, loaded world state, terrain creation, and
//! CPU-side chunk mesh preparation. Graphics APIs consume the prepared meshes;
//! they do not own or generate gameplay world data.

mod player;
pub use player::{BlockHit, PlayerController, PlayerInput};

mod save;
pub use save::WorldSaveError;

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

    /// Generates the initial prototype terrain around the origin.
    /// Existing columns are preserved so this method is safe on loaded worlds.
    pub fn generate_prototype_terrain(&mut self, chunk_radius: i32) {
        let radius = chunk_radius.max(0);
        for chunk_z in -radius..=radius {
            for chunk_x in -radius..=radius {
                self.generate_prototype_chunk(chunk_x, chunk_z);
            }
        }
    }

    /// Generates at most one missing chunk column per call, preferring the
    /// closest missing column to the player. The client can call this once per
    /// frame to spread generation and GPU uploads across frames instead of
    /// causing a large pause when crossing a chunk boundary.
    pub fn ensure_next_prototype_chunk_around(
        &mut self,
        world_x: i32,
        world_z: i32,
        chunk_radius: i32,
    ) -> Option<ChunkPos> {
        let (center, _, _) = ChunkManager::locate(world_x, world_z);
        let radius = chunk_radius.clamp(0, 8);
        let mut candidates = Vec::with_capacity(((radius * 2 + 1) as usize).pow(2));

        for offset_z in -radius..=radius {
            for offset_x in -radius..=radius {
                let Some(chunk_x) = center.x.checked_add(offset_x) else { continue; };
                let Some(chunk_z) = center.z.checked_add(offset_z) else { continue; };
                let distance = offset_x.abs() + offset_z.abs();
                candidates.push((distance, offset_x.abs().max(offset_z.abs()), chunk_x, chunk_z));
            }
        }
        candidates.sort_unstable();

        for (_, _, chunk_x, chunk_z) in candidates {
            let position = ChunkPos { x: chunk_x, z: chunk_z };
            if !self.world.is_loaded(position)
                && self.generate_prototype_chunk(chunk_x, chunk_z)
            {
                return Some(position);
            }
        }
        None
    }

    /// Ensures prototype terrain exists around the player's current chunk.
    /// Returns the number of newly generated chunk columns. Chunks are kept
    /// resident for now; disk-backed unloading is a later streaming stage.
    pub fn ensure_prototype_chunks_around(
        &mut self,
        world_x: i32,
        world_z: i32,
        chunk_radius: i32,
    ) -> usize {
        let (center, _, _) = ChunkManager::locate(world_x, world_z);
        let radius = chunk_radius.clamp(0, 8);
        let mut generated = 0;

        for offset_z in -radius..=radius {
            for offset_x in -radius..=radius {
                let Some(chunk_x) = center.x.checked_add(offset_x) else {
                    continue;
                };
                let Some(chunk_z) = center.z.checked_add(offset_z) else {
                    continue;
                };
                if self.generate_prototype_chunk(chunk_x, chunk_z) {
                    generated += 1;
                }
            }
        }
        generated
    }

    fn generate_prototype_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> bool {
        let position = ChunkPos { x: chunk_x, z: chunk_z };
        if self.world.is_loaded(position) {
            return false;
        }

        let Some(origin_x) = chunk_x.checked_mul(SECTION_EDGE as i32) else {
            return false;
        };
        let Some(origin_z) = chunk_z.checked_mul(SECTION_EDGE as i32) else {
            return false;
        };

        const MIN_Y: i32 = -32;
        let mut column = ChunkColumn::new(position);

        for local_z in 0..SECTION_EDGE {
            for local_x in 0..SECTION_EDGE {
                let world_x = origin_x + local_x as i32;
                let world_z = origin_z + local_z as i32;
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
        true
    }

    /// Builds CPU meshes for every loaded vertical section.
    /// Neighbor sections are included so internal section faces are culled.
    pub fn build_loaded_world_meshes(&self) -> Vec<WorldSectionMesh> {
        let positions: std::collections::BTreeSet<(i32, i32)> = self.world
            .iter()
            .map(|(position, _)| (position.x, position.z))
            .collect();
        self.build_world_meshes_for_chunks(&positions)
    }

    /// Builds meshes only for the requested loaded chunk columns. This lets the
    /// mobile renderer upload newly streamed or edited chunks without remeshing
    /// the entire world on the main thread.
    pub fn build_world_meshes_for_chunks(
        &self,
        positions: &std::collections::BTreeSet<(i32, i32)>,
    ) -> Vec<WorldSectionMesh> {
        let mut meshes = Vec::new();
        for &(chunk_x, chunk_z) in positions {
            let position = ChunkPos { x: chunk_x, z: chunk_z };
            let Some(column) = self.world.column(position) else {
                continue;
            };
            for (section_y, _) in column.iter_sections() {
                if let Some(mesh) = mesh_chunk_section(
                    &self.world, position, section_y, &self.blocks, 16, 16,
                ) {
                    meshes.push(WorldSectionMesh {
                        position,
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
        assert!(meshes.iter().any(|section| !section.mesh.is_empty()));
        assert!(meshes.iter().any(|section| section.section_y < 0));
        assert!(meshes.iter().any(|section| section.section_y > 0));
    }

    #[test]
    fn selective_mesh_builder_only_builds_requested_chunk_columns() {
        let mut engine = BookingEngine::new();
        engine.generate_prototype_terrain(2);
        let requested = std::collections::BTreeSet::from([(0, 0), (1, 0)]);
        let meshes = engine.build_world_meshes_for_chunks(&requested);
        assert!(meshes.iter().all(|mesh| {
            (mesh.position.x, mesh.position.z) == (0, 0)
                || (mesh.position.x, mesh.position.z) == (1, 0)
        }));
        assert!(meshes.iter().any(|mesh| mesh.position == ChunkPos { x: 0, z: 0 }));
        assert!(meshes.iter().any(|mesh| mesh.position == ChunkPos { x: 1, z: 0 }));
    }

    #[test]
    fn prototype_world_supports_negative_chunk_coordinates() {
        let mut engine = BookingEngine::new();
        engine.generate_prototype_terrain(1);
        assert!(engine.world().column(ChunkPos { x: -1, z: -1 }).is_some());
    }

    #[test]
    fn streaming_generates_missing_chunks_without_overwriting_edits() {
        let mut engine = BookingEngine::new();
        engine.generate_prototype_terrain(0);
        assert!(engine.world_mut().set_block(3, 30, 3, BlockId(4)));

        let generated = engine.ensure_prototype_chunks_around(16, 0, 1);
        assert_eq!(generated, 8);
        assert_eq!(engine.loaded_chunk_count(), 9);
        assert_eq!(engine.world().get_block(3, 30, 3), Some(BlockId(4)));
        assert!(engine.world().column(ChunkPos { x: 2, z: 0 }).is_some());
    }

    #[test]
    fn incremental_streaming_generates_only_one_chunk_per_call() {
        let mut engine = BookingEngine::new();
        engine.generate_prototype_terrain(0);
        let first = engine.ensure_next_prototype_chunk_around(0, 0, 2);
        assert!(first.is_some());
        assert_eq!(engine.loaded_chunk_count(), 2);
        let second = engine.ensure_next_prototype_chunk_around(0, 0, 2);
        assert!(second.is_some());
        assert_eq!(engine.loaded_chunk_count(), 3);
        assert_ne!(first, second);
    }

    #[test]
    fn streaming_uses_euclidean_chunk_coordinates_for_negative_positions() {
        let mut engine = BookingEngine::new();
        assert_eq!(engine.ensure_prototype_chunks_around(-1, -1, 0), 1);
        assert!(engine.world().column(ChunkPos { x: -1, z: -1 }).is_some());
    }

    #[test]
    fn terrain_generation_does_not_replace_loaded_chunk_data() {
        let mut engine = BookingEngine::new();
        engine.generate_prototype_terrain(0);
        assert!(engine.world_mut().set_block(1, 20, 1, BlockId(4)));
        engine.generate_prototype_terrain(0);
        assert_eq!(engine.world().get_block(1, 20, 1), Some(BlockId(4)));
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
    value = value.wrapping_add(0x9e37_79b9);
    (value as f32 / u32::MAX as f32) * 2.0 - 1.0
}

fn smoothstep(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
