# Minecraft Book Edition 26.60 — First Pre-Alpha

An independent, open-source voxel sandbox project written in Rust.

## First pre-alpha target

The first public test build targets **Android ARM64 (arm64-v8a)**:
- Android 8.0 or newer (API 26).
- 64-bit ARM devices.
- APK package.
- OpenGL ES renderer.

Desktop builds remain useful as development and CI checks, but they are not the first public test-build target.

## Current development foundation

- Modular Rust workspace.
- Shared block identifiers and block registry.
- 16 × 16 × 16 chunk-section storage.
- Bedrock-style terrain_texture.json metadata reader.
- CPU chunk meshing with hidden-face removal.
- Backend-independent renderer contract.
- Initial OpenGL renderer prototype.
- F3 debug overlay with FPS and frame time.

**This is not yet a playable Minecraft-compatible game.** The terrain is still test data. The renderer does not yet load the real Bedrock atlas PNG, and the project does not yet have player movement, world generation, saving, networking, or complete resource-pack/add-on support.

## Android debug controls

- F3 on a connected physical keyboard toggles the debug overlay.
- The on-screen DBG button in the upper-right corner toggles the overlay by touch.
- FPS and frame time are measured from rendered frames.

## Build checks

Desktop: cargo test --workspace and cargo fmt --all -- --check.

Android ARM64 debug APK: `cargo apk build -p book-client --lib --target aarch64-linux-android`.

The CI APK is a debug-signed test package, not a store-ready release. The Android package uses minimum SDK 26. Installation and runtime behavior still require testing on a physical Android 8+ ARM64 device.

## Compatibility policy

Compatibility will be added format by format and mod by mod. Java mods that depend on Java-specific renderer internals, mixins, or engine replacement are not initial targets. Bedrock atlas metadata support is being developed independently of proprietary game code and assets.

The project uses the MIT License; third-party dependencies retain their own licenses.
