# Roadmap

## 26.60 — Foundation

- [x] Rust workspace and crate boundaries.
- [x] Block registry and chunk-section storage.
- [x] Bedrock-style terrain atlas metadata parser.
- [x] Basic CPU chunk mesher.
- [x] Renderer backend contract.
- [x] OpenGL window and cube smoke test.
- [ ] CI build verification on supported desktop targets.
- [ ] Render a real chunk mesh through OpenGL.

## 26.61 — First voxel scene

- Camera and player input.
- Render chunk meshes instead of the cube smoke test.
- Atlas image decoding and GPU texture upload.
- Block-face UVs and basic terrain material.
- World generation and a small test world.

## Pre-Alpha 1.2.1 — Android runtime diagnostics

- [x] Resolve the media/resource-pack directory from the actual Android application ID, including side-by-side diagnostic package IDs.
- [x] Add regression tests for normal, suffixed, and malformed Android package paths.
- [x] Record long update and render/present stalls with per-stage timings.
- [ ] Use collected device logs to isolate and fix the reported periodic freeze.
- [ ] Verify an extracted resource pack loads on a physical Android device.

## Later

- World save/load.
- Lighting and ambient occlusion.
- Full Bedrock resource-pack support.
- Mod API and community compatibility tests.
- Android build and touch controls.
- Vulkan backend implementing the same renderer contract.
- Backend selection in settings.
