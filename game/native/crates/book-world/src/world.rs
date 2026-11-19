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
        assert!(
            vertical_sections > 0,
            "world needs at least one vertical section"
        );

        let generator = TerrainGenerator::new(seed);
        let mut columns = HashMap::new();

        for chunk_x in -horizontal_radius..=horizontal_radius {
            for chunk_z in -horizontal_radius..=horizontal_radius {
                let sections = (0..vertical_sections)
                    .map(|section_y| generator.generate_section(chunk_x, section_y, chunk_z))
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

    /// Encodes the complete loaded world into the versioned local save format.
    /// Sorting columns makes identical worlds produce byte-identical files.
    pub fn encode(&self) -> Vec<u8> {
        let mut positions = self.columns.keys().copied().collect::<Vec<_>>();
        positions.sort_unstable_by_key(|position| (position.x, position.z));

        let mut bytes = Vec::new();
        bytes.extend_from_slice(WORLD_SAVE_MAGIC);
        bytes.extend_from_slice(&WORLD_SAVE_FORMAT.to_le_bytes());
        bytes.extend_from_slice(&self.generator_version.to_le_bytes());
        bytes.extend_from_slice(&self.seed.to_le_bytes());
        bytes.extend_from_slice(&(positions.len() as u32).to_le_bytes());

        for position in positions {
            let column = self.columns.get(&position).expect("position came from map");
            bytes.extend_from_slice(&position.x.to_le_bytes());
            bytes.extend_from_slice(&position.z.to_le_bytes());
            bytes.extend_from_slice(&(column.sections.len() as u16).to_le_bytes());

            for section in &column.sections {
                for block in section.blocks.iter() {
                    bytes.extend_from_slice(&block.0.to_le_bytes());
                }
            }
        }

        bytes
    }

    /// Decodes a save created by a supported Book Edition world generator.
    /// Lengths and counts are validated before chunk data is allocated.
    pub fn decode(bytes: &[u8]) -> Result<Self, WorldSaveError> {
        if bytes.len() > MAX_WORLD_SAVE_BYTES {
            return Err(WorldSaveError::LimitExceeded);
        }

        let mut reader = WorldSaveReader::new(bytes);
        if reader.take(WORLD_SAVE_MAGIC.len())? != WORLD_SAVE_MAGIC {
            return Err(WorldSaveError::InvalidMagic);
        }

        let format = reader.u32()?;
        if format != WORLD_SAVE_FORMAT {
            return Err(WorldSaveError::UnsupportedFormat(format));
        }

        let generator_version = reader.u32()?;
        if generator_version != TERRAIN_GENERATOR_VERSION {
            return Err(WorldSaveError::UnsupportedGenerator(generator_version));
        }

        let seed = reader.u64()?;
        let column_count = reader.u32()?;
        if column_count == 0 || column_count > MAX_SAVE_COLUMNS {
            return Err(WorldSaveError::LimitExceeded);
        }

        let mut columns = HashMap::with_capacity(column_count as usize);
        for _ in 0..column_count {
            let position = ChunkPosition {
                x: reader.i32()?,
                z: reader.i32()?,
            };
            let section_count = reader.u16()?;
            if section_count == 0 || section_count > MAX_SAVE_SECTIONS {
                return Err(WorldSaveError::LimitExceeded);
            }

            let mut sections = Vec::with_capacity(section_count as usize);
            for _ in 0..section_count {
                let mut section = Chunk::new();
                for block in section.blocks.iter_mut() {
                    *block = BlockId(reader.u16()?);
                }
                sections.push(section);
            }

            if columns.insert(position, ChunkColumn { sections }).is_some() {
                return Err(WorldSaveError::DuplicateColumn);
            }
        }

        if !reader.is_finished() {
            return Err(WorldSaveError::TrailingData);
        }

        Ok(Self {
            seed,
            generator_version,
            generator: TerrainGenerator::new(seed),
            columns,
        })
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
    pub fn set_block(&mut self, world_x: i32, world_y: i32, world_z: i32, block: BlockId) -> bool {
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
            .map(|(position, column)| (position.x, position.z, column.sections.clone()))
            .collect::<Vec<_>>();
        columns.sort_unstable_by_key(|(x, z, _)| (*x, *z));
        columns
    }
}

const WORLD_SAVE_MAGIC: &[u8; 8] = b"BOOKWLD1";
const WORLD_SAVE_FORMAT: u32 = 1;
const MAX_SAVE_COLUMNS: u32 = 1024;
const MAX_SAVE_SECTIONS: u16 = 32;
const MAX_WORLD_SAVE_BYTES: usize = 64 * 1024 * 1024;

/// Errors returned while decoding a local Book Edition world file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldSaveError {
    InvalidMagic,
    UnsupportedFormat(u32),
    UnsupportedGenerator(u32),
    LimitExceeded,
    Truncated,
    TrailingData,
    DuplicateColumn,
}

impl std::fmt::Display for WorldSaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMagic => f.write_str("invalid Book Edition world file"),
            Self::UnsupportedFormat(version) => write!(f, "unsupported world format {version}"),
            Self::UnsupportedGenerator(version) => {
                write!(f, "unsupported terrain generator version {version}")
            }
            Self::LimitExceeded => f.write_str("world file exceeds safety limits"),
            Self::Truncated => f.write_str("world file is truncated"),
            Self::TrailingData => f.write_str("world file contains unexpected trailing data"),
            Self::DuplicateColumn => f.write_str("world file contains duplicate chunk columns"),
        }
    }
}

