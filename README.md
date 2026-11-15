# Minecraft Book Edition

Minecraft Book Edition is an independent, open and documented Bedrock-inspired game-client project. Development starts from a clean Rust game foundation; the abandoned prototype is not reused.

## Current target

- Android ARM64 first.
- The launcher is the single installed application and starts the embedded game.
- Kotlin + Jetpack Compose + Material 3 for the launcher.
- Rust for the game core and native runtime.
- Book64 with OpenGL ES 3.0 as the first graphics backend.
- Windows and Vulkan are future targets.

## Current development build

The launcher now contains the game runtime as an Android library module:

1. Open Minecraft Book Edition.
2. Press **Jogar** in the launcher.
3. The game main menu appears.
4. Choose **JOGAR** to open the voxel test scene.
5. Use the in-game return button or Android Back to return to the game menu.

The game menu includes a first-pass Configurações dialog. It currently reports the test renderer and platform; detailed graphics controls are not implemented yet.

The first scene renders deterministic grass/dirt/stone terrain generated from a 16 × 16 × 16 block section. The Rust mesher removes faces shared by adjacent solid blocks.

**This is not yet a Minecraft-compatible client.** There is no player movement, collision, persistent world, inventory, entities, multiplayer, Bedrock protocol implementation, or official game asset loading.

## Architecture

- `launcher/app/`: main Android launcher and app entry point.
- `game/android/app/`: embedded Android game Activity and OpenGL surface.
- `game/native/crates/book-core/`: platform-independent coordinates and primitives.
- `game/native/crates/book-world/`: block identifiers and chunk-section storage.
- `game/native/crates/book-meshing/`: visible voxel face generation.
- `game/native/crates/book64-gles/`: OpenGL ES 3.0 renderer.
- `game/native/crates/book-android/`: Android JNI bridge.
- `docs/ARCHITECTURE.md`: technical boundaries.
- `docs/ROADMAP.md`: staged plan.
- `docs/BUILDING.md`: build and validation instructions.

## Build and test

Run Rust checks from `game/native`:

```sh
cargo test --workspace
cargo fmt --all -- --check
```

The GitHub Actions workflow builds the native ARM64 library, runs the Rust checks and assembles the **combined launcher + game debug APK**. A successful CI run and a physical-device test are required before calling a build verified.

## Clean-room rules

- Do not reuse code from the abandoned prototype or copy Cinnabar.
- Cinnabar may be used as an architectural reference only.
- Do not bundle proprietary Minecraft binaries, assets or credentials.
- Implement and test Bedrock data/protocol compatibility independently.
- Keep the launcher as the app entry point and the game runtime embedded within it.
