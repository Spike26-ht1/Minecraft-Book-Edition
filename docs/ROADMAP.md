# Roadmap

## G0 — integrated app and first voxel scene

- [x] Preserve the existing launcher foundation.
- [x] Create a separate clean Rust game foundation.
- [x] Package the game Activity as a library inside the launcher APK.
- [x] Connect launcher **Jogar** to the embedded game menu.
- [x] Add game menu with Play, Settings and Exit.
- [x] Add landscape immersive game scene.
- [x] Add block/chunk storage, deterministic test terrain and CPU meshing.
- [x] Add Book64 OpenGL ES 3.0 rendering path.
- [ ] Verify Rust tests and Android ARM64 APK in CI.
- [ ] Install and validate on a physical Android device.

## G1 — playable voxel prototype

- [x] Add first-person free-camera look and touch movement controls for the mobile prototype.
- [x] Add a first-pass F3 diagnostic panel with measured FPS/frame time, viewport, heap and native camera telemetry.
- [x] Add an in-game test menu for F3 visibility, mobile controls and camera sensitivity.
- [ ] Add a fixed-step simulation loop, player physics, collision and grounded jumping.
- [ ] Add block ray selection and basic block interaction.
- [ ] Add player collision and gravity.
- [ ] Add multi-chunk world and cross-chunk neighbor lookup.
- [ ] Add mesh invalidation and bounded chunk loading.
- [ ] Add a full pause menu and persistent graphics settings.
- [ ] Expand F3 with biome, light, targeted block and player/world data as those systems become available.

## G2 — world foundation

- [ ] Define versioned block definitions and block states.
- [ ] Add world save/load format and recovery tests.
- [ ] Add configurable terrain generation.
- [ ] Add chunk streaming and unloading.
- [ ] Add biome and lighting systems.

## G3 — renderer

- [ ] Add Resource Pack parsing, texture atlas and material definitions (Mojang assets must remain user-supplied unless separately licensed).
- [ ] Add frustum culling and render-distance controls.
- [ ] Add GPU resource lifecycle handling.
- [ ] Profile frame pacing and memory on target devices.
- [ ] Implement Vulkan behind the Book64 backend boundary.

## G4 — Bedrock compatibility

- [ ] Select and document target Bedrock version.
- [ ] Implement protocol primitives, framing and codecs independently.
- [ ] Add packet fixtures and interoperability tests.
- [ ] Implement versioned world/resource readers.
- [ ] Verify against legitimate user-provided test data.

## G5 — gameplay and platform expansion

- [ ] Add inventory, items and entities.
- [ ] Add authentication and multiplayer after protocol design is tested.
- [ ] Add Windows host.
- [ ] Design stable public extension APIs and Book Loader after core client maturity.

## Release discipline

A CI artifact is not a device-verified build. Each test release must state version, ABI, renderer, included features, known limitations and the exact CI/device validation status.
