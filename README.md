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

The game scene supports a persistent VSync/uncapped FPS selection. Uncapped mode requests EGL swap interval 0 and the F3 panel reports the active pacing mode and measured CPU frame interval. The Android compositor and display may still cap visible presentation.\n\nThe game scene now has a mobile HUD: touch-drag camera look, a directional movement pad, vertical movement, an F3 diagnostic panel and an in-game test menu. The test menu can toggle F3 and controls and cycle camera sensitivity. The F3 panel reports measured FPS/frame time, display size, Android heap and native camera/viewport/vertex data; systems that do not exist yet are not fabricated.

The Rust world crate includes a deterministic seeded terrain generator (algorithm version 1), initial plains/forest/desert/mountain classification, surface layers and bedrock. The **JOGAR** flow opens a world-creation dialog with numeric seed entry and random-seed generation; the selected seed is remembered locally and passed through JNI into Book64. The dev.9 renderer builds a 5 × 5 horizontal region, with four 16 × 16 × 16 vertical sections per chunk column. The mesher checks neighbors across horizontal and vertical chunk boundaries, and the terrain remains fixed instead of rotating as a demo object. This is still a generated prototype: it does not yet save blocks, stream chunks, decorate biomes or reproduce Bedrock's seed algorithm. See [docs/WORLD_GENERATION.md](docs/WORLD_GENERATION.md).

**This is not yet a Minecraft-compatible client.** Mobile controls currently move a free camera through the test scene; there is not yet player collision, gravity, grounded jumping, persistent world, inventory, entities, multiplayer, Bedrock protocol implementation or Resource Pack texture loading.

## Optional official Bedrock installation detection

The launcher includes a read-only Android package probe for `com.mojang.minecraftpe`. This reports installation/version and signer fingerprints for diagnostics only; it does not prove purchase or license ownership. Resource Packs must be selected by the user through Android's document picker or supplied as files they are authorized to use. The app does not access Minecraft's private data or bundle Mojang assets. See [docs/BEDROCK_INSTALLATION_INTEGRATION.md](docs/BEDROCK_INSTALLATION_INTEGRATION.md).

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
- Do not bundle proprietary Minecraft binaries, assets or credentials. Bedrock Samples assets are subject to the Minecraft EULA; see docs/ASSETS_AND_TEXTURES.md.
- Implement and test Bedrock data/protocol compatibility independently.
- Keep the launcher as the app entry point and the game runtime embedded within it.
