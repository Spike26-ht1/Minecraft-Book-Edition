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

#[cfg(test)]
mod tests {
    use super::*;

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
