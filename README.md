# Minecraft Book Edition 26.60

An independent, open-source voxel sandbox project written in Rust.

## First test-build target

The first public test build is **64-bit x86_64 for desktop PCs**:
- Windows x86_64.
- Linux x86_64.

32-bit x86 and Android are outside the first test-build target and are not advertised as supported by this build.

## Current development foundation

- Modular Rust workspace.
- Shared block identifiers and block registry.
- 16 × 16 × 16 chunk-section storage.
- Bedrock-style terrain_texture.json metadata reader.
- CPU chunk meshing with hidden-face removal.
- Backend-independent renderer contract.
- Initial OpenGL desktop renderer.
- Test terrain generated through the CPU chunk mesher.
- F3 debug overlay with FPS, frame time, renderer/API, and test-world counters.

**This is not yet a playable Minecraft-compatible game.** The terrain is still test data. The renderer does not yet load the real Bedrock atlas PNG, and the project does not yet have player movement, world generation, saving, networking, or complete resource-pack/add-on support.

## Debug controls

- F3: show or hide the debug overlay.
- FPS is calculated from rendered frames.
- Frame time is shown in milliseconds.
- Chunk, block, and vertex values are explicitly marked as test data until connected to the real world.

## Build

Install stable Rust and a desktop OpenGL driver.

    cargo build --workspace
    cargo run -p book-client
    cargo test --workspace
    cargo fmt --all -- --check

The client currently requires an x86_64 64-bit target.

## Compatibility policy

Compatibility will be added format by format and mod by mod. Java mods that depend on Java-specific renderer internals, mixins, or engine replacement are not initial targets. Bedrock atlas metadata support is being developed independently of proprietary game code and assets.

The project uses the MIT License; third-party dependencies retain their own licenses.
