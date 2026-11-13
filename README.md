# Minecraft Book Edition — Pre-Alpha 1.2.0

An independent, open-source Bedrock-inspired client foundation written in Rust.

## Project direction

Minecraft Book Edition is intended to become an open, extensible Minecraft Bedrock-style client. Its engine is named **Booking**. The mod ecosystem is being designed around **Book Loader** and a Fabric-inspired public API.

The first test-build target is **Android ARM64 (arm64-v8a)**. Desktop targets remain part of development and CI.

## Current engine foundation

- Modular Rust workspace with separate client, world, assets, meshing, renderer, and mod API crates.
- **Booking Engine** owns block registration, loaded chunks, prototype terrain generation, and CPU-side world mesh preparation.
- 16 × 16 × 16 chunk sections and a world-coordinate chunk manager.
- Neighbor-aware face culling across all six chunk-section directions.
- Per-chunk GPU vertex buffers and distance-based chunk render culling.
- CPU texture-atlas packing and a prototype GPU atlas with nearest-neighbor sampling.
- Face normals and directional lighting.
- Shared camera state, keyboard movement/look, and touch-drag camera look.
- Android OpenGL ES 3.0 / GLSL ES 3.00 backend; desktop OpenGL backend.
- F3 debug overlay with FPS, frame time, visible chunk count, and renderer information.

## Book Loader and mod API

The shared mod manifest supports native Rust, script, and WebAssembly runtime declarations, dependencies, and capability requests. The Rust API now defines lifecycle hooks, event subscriptions, block registration, and world read/write contracts through a restricted host context.

These are API foundations, not a completed runtime. Package discovery, dependency resolution, capability enforcement, script execution, WebAssembly hosting, and a stable native ABI are still under development.

## Current limitations

This is **not yet a playable Bedrock-compatible game**. Terrain is prototype data, and the texture atlas is generated test content rather than a real Bedrock resource-pack atlas. The project still needs resource-pack file loading and PNG decoding, Bedrock block states and palettes, biomes and production terrain generation, lighting propagation, entities, inventory/gameplay systems, world persistence, networking, and full touch movement controls.

Bedrock world, resource-pack, add-on, and server compatibility must be implemented and verified subsystem by subsystem. The project does not reuse proprietary RenderDragon code or game binaries.

## Android test build

- Android 8.0 or newer (API 26).
- ARM64 / arm64-v8a.
- OpenGL ES 3.0.
- F3 on a connected physical keyboard toggles the debug overlay.
- The on-screen DBG control toggles the overlay.
- Dragging on the screen changes the camera view.

The CI APK is a debug test package, not a store-ready release. Successful compilation and APK validation do not replace runtime testing on a physical Android device.

## Build checks

- Desktop: `cargo test --workspace`
- Formatting: `cargo fmt --all -- --check`
- Android ARM64: `cargo apk build -p book-client --lib --target aarch64-linux-android`

The project uses the MIT License; third-party dependencies retain their own licenses.
