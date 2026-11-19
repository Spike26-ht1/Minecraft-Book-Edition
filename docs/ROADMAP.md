# Roadmap

The project is an independent Bedrock client written in Rust. The launcher is the Android host, not the game itself. Work is organized around player-visible game systems and versioned compatibility, not repeated renderer demonstrations.

## G0 — Android host and native rendering foundation

- [x] Keep the launcher as the single Android application entry point.
- [x] Package the game runtime in the launcher APK.
- [x] Add a landscape game Activity and mobile HUD.
- [x] Add Book64 OpenGL ES 3.0 backend.
- [x] Add world block storage, procedural test generation and CPU meshing.
- [x] Add seeded world creation and pass seed through JNI.
- [x] Add F3 telemetry and frame pacing settings.
- [x] Build ARM64 native library and combined debug APK in CI.
- [ ] Validate the current playable-session build on a physical Android device.

## G1 — first independent single-player session

- [x] Add a world model containing loaded chunk columns and 16 × 16 × 16 sections.
- [x] Separate world storage from mesh generation and rendering.
- [x] Add `book-game` as the owner of single-player session state.
- [x] Add fixed 20 Hz simulation independent of render frame rate.
- [x] Add player position, velocity, view rotation and grounded state.
- [x] Add horizontal movement, gravity, jump input and basic solid-block collision.
- [x] Make Book64 render immutable simulation snapshots instead of owning a free camera.
- [x] Make the JNI host own the simulation session and renderer separately.
- [x] Add first-person block targeting / ray selection.
- [x] Add block breaking and placement with world mutations.
- [x] Add versioned local world save, close and reopen flow.
- [ ] Add inventory data model and first hotbar interaction.

## G2 — world lifecycle and streaming

- [x] Keep the generator deterministic and versioned.
- [x] Add global block queries across loaded positive and negative chunk coordinates.
- [ ] Define versioned world metadata: identifier, name, seed, generator version, spawn and settings.
- [x] Implement bounded binary save/load for loaded chunk sections.
- [x] Add deterministic round-trip and truncated-file rejection tests.
- [ ] Persist player position, inventory and world metadata.
- [ ] Add background chunk generation, bounded work queues and unloading.
- [ ] Add mesh invalidation for changed blocks and neighboring chunks.
- [ ] Add height maps, biomes, decoration, water, caves, ores and structures in tested stages.
- [ ] Port Bedrock 26.60 world generation only after acquiring versioned reference fixtures.

## G3 — Bedrock content and rendering

- [x] Parse the first pass of user-supplied Resource Pack terrain texture metadata.
- [x] Generate a local atlas PNG and texture index.
- [ ] Upload atlas pages to Book64 and use atlas UVs for block faces.
- [ ] Implement block definitions, states, material classes and transparency.
- [ ] Add Bedrock block geometry and render-controller support.
- [ ] Add flipbook textures, animation, biome tint, lighting and fog.
- [ ] Add culling, draw batching, GPU resource lifecycle and memory budgets.
- [ ] Profile on target Android GPUs before adding more rendering features.

## G4 — Bedrock compatibility

- [ ] Pin exact target game build and record format versions separately.
- [ ] Implement versioned Bedrock registries and data readers.
- [ ] Implement world/subchunk palette and NBT readers/writers.
- [ ] Implement protocol framing, packet codecs and transport in isolated modules.
- [ ] Add versioned fixtures and differential/acceptance tests for each subsystem.
- [ ] Verify against legitimate user-provided data without bundling proprietary game files.

## G5 — full client systems

- [ ] Entities, mob behaviour and animation.
- [ ] Items, inventory, crafting and containers.
- [ ] Audio, HUD, pause/settings screens and accessibility.
- [ ] Authentication and multiplayer once local single-player is stable.
- [ ] Windows host.
- [ ] Design Book Loader after core client behaviour and data contracts stabilize.

## Release discipline

Every test build must state its version, ABI, renderer, included systems, known limitations, CI result and device-test status. CI success does not mean the APK has been tested on a physical device.
