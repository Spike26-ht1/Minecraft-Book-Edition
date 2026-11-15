# Game development roadmap

The roadmap below is the source of truth for sequencing work on the game. Each milestone must be built and tested before dependent systems are expanded.

## G0 — clean runtime and graphics proof

- [x] Create a new game-only development branch.
- [x] Keep the existing launcher branch separate.
- [x] Create an Android ARM64 game host in landscape/fullscreen.
- [x] Display the supplied Book Edition artwork in the game app.
- [x] Establish the Rust/JNI boundary.
- [x] Establish Book64 with an OpenGL ES 3.0 diagnostic scene.
- [ ] Verify the APK on a physical Android device.

## G1 — deterministic game foundation

- [ ] Define the main game loop and fixed-step simulation clock.
- [ ] Define input actions independently from Android touch controls.
- [ ] Add tests for world coordinates, chunk coordinates and block addressing.
- [ ] Add structured game logs and safe export for diagnostics.
- [ ] Establish resource ownership and lifecycle behavior across Activity pause/resume.

## G2 — world and block model

- [ ] Define block identifiers and immutable block definitions.
- [ ] Define chunk/section storage and versioned serialization.
- [ ] Implement deterministic terrain fixtures for tests.
- [ ] Add chunk meshing with neighbor-aware face visibility.
- [ ] Add bounded chunk streaming and unloading before increasing world size.

## G3 — graphics evolution

- [ ] Replace the diagnostic cube with world geometry.
- [ ] Add camera, depth, frustum culling and GPU resource lifetime management.
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