impl std::error::Error for WorldSaveError {}

struct WorldSaveReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> WorldSaveReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], WorldSaveError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(WorldSaveError::LimitExceeded)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(WorldSaveError::Truncated)?;
        self.offset = end;
        Ok(value)
    }

    fn u16(&mut self) -> Result<u16, WorldSaveError> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().expect("two-byte slice"),
        ))
    }

    fn u32(&mut self) -> Result<u32, WorldSaveError> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("four-byte slice"),
        ))
    }

    fn i32(&mut self) -> Result<i32, WorldSaveError> {
        Ok(i32::from_le_bytes(
            self.take(4)?.try_into().expect("four-byte slice"),
        ))
    }

    fn u64(&mut self) -> Result<u64, WorldSaveError> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("eight-byte slice"),
        ))
    }

    fn is_finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_save_round_trips_seed_negative_chunks_and_block_edits() {
        let mut original = World::generate(0xB00C_ED17, 1, 4);
        assert!(original.set_block(-1, 23, 16, BlockId::GRASS));
        assert!(original.set_block(16, 31, -1, BlockId::SAND));

        let encoded = original.encode();
        let restored = World::decode(&encoded).unwrap();

        assert_eq!(restored.seed(), original.seed());
        assert_eq!(restored.generator_version(), original.generator_version());
        assert_eq!(
            restored.loaded_column_count(),
            original.loaded_column_count()
        );
        assert_eq!(restored.block_at(-1, 23, 16), BlockId::GRASS);
        assert_eq!(restored.block_at(16, 31, -1), BlockId::SAND);
        assert_eq!(restored.block_at(0, 0, 0), original.block_at(0, 0, 0));
        assert_eq!(restored.encode(), encoded);
    }

    #[test]
    fn binary_save_rejects_truncated_data() {
        let world = World::generate(7, 1, 4);
        let mut encoded = world.encode();
        encoded.truncate(encoded.len() - 1);
        assert!(matches!(
            World::decode(&encoded),
            Err(WorldSaveError::Truncated)
        ));
    }

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

        assert!(world.set_block(15, 30, 0, BlockId::STONE));
        assert!(world.set_block(16, 30, 0, BlockId::DIRT));
        assert!(world.set_block(-1, 30, 0, BlockId::SAND));

        assert_eq!(world.block_at(15, 30, 0), BlockId::STONE);
        assert_eq!(world.block_at(16, 30, 0), BlockId::DIRT);
        assert_eq!(world.block_at(-1, 30, 0), BlockId::SAND);
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
