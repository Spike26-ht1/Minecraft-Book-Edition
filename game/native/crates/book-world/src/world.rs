//! In-memory world model shared by simulation, meshing and persistence layers.

use std::collections::HashMap;

use book_core::CHUNK_EDGE;

use crate::{BlockId, Chunk, TerrainGenerator, TERRAIN_GENERATOR_VERSION};

/// Horizontal chunk coordinates. A chunk column contains vertical sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPosition {
    pub x: i32,
    pub z: i32,
}

/// A horizontal column of 16×16×16 sections, ordered from low to high Y.
#[derive(Clone)]
pub struct ChunkColumn {
    sections: Vec<Chunk>,
}

impl ChunkColumn {
    pub fn sections(&self) -> &[Chunk] {
        &self.sections
    }

    pub fn section_count(&self) -> usize {
        self.sections.len()
    }

    fn block(&self, world_y: i32, local_x: i32, local_z: i32) -> BlockId {
        if world_y < 0 {
            return BlockId::AIR;
        }

        let section_y = world_y.div_euclid(CHUNK_EDGE) as usize;
        let local_y = world_y.rem_euclid(CHUNK_EDGE);
        self.sections
            .get(section_y)
            .map(|section| section.get_block(local_x, local_y, local_z))
            .unwrap_or(BlockId::AIR)
    }

    fn set_block(&mut self, world_y: i32, local_x: i32, local_z: i32, block: BlockId) -> bool {
        if world_y < 0 {
            return false;
        }

        let section_y = world_y.div_euclid(CHUNK_EDGE) as usize;
        let local_y = world_y.rem_euclid(CHUNK_EDGE);
        self.sections
            .get_mut(section_y)
            .map(|section| section.set_block(local_x, local_y, local_z, block))
            .unwrap_or(false)
    }
}

/// Loaded world data. Generation is deterministic; simulation owns mutations.
pub struct World {
    seed: u64,
    generator_version: u32,
    generator: TerrainGenerator,
    columns: HashMap<ChunkPosition, ChunkColumn>,
}

impl World {
    /// Generates a square region around chunk (0, 0).
    ///
    /// The radius is measured in chunks on each horizontal side. This is a
    /// first loaded-world implementation; streaming and disk persistence are
    /// separate systems and are deliberately not hidden inside this type.
    pub fn generate(seed: u64, horizontal_radius: i32, vertical_sections: i32) -> Self {
        assert!(horizontal_radius >= 0, "chunk radius cannot be negative");
        assert!(vertical_sections > 0, "world needs at least one vertical section");

        let generator = TerrainGenerator::new(seed);
        let mut columns = HashMap::new();

        for chunk_x in -horizontal_radius..=horizontal_radius {
            for chunk_z in -horizontal_radius..=horizontal_radius {
                let sections = (0..vertical_sections)
                    .map(|section_y| {
                        generator.generate_section(chunk_x, section_y, chunk_z)
                    })
                    .collect();
                columns.insert(
                    ChunkPosition {
                        x: chunk_x,
                        z: chunk_z,
                    },
                    ChunkColumn { sections },
                );
            }
        }

        Self {
            seed,
            generator_version: TERRAIN_GENERATOR_VERSION,
            generator,
            columns,
        }
    }

    pub const fn seed(&self) -> u64 {
        self.seed
    }

    pub const fn generator_version(&self) -> u32 {
        self.generator_version
    }

    pub fn loaded_column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn surface_height(&self, world_x: i32, world_z: i32) -> i32 {
        self.generator.surface_height(world_x, world_z)
    }

    pub fn block_at(&self, world_x: i32, world_y: i32, world_z: i32) -> BlockId {
        let position = ChunkPosition {
            x: world_x.div_euclid(CHUNK_EDGE),
            z: world_z.div_euclid(CHUNK_EDGE),
        };
        self.columns
            .get(&position)
            .map(|column| {
                column.block(
                    world_y,
                    world_x.rem_euclid(CHUNK_EDGE),
                    world_z.rem_euclid(CHUNK_EDGE),
                )
            })
            .unwrap_or(BlockId::AIR)
    }

    /// Changes a block in a loaded column. Returns false outside loaded data.
    pub fn set_block(
        &mut self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        block: BlockId,
    ) -> bool {
        let position = ChunkPosition {
            x: world_x.div_euclid(CHUNK_EDGE),
            z: world_z.div_euclid(CHUNK_EDGE),
        };
        self.columns
            .get_mut(&position)
            .map(|column| {
                column.set_block(
                    world_y,
                    world_x.rem_euclid(CHUNK_EDGE),
                    world_z.rem_euclid(CHUNK_EDGE),
                    block,
                )
            })
            .unwrap_or(false)
    }

    /// Returns owned columns for the CPU mesher. This snapshot prevents the
    /// renderer from borrowing mutable simulation storage during GPU upload.
    pub fn columns_for_meshing(&self) -> Vec<(i32, i32, Vec<Chunk>)> {
        let mut columns = self
            .columns
            .iter()
            .map(|(position, column)| {
                (position.x, position.z, column.sections.clone())
            })
            .collect::<Vec<_>>();
        columns.sort_unstable_by_key(|(x, z, _)| (*x, *z));
        columns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_world_exposes_the_same_seed_and_generator_version() {
        let world = World::generate(20261116, 2, 4);

        assert_eq!(world.seed(), 20261116);
        assert_eq!(world.generator_version(), TERRAIN_GENERATOR_VERSION);
        assert_eq!(world.loaded_column_count(), 25);
    }

    #[test]
    fn block_lookup_crosses_positive_and_negative_chunk_boundaries() {
        let mut world = World::generate(7, 1, 4);

        assert!(world.set_block(15, 90, 0, BlockId::STONE));
        assert!(world.set_block(16, 90, 0, BlockId::DIRT));
        assert!(world.set_block(-1, 90, 0, BlockId::SAND));

        assert_eq!(world.block_at(15, 90, 0), BlockId::STONE);
        assert_eq!(world.block_at(16, 90, 0), BlockId::DIRT);
        assert_eq!(world.block_at(-1, 90, 0), BlockId::SAND);
    }

    #[test]
    fn lookups_outside_loaded_columns_return_air() {
        let world = World::generate(7, 1, 4);

        assert!(world.block_at(48, 30, 0).is_air());
    }

    #[test]
    fn meshing_snapshot_is_stable_and_sorted() {
        let world = World::generate(7, 1, 4);
        let columns = world.columns_for_meshing();

        assert_eq!(columns.len(), 9);
        assert_eq!((columns[0].0, columns[0].1), (-1, -1));
        assert_eq!((columns[8].0, columns[8].1), (1, 1));
    }
}
