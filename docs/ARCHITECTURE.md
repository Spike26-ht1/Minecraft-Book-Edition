# Game architecture

## Product direction

Book Edition is being developed as an independent client, not as a wrapper around the installed Minecraft application. The game code is being written from a clean foundation in Rust. The existing launcher remains a separate project and is not modified by this branch.

The goal is to build an open, understandable client and engine that can progressively implement compatibility with Bedrock data and network formats. This is a long-term compatibility project; the current test application is a first voxel-rendering milestone.

## Layer boundaries

```text
Android game host
  └── JNI bridge
       └── Book64 graphics layer
            └── OpenGL ES 3.0 backend
       └── Book Core
            ├── Book World (blocks and chunk sections)
            ├── Book Meshing (visible voxel faces)
            ├── input/simulation (future)
            └── persistence (future)

Separate future layers:
  Bedrock-compatible protocol
  Bedrock resource/data readers
  world storage and chunk streaming
```

### Android host

Owns the Activity lifecycle, landscape orientation, immersive mode, OpenGL ES surface and app-level presentation. It forwards surface lifecycle calls through a narrow JNI bridge.

### Book Core

Contains platform-independent coordinate types and deterministic logic. Android APIs, OpenGL calls and network sockets must not leak into this crate.

### Book World

Owns block identifiers and chunk-section storage. The current section is 16 × 16 × 16 and uses compact block IDs. The test terrain is deterministic and is only a renderer fixture, not a finished terrain generator or Bedrock palette implementation.

### Book Meshing

Builds CPU-side triangle meshes from block data and removes faces shared by solid blocks. This first implementation checks neighbors inside a single section. Cross-section neighbor lookup, worker scheduling and mesh invalidation remain future work.

### Book64

Owns graphics backend selection and rendering. OpenGL ES 3.0 is the first backend. A future Vulkan backend must implement the same renderer-facing contract; adding Vulkan must not require rewriting world simulation.

### Client and protocol boundary

The client will own the game session and coordinate assets, world state, input, rendering and transport. The Bedrock-compatible protocol layer will remain independent from the renderer. Protocol versions, packet codecs and authentication flow must be documented and tested separately before claiming compatibility.

## Clean-room rules

- Do not copy code from the abandoned prototype or Cinnabar.
- Cinnabar may be studied only as an architectural reference; its code and license are not reused.
- Do not bundle official game binaries or proprietary game assets.
- Use only artwork supplied for this project or assets with an explicitly compatible license.
- Keep the launcher and game host separate.
- Do not claim Bedrock compatibility until individual subsystems pass compatibility tests.

## Current implementation

- Landscape-only Android game host with immersive fullscreen.
- User-supplied Book Edition artwork displayed in the app.
- Rust/JNI bridge from the Android OpenGL ES surface.
- Book Core block-coordinate conversion tests, including negative world coordinates.
- Book World 16 × 16 × 16 block section with deterministic grass/dirt/stone test terrain.
- Book Meshing CPU pipeline that removes faces shared by adjacent solid blocks.
- Book64 OpenGL ES 3.0 backend that uploads and rotates the generated voxel terrain.

There is no player simulation, persistent world, inventory, entity system or Bedrock network implementation in this milestone.
