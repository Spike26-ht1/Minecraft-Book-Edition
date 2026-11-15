# Game development roadmap

The roadmap below is the source of truth for sequencing work on the game. Each milestone must be built and tested before dependent systems are expanded.

## G0 — clean runtime and voxel-rendering proof

- [x] Create a new game-only development branch.
- [x] Keep the existing launcher branch separate.
- [x] Create an Android ARM64 game host in landscape/fullscreen.
- [x] Display the supplied Book Edition artwork in the game app.
- [x] Establish the Rust/JNI boundary.
- [x] Create a compact 16 × 16 × 16 block section.
- [x] Generate deterministic grass/dirt/stone test terrain.
- [x] Mesh visible faces and remove faces shared by adjacent solid blocks.
- [x] Render the voxel test terrain with Book64/OpenGL ES 3.0.
- [ ] Verify the APK on a physical Android device.

## G1 — deterministic game foundation

- [ ] Define the main game loop and fixed-step simulation clock.
- [ ] Define input actions independently from Android touch controls.
- [ ] Expand tests for world coordinates, block addressing and section boundaries.
- [ ] Add structured game logs and safe export for diagnostics.
- [ ] Establish resource ownership and lifecycle behavior across Activity pause/resume.
- [ ] Add cross-chunk neighbor lookup and mesh invalidation.

## G2 — world systems

- [ ] Define immutable block definitions and block properties.
- [ ] Add versioned chunk serialization.
- [ ] Replace the test terrain fixture with a configurable terrain generator.
- [ ] Add bounded chunk streaming and unloading before increasing world size.
- [ ] Add biome data and lighting propagation.

## G3 — graphics evolution

- [ ] Add camera controls and player input.
- [ ] Add frustum culling, render-distance management and GPU resource lifetime handling.
- [ ] Add texture atlas loading for user-supplied test assets.
- [ ] Profile CPU, GPU and frame pacing on the target Android device.
- [ ] Keep Vulkan as a later backend behind the same Book64 renderer boundary.

## G4 — Bedrock compatibility

- [ ] Select and document a specific Bedrock protocol/data version.
- [ ] Implement packet primitives and framing independently.
- [ ] Add protocol fixtures and interoperability tests.
- [ ] Implement world/resource format readers in isolated crates.
- [ ] Verify each format against legitimate user-provided test data.

## G5 — gameplay and online systems

- [ ] Player controller and collision.
- [ ] Interaction, inventory and item model.
- [ ] Entity model and simulation.
- [ ] Authentication and network session flow after protocol requirements are documented.
- [ ] Multiplayer/server interoperability tests.

## G6 — platform expansion and modding

- [ ] Windows host.
- [ ] Vulkan renderer backend.
- [ ] Stable public extension API.
- [ ] Book Loader and Rust/script integration only after the core client is stable.

## Release discipline

Every test build must state its version, target ABI, renderer, included functionality and known limitations. A successful CI build is not a substitute for testing on a physical device.
