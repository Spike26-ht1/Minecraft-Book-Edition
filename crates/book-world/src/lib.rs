use book_core::{BlockId, BlockState};

pub const SECTION_EDGE: usize = 16;
pub const SECTION_VOLUME: usize = SECTION_EDGE * SECTION_EDGE * SECTION_EDGE;

/// Integer coordinates identifying a chunk column in the world.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

/// Palette-compressed storage for a fixed number of block states.
///
/// Entries use a compact palette index and a variable bit width. This is a
/// Booking storage foundation; exact Bedrock network/chunk serialization is
/// implemented separately.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PalettedBlockStorage {
    capacity: usize,
    bits_per_entry: u8,
    palette: Vec<BlockState>,
    palette_indices: std::collections::BTreeMap<BlockState, u16>,
    data: Vec<u64>,
}

/// Safe, validated export of palette-compressed section storage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PalettedBlockStorageSnapshot {
    pub capacity: usize,
    pub bits_per_entry: u8,
    pub palette: Vec<BlockState>,
    pub data: Vec<u64>,
}

impl PalettedBlockStorage {
    pub fn new(capacity: usize) -> Self {
        let mut palette_indices = std::collections::BTreeMap::new();
        palette_indices.insert(BlockState::AIR, 0);
        let bits_per_entry = 4;
        let word_count = (capacity * bits_per_entry as usize).div_ceil(64);
        Self {
            capacity,
            bits_per_entry,
            palette: vec![BlockState::AIR],
            palette_indices,
            data: vec![0; word_count],
        }
    }

    pub fn len(&self) -> usize { self.capacity }
    pub fn is_empty(&self) -> bool { self.capacity == 0 }
    pub fn bits_per_entry(&self) -> u8 { self.bits_per_entry }
    pub fn palette(&self) -> &[BlockState] { &self.palette }

    pub fn snapshot(&self) -> PalettedBlockStorageSnapshot {
        PalettedBlockStorageSnapshot {
            capacity: self.capacity,
            bits_per_entry: self.bits_per_entry,
            palette: self.palette.clone(),
            data: self.data.clone(),
        }
    }

    /// Reconstructs packed storage only after validating all dimensions,
    /// palette indices, and word counts from untrusted save data.
    pub fn from_snapshot(snapshot: PalettedBlockStorageSnapshot) -> Option<Self> {
        if snapshot.bits_per_entry < 4 || snapshot.bits_per_entry > 16
            || snapshot.palette.is_empty()
            || snapshot.palette.len() > (1_usize << snapshot.bits_per_entry)
            || snapshot.palette.first().copied() != Some(BlockState::AIR)
        {
            return None;
        }

        let total_bits = snapshot.capacity.checked_mul(snapshot.bits_per_entry as usize)?;
        let rounded_bits = total_bits.checked_add(63)?;
        let expected_words = rounded_bits / 64;
        if snapshot.data.len() != expected_words {
            return None;
        }

        let mut palette_indices = std::collections::BTreeMap::new();
        for (index, state) in snapshot.palette.iter().copied().enumerate() {
            let index = u16::try_from(index).ok()?;
            if palette_indices.insert(state, index).is_some() {
                return None;
            }
        }

        let storage = Self {
            capacity: snapshot.capacity,
            bits_per_entry: snapshot.bits_per_entry,
            palette: snapshot.palette,
            palette_indices,
            data: snapshot.data,
        };

        if (0..storage.capacity).any(|index| {
            storage.read_index(index) as usize >= storage.palette.len()
        }) {
            return None;
        }
        Some(storage)
    }

    pub fn get(&self, index: usize) -> Option<BlockState> {
        if index >= self.capacity { return None; }
        let palette_index = self.read_index(index) as usize;
        self.palette.get(palette_index).copied()
    }

    pub fn set(&mut self, index: usize, state: BlockState) -> bool {
        if index >= self.capacity { return false; }
        let palette_index = if let Some(index) = self.palette_indices.get(&state) {
            *index
        } else {
            let next = self.palette.len() as u16;
            self.palette.push(state);
            self.palette_indices.insert(state, next);
            self.ensure_palette_capacity();
            next
        };
        self.write_index(index, palette_index);
        true
    }

