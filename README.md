# Minecraft Book Edition — Pre-Alpha 1.2.0

An independent, open-source Bedrock-inspired client foundation written in Rust.

## Project direction

Minecraft Book Edition is intended to become an open, extensible Minecraft Bedrock-style client. Its engine is named **Booking**. The mod ecosystem is being designed around **Book Loader** and a Fabric-inspired public API.

The first test-build target is **Android ARM64 (arm64-v8a)**. Desktop targets remain part of development and CI.

## Current engine foundation

- Modular Rust workspace with separate client, world, assets, meshing, renderer, and mod API crates.
- **Booking Engine** owns block registration, loaded chunks, prototype terrain generation, player simulation, persistence, and CPU-side world mesh preparation.
- 16 × 16 × 16 chunk sections, a world-coordinate chunk manager, and compact palette-backed block-state storage.
- Multi-section prototype terrain with a bedrock floor, stone/dirt layers, a grass surface, and deterministic smooth height variation.
- Player-centered chunk expansion: new terrain columns are generated as the player crosses chunk boundaries. Loaded chunks are not unloaded yet.
- Neighbor-aware face culling across all six chunk-section directions.
- One merged GPU vertex buffer per chunk column, with distance-based chunk render culling.
- CPU texture-atlas packing and a prototype GPU atlas with nearest-neighbor sampling.
- Face normals and directional lighting.
- First-person player controller with gravity, jumping, sprinting, and axis-separated voxel collision.
- 3D DDA block raycasting, touch-tap/mouse block breaking, and long-press prototype stone placement.
- Versioned binary world saves that preserve compressed section palettes, validate checksums, and autosave after edits and Android lifecycle suspension.
- Separate validated player-state saves for position and view direction, restored only when the saved pose is clear of solid blocks.
- Android keyboard movement/look plus a virtual left analog stick and mobile action buttons for jump, sprint, mine, and place.
- Multi-touch routing keeps joystick, camera-look, and action-button fingers independent.
- Pixel-art stone HUD controls with pressed-state feedback; these are generated Bedrock-inspired placeholders, not Mojang's original UI sprites.
- F3 debug overlay with FPS, frame time, visible chunk count, and renderer information.

## Book Loader and mod API

The shared mod manifest supports native Rust, script, and WebAssembly runtime declarations, dependencies, and capability requests. The Rust API now defines lifecycle hooks, event subscriptions, block registration, and world read/write contracts through a restricted host context.

These are API foundations, not a completed runtime. Package discovery, dependency resolution, capability enforcement, script execution, WebAssembly hosting, and a stable native ABI are still under development.

## Current limitations

This is **not yet a Bedrock-compatible game**. It starts with a 9 × 9 prototype chunk area and expands around the player, but does not unload distant chunks yet. The terrain texture atlas is generated test content, not a real Bedrock resource-pack atlas. The mobile HUD uses a generated Bedrock-inspired stone pixel pattern until the consent-based vanilla asset downloader and official UI texture loader are integrated. The custom Booking save file is not a Bedrock world format. Remaining work includes disk-backed chunk unloading/region storage, production terrain and biome generation, Bedrock block-state property definitions and exact palette serialization, lighting propagation, entity rendering/simulation, inventory and item systems, inventory persistence, networking, and full touch movement controls. The player and block-interaction systems are early foundations, not complete Bedrock gameplay.

Bedrock world, resource-pack, add-on, and server compatibility must be implemented and verified subsystem by subsystem. The project does not reuse proprietary RenderDragon code or game binaries.

## Android test build

- Android 8.0 or newer (API 26).
- ARM64 / arm64-v8a.
- OpenGL ES 3.0.
- F3 on a connected physical keyboard toggles the debug overlay.
- Left analog stick: move and strafe.
- PULO: jump; CORRER: sprint while held.
- MINAR: break the targeted block; COLOCAR: place prototype stone against the targeted face.
- Dragging outside the controls changes the camera view.
- DBG toggles the debug overlay from the upper-right safe area.

The CI APK is a debug test package, not a store-ready release. Successful compilation and APK validation do not replace runtime testing on a physical Android device. World saves are stored under the app's Android/media data directory in the Booking binary format.

## Build checks

- Desktop: `cargo test --workspace`
- Formatting: `cargo fmt --all -- --check`
- Android ARM64: `cargo apk build -p book-client --lib --target aarch64-linux-android`

The project uses the MIT License; third-party dependencies retain their own licenses.
