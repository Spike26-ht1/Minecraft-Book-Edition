//! Minimal voxel storage used by the first Book Edition world prototype.

use book_core::CHUNK_EDGE;

/// Number of block slots in one 16 × 16 × 16 chunk section.
pub const CHUNK_VOLUME: usize =
    (CHUNK_EDGE as usize) * (CHUNK_EDGE as usize) * (CHUNK_EDGE as usize);

/// Stable identifier for a block type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct BlockId(u16);

impl BlockId {
    pub const AIR: Self = Self(0);
    pub const GRASS: Self = Self(1);
    pub const DIRT: Self = Self(2);
    pub const STONE: Self = Self(3);

    pub const fn is_air(self) -> bool {
        self.0 == Self::AIR.0
    }

    /// Base display color for the first untextured renderer.
    pub const fn debug_color(self) -> [f32; 3] {
        match self.0 {
            1 => [0.34, 0.68, 0.29],
            2 => [0.48, 0.32, 0.19],
            3 => [0.48, 0.51, 0.54],
            _ => [0.0, 0.0, 0.0],
        }
    }
}

/// One 16 × 16 × 16 block section.
///
/// This first implementation stores compact block identifiers only. Block
/// properties, palettes and persistence will be added in later milestones.
#[derive(Clone)]
pub struct Chunk {
    blocks: Box<[BlockId; CHUNK_VOLUME]>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            blocks: Box::new([BlockId::AIR; CHUNK_VOLUME]),
        }
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> BlockId {
        let Some(index) = block_index(x, y, z) else {
            return BlockId::AIR;
        };

        self.blocks[index]
    }

    /// Sets a block and returns false when the local coordinate is outside
    /// the chunk section.
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: BlockId) -> bool {
        let Some(index) = block_index(x, y, z) else {
            return false;
        };

        self.blocks[index] = block;
        true
    }

    /// Creates a deterministic, small terrain patch for renderer development.
    /// This is a test fixture, not the final terrain generator.
    pub fn generate_test_terrain() -> Self {
        let mut chunk = Self::new();

        for x in 0..CHUNK_EDGE {
            for z in 0..CHUNK_EDGE {
                let variation = (x * 3 + z * 5 + (x * z) % 7).rem_euclid(3);
                let surface_y = 4 + variation;

                for y in 0..=surface_y {
                    let block = if y == surface_y {
                        BlockId::GRASS
                    } else if y >= surface_y - 3 {
                        BlockId::DIRT
                    } else {
                        BlockId::STONE
                    };

                    let _ = chunk.set_block(x, y, z, block);
                }
            }
        }

        chunk
    }
}

fn block_index(x: i32, y: i32, z: i32) -> Option<usize> {
    if !(0..CHUNK_EDGE).contains(&x)
        || !(0..CHUNK_EDGE).contains(&y)
        || !(0..CHUNK_EDGE).contains(&z)
    {
        return None;
    }

    Some((((y as usize) * CHUNK_EDGE as usize) + z as usize) * CHUNK_EDGE as usize + x as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_chunk_is_completely_empty() {
        let chunk = Chunk::new();
        assert!(chunk.get_block(0, 0, 0).is_air());
        assert!(chunk.get_block(15, 15, 15).is_air());
    }

    #[test]
    fn block_can_be_written_and_read_back() {
        let mut chunk = Chunk::new();

        assert!(chunk.set_block(3, 4, 5, BlockId::STONE));
        assert_eq!(chunk.get_block(3, 4, 5), BlockId::STONE);
    }

    #[test]
    fn out_of_range_coordinates_are_safe() {
        let mut chunk = Chunk::new();

        assert!(!chunk.set_block(-1, 0, 0, BlockId::GRASS));
        assert!(chunk.get_block(16, 0, 0).is_air());
        assert!(chunk.get_block(0, -1, 0).is_air());
        assert!(chunk.get_block(0, 0, 16).is_air());
    }

    #[test]
    fn test_terrain_contains_surface_and_subsurface_blocks() {
        let chunk = Chunk::generate_test_terrain();

        assert_eq!(chunk.get_block(0, 4, 0), BlockId::GRASS);
        assert_eq!(chunk.get_block(0, 3, 0), BlockId::DIRT);
        assert_eq!(chunk.get_block(0, 0, 0), BlockId::STONE);
    }
}