    pub fn iter_states(&self) -> impl Iterator<Item = BlockState> + '_ {
        (0..self.capacity).filter_map(|index| self.get(index))
    }

    fn ensure_palette_capacity(&mut self) {
        let required = (usize::BITS - (self.palette.len() - 1).leading_zeros()) as u8;
        let new_bits = required.max(4);
        if new_bits <= self.bits_per_entry { return; }

        let old_bits = self.bits_per_entry;
        let old_indices: Vec<u16> = (0..self.capacity)
            .map(|index| self.read_index_with_bits(index, old_bits))
            .collect();
        self.bits_per_entry = new_bits;
        self.data = vec![0; (self.capacity * new_bits as usize).div_ceil(64)];
        for (index, palette_index) in old_indices.into_iter().enumerate() {
            self.write_index(index, palette_index);
        }
    }

    fn read_index(&self, index: usize) -> u16 {
        self.read_index_with_bits(index, self.bits_per_entry)
    }

    fn read_index_with_bits(&self, index: usize, bits: u8) -> u16 {
        let bit_index = index * bits as usize;
        let word_index = bit_index / 64;
        let shift = (bit_index % 64) as u32;
        let mask = (1_u64 << bits) - 1;
        let mut value = self.data[word_index] >> shift;
        if shift + bits as u32 > 64 {
            value |= self.data[word_index + 1] << (64 - shift);
        }
        (value & mask) as u16
    }

    fn write_index(&mut self, index: usize, value: u16) {
        let bits = self.bits_per_entry;
        let bit_index = index * bits as usize;
        let word_index = bit_index / 64;
        let shift = (bit_index % 64) as u32;
        let mask = (1_u64 << bits) - 1;
        let value = value as u64 & mask;

        self.data[word_index] =
            (self.data[word_index] & !(mask << shift)) | (value << shift);

        if shift + bits as u32 > 64 {
            let spill_bits = shift + bits as u32 - 64;
            let spill_mask = (1_u64 << spill_bits) - 1;
            self.data[word_index + 1] =
                (self.data[word_index + 1] & !spill_mask)
                | ((value >> (bits as u32 - spill_bits)) & spill_mask);
        }
    }
}

/// A 16 × 16 × 16 section of block states.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChunkSection {
    storage: PalettedBlockStorage,
}

impl Default for ChunkSection {
    fn default() -> Self {
        Self { storage: PalettedBlockStorage::new(SECTION_VOLUME) }
    }
}

impl ChunkSection {
    pub fn new() -> Self { Self::default() }

    pub fn snapshot(&self) -> PalettedBlockStorageSnapshot {
        self.storage.snapshot()
    }

    pub fn from_snapshot(snapshot: PalettedBlockStorageSnapshot) -> Option<Self> {
        if snapshot.capacity != SECTION_VOLUME {
            return None;
        }
        Some(Self {
            storage: PalettedBlockStorage::from_snapshot(snapshot)?,
        })
    }

    /// Returns the linear array index for local section coordinates.
    pub fn index(x: usize, y: usize, z: usize) -> Option<usize> {
        if x >= SECTION_EDGE || y >= SECTION_EDGE || z >= SECTION_EDGE {
            return None;
        }
        Some(x + SECTION_EDGE * (z + SECTION_EDGE * y))
    }

    pub fn get_state(&self, x: usize, y: usize, z: usize) -> Option<BlockState> {
        self.storage.get(Self::index(x, y, z)?)
    }

    pub fn set_state(&mut self, x: usize, y: usize, z: usize, state: BlockState) -> bool {
        let Some(index) = Self::index(x, y, z) else { return false; };
        self.storage.set(index, state)
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<BlockId> {
        self.get_state(x, y, z).map(|state| state.block)
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockId) -> bool {
        self.set_state(x, y, z, BlockState::new(block, 0))
    }

    pub fn blocks(&self) -> impl Iterator<Item = BlockState> + '_ {
        self.storage.iter_states()
    }

    pub fn palette(&self) -> &[BlockState] { self.storage.palette() }
    pub fn bits_per_entry(&self) -> u8 { self.storage.bits_per_entry() }
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

    pub fn insert_section(&mut self, section_y: i32, section: ChunkSection) -> Option<ChunkSection> {
        self.sections.insert(section_y, section)
    }

