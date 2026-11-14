use std::collections::BTreeMap;

/// Compact identifier stored in chunk data.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct BlockId(pub u16);

impl BlockId {
    pub const AIR: Self = Self(0);
}

/// A block type and its compact interned state identifier.
/// State identifiers are resolved through block definitions, keeping each
/// stored voxel compact while allowing orientation and variant data.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct BlockState {
    pub block: BlockId,
    pub state: u16,
}

impl BlockState {
    pub const AIR: Self = Self { block: BlockId::AIR, state: 0 };

    pub const fn new(block: BlockId, state: u16) -> Self {
        Self { block, state }
    }
}

/// Tile coordinate in a texture atlas, measured in tiles rather than pixels.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AtlasTile {
    pub x: u16,
    pub y: u16,
}

/// Content definition for a block known to the engine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockDefinition {
    pub id: BlockId,
    pub name: String,
    pub atlas_tile: AtlasTile,
    pub solid: bool,
    pub opaque: bool,
}

/// Registry mapping stable block identifiers to their definitions.
#[derive(Clone, Debug, Default)]
pub struct BlockRegistry {
    definitions: BTreeMap<BlockId, BlockDefinition>,
}

impl BlockRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a definition. Returns the previous definition, if the ID existed.
    pub fn register(&mut self, definition: BlockDefinition) -> Option<BlockDefinition> {
        self.definitions.insert(definition.id, definition)
    }

    pub fn get(&self, id: BlockId) -> Option<&BlockDefinition> {
        self.definitions.get(&id)
    }

    pub fn contains(&self, id: BlockId) -> bool {
        self.definitions.contains_key(&id)
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_and_retrieves_a_block() {
        let mut registry = BlockRegistry::new();
        let stone = BlockDefinition {
            id: BlockId(1),
            name: "book:stone".to_owned(),
            atlas_tile: AtlasTile { x: 0, y: 0 },
            solid: true,
            opaque: true,
        };

        registry.register(stone.clone());

        assert_eq!(registry.get(BlockId(1)), Some(&stone));
        assert!(registry.contains(BlockId(1)));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn air_has_the_reserved_zero_identifier() {
        assert_eq!(BlockId::AIR, BlockId(0));
    }
}
