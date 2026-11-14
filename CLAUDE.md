# Claude project guide — Minecraft Book Edition

## Project identity
- Repository: https://github.com/Spike26-ht1/Minecraft-Book-Edition
- Main development branch: `foundation/voxel-core`
- Current milestone: Pre-Alpha 1.2.0
- Primary target: Android ARM64 (`arm64-v8a`), Android API 26+
- Graphics target: OpenGL ES 3
- Language: Rust
- This is an early Minecraft-like engine foundation, not a complete Minecraft clone.

## Critical current issue
The first Android APK installed on a Samsung Galaxy A17 4G but closed roughly half a second after launch. The cause has not been confirmed. Do not claim that the startup problem is fixed without physical-device testing and collected logs.

## Android storage added in Pre-Alpha 1.2.0
At Android startup, the app derives and validates its app-specific media directory from `AndroidApp::external_data_path()`, then creates `Android/media/com.spike26.minecraftbookedition/` with:
- `game/minecraftWorlds`
- `game/resource_packs`
- `game/behavior_packs`
- `game/development_resource_packs`
- `game/development_behavior_packs`
- `game/minecraftpe`
- `game/logs/latest.log`
- `launcher/config`
- `launcher/profiles`
- `launcher/instances`
- `launcher/downloads`
- `launcher/cache`
- `launcher/logs/launcher.log`

The log files receive timestamped startup entries. This creates directories only; it does not yet implement world persistence, pack import/discovery, launcher profiles, or settings persistence. If the external path does not match the expected Android layout, startup continues and the issue is reported through Logcat.

## Architecture
Workspace crates: `book-core`, `book-world`, `book-assets`, `book-meshing`, `book-renderer-api`, `book-renderer-opengl`, and `book-client`.

Android startup path:
1. `crates/book-client/Cargo.toml` — Android package metadata and NativeActivity setup.
2. `crates/book-client/src/lib.rs` — `android_main`, Android logger, storage initialization, Winit event loop and lifecycle.
3. `crates/book-renderer-opengl/src/lib.rs` — EGL/GLES context, surface and Glium display.
4. `.github/workflows/ci.yml` — Android SDK/NDK setup, APK build, validation and artifact upload.

## Required engineering workflow
Identify the exact branch and commit; inspect current source; separate confirmed defects from hypotheses; prioritize Android startup stability; prefer minimal changes; run formatting, tests and Android ARM64 build; report exactly what passed. CI proves compilation and packaging only, not physical-device behavior. Keep secrets and signing keys out of source and commits.

## Recommended model roles
Haiku: read-only audit with evidence and test suggestions.
Sonnet: independently verify findings, implement minimal justified changes, run checks, review the diff, and report remaining risks. Never claim a physical-device fix from CI alone.
