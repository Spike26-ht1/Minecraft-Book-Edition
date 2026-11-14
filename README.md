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
- Startup prepares only the spawn chunk before opening the Android render loop. The remaining 80 columns in the initial 9 × 9 area are generated and uploaded one column per frame behind a temporary loading overlay. Later player-centered expansion also runs incrementally. Loaded chunks are not unloaded yet.
- Native Android launch splash plus an in-engine `BOOKING / CARREGANDO MUNDO` screen with progress feedback. The in-engine screen stays visible until the initial world data and its GPU meshes are ready.
- Saved-world GPU uploads are deferred until after the first loading frame and synchronized incrementally, rather than uploading the whole saved world inside renderer construction.
- Neighbor-aware face culling across all six chunk-section directions.
- One merged GPU vertex buffer per chunk column, with distance-based chunk render culling.
- CPU texture-atlas packing, PNG decoding, and nearest-neighbor GPU sampling. Android now checks `Android/media/com.spike26.minecraftbookedition/game/resource_packs/` for an extracted Bedrock resource pack containing `textures/terrain_texture.json` and the referenced PNGs; it loads the grass-top, dirt, and stone tiles when present, otherwise it safely falls back to the generated prototype atlas.
- Face normals and directional lighting.
- First-person player controller with gravity, jumping, sprinting, and axis-separated voxel collision.
- 3D DDA block raycasting, touch-tap/mouse block breaking, and long-press prototype stone placement.
- Versioned binary world saves that preserve compressed section palettes, validate checksums, and autosave after edits and Android lifecycle suspension.
- Separate validated player-state saves for position and view direction, restored only when the saved pose is clear of solid blocks.
- Android keyboard movement/look plus a virtual left analog stick and mobile action buttons for jump, sprint, mine, and place.
- Multi-touch routing keeps joystick, camera-look, and action-button fingers independent.
- Pixel-art stone HUD controls with pressed-state feedback; these are generated Bedrock-inspired placeholders, not Mojang's original UI sprites.
- Test settings menu (`CFG`) with 30/60/90 FPS and uncapped modes plus 2/4/6/8/12-chunk render-distance choices.
- Persistent GPU buffers for the mobile HUD; debug statistics update at a limited interval rather than rebuilding the overlay every frame.
- F3 / DBG debug overlay with FPS, frame time, visible chunk count, and renderer information.

## Book Loader and mod API

The shared mod manifest supports native Rust, script, and WebAssembly runtime declarations, dependencies, and capability requests. The Rust API now defines lifecycle hooks, event subscriptions, block registration, and world read/write contracts through a restricted host context.

These are API foundations, not a completed runtime. Package discovery, dependency resolution, capability enforcement, script execution, WebAssembly hosting, and a stable native ABI are still under development.

## Current limitations

This is **not yet a Bedrock-compatible game**. It starts with a 9 × 9 prototype chunk area and expands around the player incrementally, but does not unload distant chunks yet. The resource-pack reader currently accepts extracted folders (not `.mcpack` archives), searches the resource-pack directory and one level of child folders, and imports only grass-top, dirt, and stone terrain textures; full block texture variants, animation, mipmaps, resource-pack priority/overrides, and automatic asset downloading are not implemented. The mobile HUD uses a generated Bedrock-inspired stone pixel pattern until the official UI texture loader is integrated. The custom Booking save file is not a Bedrock world format. Remaining work includes disk-backed chunk unloading/region storage, production terrain and biome generation, Bedrock block-state property definitions and exact palette serialization, lighting propagation, entity rendering/simulation, inventory and item systems, inventory persistence, networking, and full touch movement controls. The player and block-interaction systems are early foundations, not complete Bedrock gameplay.

Bedrock world, resource-pack, add-on, and server compatibility must be implemented and verified subsystem by subsystem. The project does not reuse proprietary RenderDragon code or game binaries.

## Importing an extracted Bedrock resource pack

Copy an extracted resource-pack folder into:

`Android/media/com.spike26.minecraftbookedition/game/resource_packs/<pack-name>/`

The folder should contain `textures/terrain_texture.json` and the PNG files referenced by its `texture_data` entries. The current prototype importer looks for `grass_top` (or `grass` / `grass_block`), `dirt`, and `stone`. It does not download proprietary game assets, extract `.mcpack` archives, or access another app's private Minecraft installation. If no compatible extracted pack is found, the renderer uses its built-in test atlas.

## Android test build

- Android 8.0 or newer (API 26).
- ARM64 / arm64-v8a.
- Landscape-only immersive fullscreen mode with a native Android startup splash theme and a temporary in-engine loading screen.
- OpenGL ES 3.0.
- F3 on a connected physical keyboard toggles the debug overlay.
- Left analog stick: move and strafe.
- PULO: jump; CORRER: sprint while held.
- MINAR: break the targeted block; COLOCAR: place prototype stone against the targeted face.
- Dragging outside the controls changes the camera view.
- DBG toggles the debug overlay from the upper-right safe area.
- CFG opens test graphics settings: FPS cap (30/60/90/uncapped) and render distance (2–12 chunks).
- Uncapped FPS and higher render distances can increase power use, heat, and frame-time variation; they are test options, not guaranteed performance gains.

The CI APK is a debug test package, not a store-ready release. Successful compilation and APK validation do not replace runtime testing on a physical Android device. World saves are stored under the app's Android/media data directory in the Booking binary format.

## Build checks

- Desktop: `cargo test --workspace`
- Formatting: `cargo fmt --all -- --check`
- Android ARM64: `cargo apk build -p book-client --lib --target aarch64-linux-android`

The project uses the MIT License; third-party dependencies retain their own licenses.
