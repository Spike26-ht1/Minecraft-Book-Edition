//! Platform-independent primitives for the Book Edition game runtime.

/// Edge length of one horizontal chunk side, in blocks.
pub const CHUNK_EDGE: i32 = 16;

/// Horizontal chunk coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPosition {
    pub x: i32,
    pub z: i32,
}

/// Integer block position in world space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPosition {
    /// Returns the horizontal chunk containing this block.
    ///
    /// Euclidean division is intentional: negative world positions must map
    /// consistently to the chunk on their left/back side.
    pub const fn chunk_position(self) -> ChunkPosition {
        ChunkPosition {
            x: self.x.div_euclid(CHUNK_EDGE),
            z: self.z.div_euclid(CHUNK_EDGE),
        }
    }

    /// Returns the block's local coordinate inside its chunk section.
    /// Every component is in the range 0..16.
    pub const fn local_position(self) -> [u8; 3] {
        [
            self.x.rem_euclid(CHUNK_EDGE) as u8,
            self.y.rem_euclid(CHUNK_EDGE) as u8,
            self.z.rem_euclid(CHUNK_EDGE) as u8,
        ]
    }
}

/// Explicit lifecycle states for a future game session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPhase {
    Booting,
    Ready,
    Running,
    Paused,
    Stopped,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_coordinates_map_to_expected_chunk_and_local_position() {
        let block = BlockPosition { x: 35, y: 7, z: 18 };

        assert_eq!(block.chunk_position(), ChunkPosition { x: 2, z: 1 });
        assert_eq!(block.local_position(), [3, 7, 2]);
    }

    #[test]
    fn negative_coordinates_use_euclidean_chunk_mapping() {
        let block = BlockPosition {
            x: -1,
            y: -1,
            z: -17,
        };

        assert_eq!(block.chunk_position(), ChunkPosition { x: -1, z: -2 });
        assert_eq!(block.local_position(), [15, 15, 15]);
    }

    #[test]
    fn exact_negative_chunk_boundary_maps_without_off_by_one() {
        let block = BlockPosition {
            x: -16,
            y: 0,
            z: -32,
        };

        assert_eq!(block.chunk_position(), ChunkPosition { x: -1, z: -2 });
        assert_eq!(block.local_position(), [0, 0, 0]);
    }
}
