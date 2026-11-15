# Minecraft Book Edition

Minecraft Book Edition is an independent, open and documented game-client project. This branch is the **clean game foundation**: it does not reuse the previous prototype engine or its runtime code.

## Current development target

- Android ARM64 first.
- Rust for the game core and native runtime.
- Book64 as the graphics layer.
- OpenGL ES 3.0 as the first Android renderer.
- Vulkan is a later selectable backend, not part of this first test.
- Windows is a future platform.
- The Android game application is separate from the existing launcher.

## First game test scene

The first Android test app opens in landscape and immersive fullscreen, displays the supplied Book Edition artwork, and renders a small deterministic voxel terrain through Book64/OpenGL ES 3.0. The scene uses grass, dirt and stone block IDs and CPU-side neighbor-aware face culling. It verifies the Android surface, JNI boundary, Rust world model, meshing path and native renderer before larger world systems are added.

This is **not yet a Minecraft-compatible game**. It does not include player controls, a persistent world, inventory, entities, networking, login, or Bedrock assets.

## Architecture

- `game/android/`: Android game host application.
- `game/native/crates/book-core/`: platform-independent game primitives.
- `game/native/crates/book-world/`: compact block and chunk-section storage.
- `game/native/crates/book-meshing/`: CPU voxel meshing with internal-face removal.
- `game/native/crates/book64-gles/`: first Book64 renderer backend.
- `game/native/crates/book-android/`: Android JNI bridge.
- `docs/ARCHITECTURE.md`: technical boundaries and compatibility strategy.
- `docs/ROADMAP.md`: staged game development plan.

## Build and test

From the repository root:

```sh
cd game/native
cargo test --workspace
cargo fmt --all -- --check
```

Android ARM64 build (requires Android SDK/NDK and `cargo-ndk`):

```sh
cd game/native
cargo ndk -t arm64-v8a -o ../android/app/src/main/jniLibs build --release -p book-android
gradle -p ../android assembleDebug
```

The GitHub Actions workflow performs the same checks and publishes a debug APK artifact when the build succeeds.

## Scope and ownership

Only the supplied project artwork is included as a visual resource at this stage. Minecraft assets, official binaries, authentication secrets, and proprietary renderer code are not bundled. Compatibility with Bedrock formats and network protocols will be implemented independently and verified subsystem by subsystem.
