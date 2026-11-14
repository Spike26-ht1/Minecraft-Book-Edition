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

## Later

- World save/load.
- Lighting and ambient occlusion.
- Full Bedrock resource-pack support.
- Mod API and community compatibility tests.
- Android build and touch controls.
- Vulkan backend implementing the same renderer contract.
- Backend selection in settings.
