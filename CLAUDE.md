# Claude project guide — Minecraft Book Edition

## Project identity

- Repository: https://github.com/Spike26-ht1/Minecraft-Book-Edition
- Main development branch: `foundation/voxel-core`
- Current milestone: Pre-Alpha 1.1.0
- Primary target: Android ARM64 (`arm64-v8a`), Android API 26+
- Graphics target: OpenGL ES 3
- Language: Rust
- This is an early Minecraft-like engine foundation, not a complete Minecraft clone.

## Critical current issue

The first Android APK installed on a Samsung Galaxy A17 4G but closed roughly half a second after launch. Android did not show a crash dialog. The cause is unknown.

Pre-Alpha 1.1.0 added Android logging using the `BookEdition` tag, a panic hook, and checkpoints around Winit/EGL/OpenGL initialization. It also changed the requested depth buffer from 24 to 16 bits and added the launcher icon. These are diagnostic/compatibility attempts, not proof that the runtime crash is fixed.

Important distinction:
- A successful GitHub Actions build proves compilation and APK packaging only.
- It does NOT prove the app launches or renders correctly on a physical phone.
- Do not claim the Samsung A17 issue is fixed without runtime evidence or logs.

## Architecture

Workspace crates currently include:

- `crates/book-core`
- `crates/book-world`
- `crates/book-assets`
- `crates/book-meshing`
- `crates/book-renderer-api`
- `crates/book-renderer-opengl`
- `crates/book-client`

Android startup path to inspect first:

1. `crates/book-client/Cargo.toml` — Android package metadata, native activity setup, resources.
2. `crates/book-client/src/lib.rs` — `android_main`, Android logger, Winit event loop, lifecycle.
3. `crates/book-renderer-opengl/Cargo.toml` — GL dependencies.
4. `crates/book-renderer-opengl/src/lib.rs` — Android window, EGL config, GLES context, surface and Glium display.
5. `.github/workflows/ci.yml` — Android SDK/NDK setup, APK build, validation and artifact upload.

Inspect the actual current source before making claims. Do not assume the descriptions in this file are still accurate after later commits.

## Known project limitations

The project is a technical pre-alpha. Do not imply that features exist unless source code proves they are implemented. In particular, player controls, camera, full world generation, persistence, networking, complete lighting/transparency, real PNG atlas loading and complete Bedrock resource-pack/add-on support may still be incomplete.

## Required engineering workflow

1. Start by identifying the exact branch and commit being reviewed.
2. Read the relevant source files and trace the complete startup path.
3. Separate confirmed defects from hypotheses. Give file paths and line numbers where possible.
4. Prioritize the Android launch failure before unrelated features.
5. Prefer the smallest reversible fix. Avoid broad rewrites.
6. Do not delete or replace existing working code without explaining why.
7. Do not invent APIs, dependencies, test results, device capabilities or runtime logs.
8. Check Android lifecycle behavior, NativeActivity entry point, Winit Android integration, EGL config selection, GLES version, surface creation, context activation, Glium initialization, shader compilation and panic/error handling.
9. Consider that the Samsung A17 is ARM64, but verify device requirements from evidence rather than assuming every EGL/GLES configuration is supported.
10. Run formatting, tests, desktop checks and Android ARM64 build when available.
11. Report exactly what passed. Clearly state that physical-device testing remains outstanding until it is actually performed.
12. Keep secrets, API keys, signing keys and personal tokens out of source files and commits.

## Recommended model roles

### Haiku — investigation first

Use Haiku for a read-only audit:
- Map the Android startup path.
- Find likely crash points and unsafe assumptions.
- Inspect error paths, panics and missing logs.
- Rank hypotheses by evidence and propose tests.
- Do not modify files during the initial audit.
- Return a concise report with file paths, exact evidence, confidence, and the smallest next diagnostic step.

### Sonnet — implementation and review

Use Sonnet after reviewing Haiku's findings:
- Independently verify the findings against current source.
- Implement only well-supported fixes.
- Add regression tests or better diagnostics where possible.
- Run available checks and inspect the final diff.
- Do not claim a physical-device fix based only on CI.
- Prepare a focused commit or pull request and summarize changed files and test results.

### Collaboration rule

Haiku's report is a hypothesis list, not authority. Sonnet must verify every proposed change. If the two models disagree, inspect the code and tests rather than choosing a model by name.

## Suggested first investigation

Trace from Android's `android_main` through Winit's `Resumed` event to the first rendered frame. Identify every operation that can panic, return an error, or terminate the event loop. Determine whether the new `BookEdition` logs can capture failures before, during and after EGL/GLES setup. Propose a way to collect the startup log from a physical Android device without exposing secrets.
