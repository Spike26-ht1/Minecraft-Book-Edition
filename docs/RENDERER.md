# Renderer design

## Initial backend: OpenGL

The 26.60 desktop prototype uses Glium over Glutin to create an OpenGL context, compile a shader program, draw a rotating cube, and present frames.

This cube is a graphics smoke test. It is not the voxel-world renderer.

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

1. GPU upload for book-meshing output.
2. Camera matrices and view controls.
3. Bedrock atlas image decoding and texture upload.
4. Chunk draw batching and frustum culling.
5. Transparent render layers and depth sorting.
6. Profiling before adding advanced optimizations.
