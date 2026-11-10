use book_core::{AtlasTile, BlockDefinition, BlockId, BlockRegistry};
use book_world::ChunkSection;

fn main() {
    let mut registry = BlockRegistry::new();
    registry.register(BlockDefinition {
        id: BlockId(1),
        name: "book:stone".to_owned(),
        atlas_tile: AtlasTile { x: 0, y: 0 },
        solid: true,
        opaque: true,
    });

    let mut section = ChunkSection::new();
    section.set(0, 0, 0, BlockId(1));

    println!("Minecraft Book Edition — foundation");
    println!("Registered blocks: {}", registry.len());
    println!("Block at (0, 0, 0): {:?}", section.get(0, 0, 0));
    println!("Renderer: not implemented yet");
}
