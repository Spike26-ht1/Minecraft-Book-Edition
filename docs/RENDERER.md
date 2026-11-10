# Renderer design

## Initial backend: OpenGL

The 26.60 test client targets desktop x86_64 (64-bit) on Windows and Linux. The OpenGL backend uses Glium over Glutin to create a context and draw chunk-mesh test geometry.

The current terrain is generated test data, not a loaded world.

## F3 debug overlay

Press F3 to show or hide the in-game debug overlay. The initial panel reports:
- FPS, calculated from rendered frames.
- Frame time in milliseconds.
- Active renderer and graphics API.
- Chunk, block, and vertex counters.

Chunk and block counts are marked as test values until a real world/chunk manager is connected. The overlay uses a compact built-in bitmap font so the initial debug HUD does not require an external font asset.

## Backend-neutral boundary

The client consumes book-renderer-api and chooses a backend implementation. World storage, block definitions, resource-pack metadata, and CPU meshes do not import OpenGL types.

A future Vulkan implementation should be a separate crate implementing RendererBackend. Backend selection will happen during startup or in settings.

## Why not translate Vulkan into OpenGL?

A Vulkan-to-OpenGL command translator would add complexity and overhead while duplicating functionality that belongs in a renderer backend. The intended design is one rendering interface with separate native implementations:

    book-renderer-api
       /          \
    OpenGL       Vulkan
    backend      backend
       \          /
        book-client

Sodium is a Java Edition renderer optimization mod, not a Vulkan-to-OpenGL translation layer. We can study public performance techniques and design ideas, but Book Edition's renderer will be implemented independently in Rust.

## Next renderer tasks

1. Camera matrices and view controls.
2. Bedrock atlas image decoding and texture upload.
3. Replace test terrain with loaded/generated world chunks.
4. Chunk draw batching and frustum culling.
5. Accurate world, chunk, entity, and memory values in the F3 panel.
6. Transparent render layers and depth sorting.
7. Profiling before adding advanced optimizations.
