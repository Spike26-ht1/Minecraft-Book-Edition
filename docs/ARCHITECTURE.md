# Game architecture

## Product direction

Book Edition is being developed as an independent client, not as a wrapper around the installed Minecraft application. The game code is being written from a clean foundation in Rust. The existing launcher remains a separate project and is not modified by this branch.

The goal is to build an open, understandable client and engine that can progressively implement compatibility with documented and tested Bedrock data and network formats. This is a long-term compatibility project; the initial test application is only a graphics/runtime smoke test.

## Layer boundaries

```text
Android game host
  └── JNI bridge
       └── Book64 graphics API
            └── OpenGL ES 3.0 backend
       └── Book Core
            ├── world model (future)
            ├── session lifecycle
            └── input/simulation (future)

Future protocol and asset layers remain separate from rendering:
  Bedrock-compatible protocol
  Bedrock resource/data readers
  world storage and chunk streaming
```

### Android host

Owns the Activity lifecycle, landscape orientation, immersive mode, OpenGL ES surface and app-level presentation. It forwards surface lifecycle calls through a narrow JNI bridge.

### Book Core

Contains platform-independent data types and deterministic logic. Android APIs, OpenGL calls and network sockets must not leak into this crate.

### Book64

Owns graphics backend selection and rendering contracts. OpenGL ES 3.0 is the first backend. A future Vulkan backend must implement the same renderer-facing contract; adding Vulkan must not require rewriting world simulation.

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

The first milestone contains:
- a landscape-only Android game host;
- an immersive full-screen presentation;
- the user-supplied Book Edition artwork shown in the app;
- a JNI bridge into Rust;
- a Book64 OpenGL ES 3.0 backend that draws a rotating diagnostic cube;
- deterministic Rust tests for chunk-coordinate conversion, including negative world coordinates.

There is no world simulation or Bedrock network implementation in this milestone.
