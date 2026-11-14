# Renderer design

## First test-build platform: Android ARM64

The first public test build targets Android 8.0 (API 26) and newer on 64-bit ARM devices (aarch64-linux-android). The package format is APK.

The initial graphics backend is OpenGL ES. The renderer remains behind the backend-neutral book-renderer-api interface so a separate Vulkan backend can be introduced later without translating Vulkan commands into OpenGL.

## OpenGL renderer

The current renderer uses Glium over Glutin/Winit to create the graphics context and draw chunk-mesh test geometry. Android window and graphics-surface creation must follow the Winit lifecycle: create the surface after Resumed and release it on Suspended.

The current terrain is generated test data, not a loaded world.

## F3 debug overlay

The debug panel reports:
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
