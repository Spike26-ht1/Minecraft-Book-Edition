# Booking Engine

Booking is the rendering and world engine of Minecraft Book Edition. The client
owns platform lifecycle, input and presentation; Booking owns world state,
registries, terrain generation, and CPU-side preparation of renderable chunks.

## Current architecture

- **book-core:** stable block identifiers and content definitions.
- **book-world:** chunk columns, vertical sections, loaded-world lookup, and compact palette-backed block-state storage.
- **book-meshing:** CPU geometry generation with neighbor-face culling.
- **booking-engine:** world orchestration, built-in block registry, prototype
  terrain generation, player simulation, versioned world persistence, and
  section-mesh preparation.
- **book-renderer-api:** graphics-backend contract.
- **book-renderer-opengl:** OpenGL / OpenGL ES backend and GPU presentation.
- **book-client:** Android application lifecycle, input event loop, world loading, autosaving, player-centered chunk expansion, and multi-touch mobile controls.

The renderer must consume world meshes prepared by Booking. Terrain generation
and block registry logic must not be duplicated inside graphics backends.

## World storage and streaming

The current prototype generator creates deterministic multi-section terrain
for individual chunk columns. As the player enters a new chunk, Booking ensures
that a radius of nearby prototype chunks is loaded. Distant chunks are still
kept in memory; disk-backed unloading and bounded residency remain planned.

World persistence uses a versioned Booking binary container. Each vertical
section stores its palette, packed indices, and validated dimensions. The file
has a checksum and is written through a temporary sibling file before replace.
The Android client loads the saved world at startup, saves after block edits,
and saves again when the activity is suspended or the event loop exits. Player
position and view direction are stored separately and validated against the
loaded world before restoration. These are project-specific formats and are
not compatible with Bedrock world files.

## Mobile controls

The Android client shares a pixel-coordinate layout between touch hit testing
and the renderer. A left virtual joystick supplies normalized strafe/forward
axes. Four independent right-side buttons provide jump, held sprint, mine, and
place actions. Touch identifiers are routed independently so the movement
stick can remain held while another finger looks around or presses an action.
The visible HUD currently uses generated stone-like pixel panels; it does not
ship Mojang's original UI sprites. Those will be loaded from the local vanilla
resource-pack installation only after the consent-based asset downloader and
texture loader are implemented.

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

The current terrain generator and built-in texture atlas are prototypes. Camera state, player gravity/collision, keyboard movement, touch-look, and basic block interaction are now connected to the renderer. The world format, biome and
noise algorithms, block states, lighting model, resource-pack rendering,
entity pipeline, and Bedrock network protocol are not yet Bedrock-compatible.
Those systems must be implemented and tested independently before claiming
Bedrock world, pack, or server compatibility.
