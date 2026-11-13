use book_core::BlockId;

pub const SECTION_EDGE: usize = 16;
pub const SECTION_VOLUME: usize = SECTION_EDGE * SECTION_EDGE * SECTION_EDGE;

/// Integer coordinates identifying a chunk column in the world.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

/// A 16 × 16 × 16 section of blocks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChunkSection {
    blocks: [BlockId; SECTION_VOLUME],
}

impl Default for ChunkSection {
    fn default() -> Self {
        Self {
            blocks: [BlockId::AIR; SECTION_VOLUME],
        }
    }
}

impl ChunkSection {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the linear array index for local section coordinates.
    pub fn index(x: usize, y: usize, z: usize) -> Option<usize> {
        if x >= SECTION_EDGE || y >= SECTION_EDGE || z >= SECTION_EDGE {
            return None;
        }

        Some(x + SECTION_EDGE * (z + SECTION_EDGE * y))
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<BlockId> {
        let index = Self::index(x, y, z)?;
        Some(self.blocks[index])
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockId) -> bool {
        let Some(index) = Self::index(x, y, z) else {
            return false;
        };

        self.blocks[index] = block;
        true
    }

    pub fn blocks(&self) -> &[BlockId; SECTION_VOLUME] {
        &self.blocks
    }
}


/// A vertical column of 16 × 16 sections at one horizontal chunk position.
/// Section coordinates use Euclidean division, so negative world heights map
/// to the correct section and local block coordinate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChunkColumn {
    pub position: ChunkPos,
    sections: std::collections::BTreeMap<i32, ChunkSection>,
}

impl ChunkColumn {
    pub fn new(position: ChunkPos) -> Self {
        Self {
            position,
            sections: std::collections::BTreeMap::new(),
        }
    }

    pub fn section(&self, section_y: i32) -> Option<&ChunkSection> {
        self.sections.get(&section_y)
    }

    pub fn section_mut(&mut self, section_y: i32) -> &mut ChunkSection {
        self.sections.entry(section_y).or_default()
    }

    pub fn section_count(&self) -> usize {
        self.sections.len()
    }

    /// Reads a block using local X/Z coordinates and absolute world Y.
    pub fn get_block(&self, local_x: usize, world_y: i32, local_z: usize) -> Option<BlockId> {
        if local_x >= SECTION_EDGE || local_z >= SECTION_EDGE {
            return None;
        }

        let section_y = world_y.div_euclid(SECTION_EDGE as i32);
        let local_y = world_y.rem_euclid(SECTION_EDGE as i32) as usize;
        self.sections.get(&section_y)?.get(local_x, local_y, local_z)
    }

    /// Writes a block using local X/Z coordinates and absolute world Y.
    /// Returns false when X or Z is outside this column.
    pub fn set_block(
        &mut self,
        local_x: usize,
        world_y: i32,
        local_z: usize,
        block: BlockId,
    ) -> bool {
        if local_x >= SECTION_EDGE || local_z >= SECTION_EDGE {
            return false;
        }

        let section_y = world_y.div_euclid(SECTION_EDGE as i32);
        let local_y = world_y.rem_euclid(SECTION_EDGE as i32) as usize;
        self.section_mut(section_y).set(local_x, local_y, local_z, block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn column_stores_blocks_across_vertical_sections() {
        let mut column = ChunkColumn::new(ChunkPos { x: 2, z: -3 });
        assert!(column.set_block(4, 17, 5, BlockId(9)));
        assert_eq!(column.get_block(4, 17, 5), Some(BlockId(9)));
        assert_eq!(column.section_count(), 1);
        assert!(column.section(1).is_some());
    }

    #[test]
    fn column_maps_negative_world_heights_correctly() {
        let mut column = ChunkColumn::new(ChunkPos::default());
        assert!(column.set_block(1, -1, 2, BlockId(4)));
        assert!(column.set_block(1, -16, 2, BlockId(5)));
        assert_eq!(column.get_block(1, -1, 2), Some(BlockId(4)));
        assert_eq!(column.get_block(1, -16, 2), Some(BlockId(5)));
        assert!(column.section(-1).is_some());
    }

    #[test]
    fn column_rejects_horizontal_coordinates_outside_chunk() {
        let mut column = ChunkColumn::new(ChunkPos::default());
        assert!(!column.set_block(16, 0, 0, BlockId(1)));
        assert_eq!(column.get_block(0, 0, 16), None);
        assert_eq!(column.section_count(), 0);
    }

    #[test]
    fn new_section_is_filled_with_air() {
        let section = ChunkSection::new();

        assert!(section.blocks().iter().all(|block| *block == BlockId::AIR));
    }

    #[test]
    fn stores_and_reads_a_block() {
        let mut section = ChunkSection::new();

        assert!(section.set(2, 3, 4, BlockId(7)));
        assert_eq!(section.get(2, 3, 4), Some(BlockId(7)));
    }

    #[test]
    fn rejects_out_of_bounds_coordinates() {
        let mut section = ChunkSection::new();

        assert!(!section.set(16, 0, 0, BlockId(1)));
        assert_eq!(section.get(16, 0, 0), None);
    }
}
