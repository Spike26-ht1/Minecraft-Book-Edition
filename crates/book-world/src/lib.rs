use book_core::BlockId;

pub const SECTION_EDGE: usize = 16;
pub const SECTION_VOLUME: usize = SECTION_EDGE * SECTION_EDGE * SECTION_EDGE;

/// Integer coordinates identifying a chunk column in the world.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
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


/// Manages the set of currently loaded chunk columns.
///
/// World X/Z coordinates are mapped with Euclidean division, which keeps
/// local coordinates in the 0..16 range even for negative positions.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ChunkManager {
    columns: std::collections::BTreeMap<ChunkPos, ChunkColumn>,
}

impl ChunkManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Converts absolute block coordinates to a chunk position and local X/Z.
    pub fn locate(world_x: i32, world_z: i32) -> (ChunkPos, usize, usize) {
        let edge = SECTION_EDGE as i32;
        let chunk_x = world_x.div_euclid(edge);
        let chunk_z = world_z.div_euclid(edge);
        let local_x = world_x.rem_euclid(edge) as usize;
        let local_z = world_z.rem_euclid(edge) as usize;

        (ChunkPos { x: chunk_x, z: chunk_z }, local_x, local_z)
    }

    /// Inserts a loaded column, returning the previous column at that position.
    pub fn insert_column(&mut self, column: ChunkColumn) -> Option<ChunkColumn> {
        self.columns.insert(column.position, column)
    }

    pub fn column(&self, position: ChunkPos) -> Option<&ChunkColumn> {
        self.columns.get(&position)
    }

    pub fn column_mut(&mut self, position: ChunkPos) -> Option<&mut ChunkColumn> {
        self.columns.get_mut(&position)
    }

    pub fn remove_column(&mut self, position: ChunkPos) -> Option<ChunkColumn> {
        self.columns.remove(&position)
    }

    pub fn loaded_count(&self) -> usize {
        self.columns.len()
    }

    pub fn is_loaded(&self, position: ChunkPos) -> bool {
        self.columns.contains_key(&position)
    }

    /// Reads an absolute world block position.
    /// Returns None if its chunk or vertical section is not loaded.
    pub fn get_block(&self, world_x: i32, world_y: i32, world_z: i32) -> Option<BlockId> {
        let (position, local_x, local_z) = Self::locate(world_x, world_z);
        self.columns.get(&position)?.get_block(local_x, world_y, local_z)
    }

    /// Writes an absolute world block position.
    /// Returns false when the corresponding chunk column is not loaded.
    pub fn set_block(
        &mut self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        block: BlockId,
    ) -> bool {
        let (position, local_x, local_z) = Self::locate(world_x, world_z);
        let Some(column) = self.columns.get_mut(&position) else {
            return false;
        };

        column.set_block(local_x, world_y, local_z, block)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ChunkPos, &ChunkColumn)> {
        self.columns.iter()
    }
}


#[cfg(test)]
mod manager_tests {
    use super::*;

    #[test]
    fn locate_maps_positive_and_negative_world_coordinates() {
        assert_eq!(
            ChunkManager::locate(16, 31),
            (ChunkPos { x: 1, z: 1 }, 0, 15)
        );
        assert_eq!(
            ChunkManager::locate(-1, -16),
            (ChunkPos { x: -1, z: -1 }, 15, 0)
        );
        assert_eq!(
            ChunkManager::locate(-17, 16),
            (ChunkPos { x: -2, z: 1 }, 15, 0)
        );
    }

    #[test]
    fn manager_reads_and_writes_across_chunk_boundaries() {
        let mut manager = ChunkManager::new();
        manager.insert_column(ChunkColumn::new(ChunkPos { x: 0, z: 0 }));
        manager.insert_column(ChunkColumn::new(ChunkPos { x: 1, z: 0 }));
        manager.insert_column(ChunkColumn::new(ChunkPos { x: -1, z: 0 }));

        assert!(manager.set_block(15, 4, 2, BlockId(3)));
        assert!(manager.set_block(16, 4, 2, BlockId(4)));
        assert!(manager.set_block(-1, 4, 2, BlockId(5)));

        assert_eq!(manager.get_block(15, 4, 2), Some(BlockId(3)));
        assert_eq!(manager.get_block(16, 4, 2), Some(BlockId(4)));
        assert_eq!(manager.get_block(-1, 4, 2), Some(BlockId(5)));
    }

    #[test]
    fn manager_does_not_create_missing_columns_implicitly() {
        let mut manager = ChunkManager::new();

        assert_eq!(manager.get_block(0, 0, 0), None);
        assert!(!manager.set_block(0, 0, 0, BlockId(1)));
        assert_eq!(manager.loaded_count(), 0);
    }

    #[test]
    fn manager_replaces_and_removes_columns() {
        let mut manager = ChunkManager::new();
        let position = ChunkPos { x: 2, z: -3 };

        assert!(manager.insert_column(ChunkColumn::new(position)).is_none());
        assert!(manager.insert_column(ChunkColumn::new(position)).is_some());
        assert_eq!(manager.loaded_count(), 1);
        assert!(manager.remove_column(position).is_some());
        assert_eq!(manager.loaded_count(), 0);
    }

    #[test]
    fn manager_iteration_is_in_chunk_position_order() {
        let mut manager = ChunkManager::new();
        manager.insert_column(ChunkColumn::new(ChunkPos { x: 2, z: 0 }));
        manager.insert_column(ChunkColumn::new(ChunkPos { x: -1, z: 0 }));

        let positions: Vec<_> = manager.iter().map(|(position, _)| position.x).collect();
        assert_eq!(positions, vec![-1, 2]);
    }
}