    /// Iterates loaded vertical sections in ascending world-height order.
    pub fn iter_sections(&self) -> impl Iterator<Item = (i32, &ChunkSection)> {
        self.sections.iter().map(|(section_y, section)| (*section_y, section))
    }

    /// Reads a block state using local X/Z coordinates and absolute world Y.
    pub fn get_state(&self, local_x: usize, world_y: i32, local_z: usize) -> Option<BlockState> {
        if local_x >= SECTION_EDGE || local_z >= SECTION_EDGE {
            return None;
        }

        let section_y = world_y.div_euclid(SECTION_EDGE as i32);
        let local_y = world_y.rem_euclid(SECTION_EDGE as i32) as usize;
        self.sections.get(&section_y)?.get_state(local_x, local_y, local_z)
    }

    /// Reads only the block type at an absolute world Y coordinate.
    pub fn get_block(&self, local_x: usize, world_y: i32, local_z: usize) -> Option<BlockId> {
        self.get_state(local_x, world_y, local_z).map(|state| state.block)
    }

    /// Writes a block state using local X/Z coordinates and absolute world Y.
    pub fn set_state(
        &mut self,
        local_x: usize,
        world_y: i32,
        local_z: usize,
        state: BlockState,
    ) -> bool {
        if local_x >= SECTION_EDGE || local_z >= SECTION_EDGE {
            return false;
        }

        let section_y = world_y.div_euclid(SECTION_EDGE as i32);
        let local_y = world_y.rem_euclid(SECTION_EDGE as i32) as usize;
        self.section_mut(section_y).set_state(local_x, local_y, local_z, state)
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
        self.set_state(local_x, world_y, local_z, BlockState::new(block, 0))
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
    fn palette_storage_round_trips_values_across_word_boundaries() {
        let mut storage = PalettedBlockStorage::new(64);
        for index in 0..64 {
            let state = BlockState::new(BlockId((index % 19 + 1) as u16), (index % 7) as u16);
            assert!(storage.set(index, state));
        }
        for index in 0..64 {
            let expected = BlockState::new(
                BlockId((index % 19 + 1) as u16),
                (index % 7) as u16,
            );
            assert_eq!(storage.get(index), Some(expected));
        }
        assert!(storage.bits_per_entry() >= 5);
    }

    #[test]
    fn palette_storage_rejects_out_of_bounds_indices() {
        let mut storage = PalettedBlockStorage::new(4);
        assert!(!storage.set(4, BlockState::new(BlockId(1), 0)));
        assert_eq!(storage.get(4), None);
    }

    #[test]
    fn new_section_is_filled_with_air() {
        let section = ChunkSection::new();

        assert!(section.blocks().all(|state| state == BlockState::AIR));
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

    pub fn clear(&mut self) {
        self.columns.clear();
    }

    pub fn is_loaded(&self, position: ChunkPos) -> bool {
        self.columns.contains_key(&position)
    }

    /// Reads an absolute world block state.
    /// Returns None if its chunk or vertical section is not loaded.
    pub fn get_state(&self, world_x: i32, world_y: i32, world_z: i32) -> Option<BlockState> {
        let (position, local_x, local_z) = Self::locate(world_x, world_z);
        self.columns.get(&position)?.get_state(local_x, world_y, local_z)
    }

    /// Reads only the block type at an absolute world position.
    pub fn get_block(&self, world_x: i32, world_y: i32, world_z: i32) -> Option<BlockId> {
        self.get_state(world_x, world_y, world_z).map(|state| state.block)
    }

    /// Writes an absolute world block state.
    pub fn set_state(
        &mut self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        state: BlockState,
    ) -> bool {
        let (position, local_x, local_z) = Self::locate(world_x, world_z);
        let Some(column) = self.columns.get_mut(&position) else {
            return false;
        };
        column.set_state(local_x, world_y, local_z, state)
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
        self.set_state(world_x, world_y, world_z, BlockState::new(block, 0))
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
    fn manager_preserves_block_state_across_chunk_boundaries() {
        let mut manager = ChunkManager::new();
        manager.insert_column(ChunkColumn::new(ChunkPos { x: 1, z: 0 }));
        let state = BlockState::new(BlockId(7), 23);
        assert!(manager.set_state(16, 5, 2, state));
        assert_eq!(manager.get_state(16, 5, 2), Some(state));
        assert_eq!(manager.get_block(16, 5, 2), Some(BlockId(7)));
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
