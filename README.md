# Minecraft Book Edition

Book Edition is an independent, open and documented reimplementation project targeting Minecraft Bedrock Edition. The game client is being written in Rust; Android is the first host platform.

## Product direction

- **A real independent client**, not a wrapper that launches the official game's native runtime.
- **Android ARM64 first**, with Windows planned as another host.
- **Kotlin + Jetpack Compose** for launcher, settings, files, and logs.
- **Rust** for world state, simulation, client systems, and native runtime.
- **Book64** as the rendering boundary; OpenGL ES 3.0 is the initial Android backend.
- Bedrock formats and behaviour are implemented independently and validated against versioned fixtures. Bedrock 26.60 is the target compatibility line.

## Current development milestone

The launcher and game runtime are packaged together in one APK. The in-game menu starts a Rust-owned single-player session.

The current Rust layers are:

- `book-core`: shared coordinates and low-level types.
- `book-world`: loaded world columns, 16 × 16 × 16 sections, block lookup/mutation, and the current versioned procedural generator.
- `book-game`: single-player session, fixed 20 TPS simulation clock, player state, movement, gravity, jumping and basic block collision.
- `book-meshing`: CPU mesh extraction and shared-face removal across loaded chunks.
- `book64-gles`: OpenGL ES presentation of a world mesh and immutable player render snapshots.
- `book-android`: Android JNI runtime composition; it owns the game session and renderer as separate objects.

The mobile directional pad now controls a simulated player. Touch drag rotates the view. The jump button requests a jump; it no longer moves the camera upward. The F3 panel reads telemetry from the game session and Book64.

The current generator is Book Edition's own algorithm version 1; it is **not** Bedrock's seed algorithm. The loaded world currently contains a 5 × 5 region of horizontal chunk columns and four vertical sections per column.

## Not implemented yet

This milestone is the beginning of the playable client, not Bedrock parity. The following remain future work:

- Saving and reopening world data.
- Breaking/placing blocks and block ray selection.
- Chunk streaming, background generation, unloading and mesh invalidation.
- Bedrock-compatible block states, registries, world formats and world generation.
- Uploading imported Resource Pack atlases to the GPU and using real block UVs/materials.
- Lighting, entities, inventory, audio, UI screens, protocol, authentication and multiplayer.

The Resource Pack importer currently creates a local atlas and index only. It does not yet connect that atlas to Book64. The launcher can detect the official package for diagnostics, but installation detection does not prove a license or grant access to another app's private sandbox.

## Build and test

From `game/native`:

```sh
cargo test --workspace
cargo fmt --all -- --check
```

GitHub Actions runs Rust tests and formatting, builds the ARM64 native library, then assembles the combined launcher + game debug APK. CI success proves compilation and automated tests; it does not replace installation and testing on a physical device.

## Reference study

The architectural lessons and license review for Horizon, Cinnabar and LeviLauncher are recorded in [docs/REFERENCE_STUDY.md](docs/REFERENCE_STUDY.md). The references are studied for architecture and engineering methods; their code is not copied.

## Clean-room rules

- Do not reuse code from the abandoned prototype.
- Do not copy or link Cinnabar or Horizon source into this repository.
- Review the license of every dependency before adding it.
- Do not bundle or redistribute proprietary Minecraft binaries, assets or credentials.
- Resource packs and other assets remain user-supplied and local unless separately licensed.
- Do not claim Bedrock compatibility without version-specific tests.
- Keep the launcher as the app entry point and the independent game runtime embedded in the APK.
