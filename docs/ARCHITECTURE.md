# Architecture — 26.60

## Design rules

1. Keep world data independent from graphics APIs.
2. Keep the renderer API independent from OpenGL and Vulkan types.
3. Keep resource-pack metadata separate from GPU texture objects.
4. Prefer safe Rust and small dependencies in foundational crates.
5. Add compatibility incrementally, with fixtures and tests.

## Crates

- book-core: shared block identifiers and definitions.
- book-world: chunk-section storage.
- book-assets: Bedrock-style atlas metadata parsing.
- book-meshing: CPU chunk mesh generation.
- book-renderer-api: backend-neutral renderer contract.
- book-renderer-opengl: initial OpenGL implementation.
- book-client: desktop prototype entry point.

The OpenGL renderer implements the renderer contract. A future Vulkan backend will implement the same contract as a separate backend. There is no command translation layer between Vulkan and OpenGL.

## World storage

The first world primitive is a 16 × 16 × 16 section with compact 16-bit block identifiers. Full chunk columns, height ranges, lighting, biome storage, serialization, and networking are future work.

## Resource packs and texture atlases

book-assets reads a subset of the metadata structure used by Bedrock-style terrain_texture.json files. It does not yet decode the atlas image, resolve every texture variation, or implement all resource-pack rules. GPU upload belongs to the renderer.

## Meshing

book-meshing creates CPU-side face vertices and omits faces adjacent to solid blocks in the same section. Neighbor-section visibility, per-face textures, ambient occlusion, greedy meshing, and lighting are future improvements.

## Renderer

The first implementation uses Glium/Glutin over OpenGL. The client draws an animated cube to verify context creation, shader compilation, depth testing, and frame presentation. This is a smoke test, not the final voxel renderer.

## Compatibility

External formats are imported into a documented Book Edition content model. Support is implemented per format and per mod. Arbitrary Java bytecode, mixins, and Java renderer replacement mods are outside the initial scope.

## Platforms

The data layer is portable Rust. Desktop OpenGL is the first renderer milestone. x86, x86_64, and Android require separate dependency, driver, and CI validation before being called supported.
