# Minecraft Book Edition 26.60

An independent, open-source voxel sandbox project written in Rust.

The goal is a lightweight, modular game with a native engine and a community-led compatibility path for selected Java content mods and Bedrock add-ons/resource packs.

## 26.60 status

This is the first development foundation and OpenGL renderer prototype:

- Rust workspace split into focused crates.
- Shared block identifiers and block registry.
- 16 × 16 × 16 chunk-section storage.
- Bedrock-style terrain_texture.json metadata reader.
- CPU chunk meshing with hidden-face removal.
- Backend-independent renderer contract.
- OpenGL window and animated cube smoke test.

**This is not yet a playable Minecraft-compatible game.** It does not yet load a complete Bedrock resource pack, generate or save worlds, connect to servers, or run Java mods. The cube is a renderer test, not the final chunk renderer.

## Build

Install stable Rust and a desktop OpenGL driver.

    cargo build --workspace
    cargo run -p book-client
    cargo test --workspace
    cargo fmt --all -- --check

## Platform policy

The data crates are platform-independent. x86 and x86_64 are project targets, but each renderer/windowing dependency must be verified for the exact target and operating system. This first OpenGL prototype is a desktop milestone; Android and 32-bit release packages are not ready yet.

## Compatibility policy

Compatibility will be added format by format and mod by mod. Java mods that depend on Java-specific renderer internals, mixins, or engine replacement are not initial targets. Bedrock atlas metadata support is being developed independently of proprietary game code and assets.

Read CONTRIBUTING.md and docs/ARCHITECTURE.md before contributing. The project uses the MIT License; third-party dependencies retain their own licenses.
