# Booking Engine

Booking is the rendering and world engine of Minecraft Book Edition. The client
owns platform lifecycle, input and presentation; Booking owns world state,
registries, terrain generation, and CPU-side preparation of renderable chunks.

## Current architecture

- **book-core:** stable block identifiers and content definitions.
- **book-world:** chunk columns, vertical sections, loaded-world lookup, and compact palette-backed block-state storage.
- **book-meshing:** CPU geometry generation with neighbor-face culling.
- **booking-engine:** world orchestration, built-in block registry, prototype
  terrain generation, and section-mesh preparation.
- **book-renderer-api:** graphics-backend contract.
- **book-renderer-opengl:** OpenGL / OpenGL ES backend and GPU presentation.
- **book-client:** Android application lifecycle and input event loop.

The renderer must consume world meshes prepared by Booking. Terrain generation
and block registry logic must not be duplicated inside graphics backends.

## Graphics targets

Android currently creates an OpenGL ES 3.0 context and uses GLSL ES 3.00.
This is the initial mobile compatibility baseline. The renderer API is intended
to allow additional backends, including Vulkan, without coupling the world
engine to a specific graphics API.

The phrase "mobile renderer" refers to a backend chosen for the device and its
supported graphics APIs. It does not imply that proprietary RenderDragon code
or binaries are reused.

## Planned render pipeline

1. Load world and resource-pack assets.
2. Resolve block definitions and texture atlas regions.
3. Generate or update palette-backed chunk sections.
4. Build CPU meshes using all six loaded neighbors.
5. Cull chunks outside the camera frustum and configured render distance.
6. Upload changed meshes to per-chunk GPU buffers.
7. Sample the block texture atlas with pixel-preserving filtering.
8. Render opaque terrain, cutout geometry, translucent geometry, sky, and
   environment passes in a defined order.
9. Apply mobile-appropriate post-processing and present the frame.

## Mod extension points

The Book Mod API is intended to expose stable hooks for block registration,
world events, gameplay changes, and renderer extensions. Mods should be able
to add or replace behavior through documented APIs. Direct access to raw GPU
handles or unrestricted mutation of engine internals is not the default API.

Native Rust modules are trusted platform-specific code and need a versioned
host ABI. Scripts and WebAssembly should use capability-limited host interfaces.
A requested capability is not automatically granted.

## Compatibility boundary

The current terrain generator and built-in texture atlas are prototypes. Camera state and basic Android touch-look/keyboard movement are now connected to the renderer. The world format, biome and
noise algorithms, block states, lighting model, resource-pack rendering,
entity pipeline, and Bedrock network protocol are not yet Bedrock-compatible.
Those systems must be implemented and tested independently before claiming
Bedrock world, pack, or server compatibility.
