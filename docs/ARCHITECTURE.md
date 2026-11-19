# Architecture

## Product boundary

Book Edition is an independent Bedrock-compatible client being reimplemented in Rust. The Android application is a single APK containing the Kotlin launcher and the Rust game runtime. The launcher does not start or inject into the official Minecraft runtime.

## Android application relationship

```text
Book Edition APK
  ├── Launcher Activity (Kotlin / Compose)
  │    ├── Home and game entry
  │    ├── Settings
  │    ├── Version / content management
  │    └── Local logs and export
  └── Game Activity (landscape)
       ├── Game menu and touch HUD
       └── JNI runtime host
            ├── book-game: authoritative single-player session
            │    └── book-world: loaded block data
            └── book64-gles: presentation backend
                 └── book-meshing: world-to-mesh extraction
```

## Rust boundaries

- **book-core:** shared coordinates and low-level contracts.
- **book-world:** block identifiers, chunk sections, loaded columns, global block lookup, terrain generation and a versioned binary save codec. World data is independent of Android and rendering.
- **book-game:** session lifecycle, player state, input intent, fixed-step simulation, movement, gravity, jump, collision and first-person block interaction. It owns the authoritative state for the current single-player session.
- **book-meshing:** converts a world snapshot into visible geometry and removes shared faces. It does not own or mutate the world.
- **Book64:** uploads geometry and renders immutable `RenderState` snapshots. It does not own player movement, physics, world generation or input.
- **book-android:** owns the native runtime composition and exposes a narrow JNI boundary. It holds the game session, renderer and local world-save path separately.
- **Kotlin launcher/game host:** handles Android lifecycle, user interface, touch input, file selection and local logs. It does not implement game rules.

Future boundaries will be added when the corresponding systems exist: `book-save`, `book-assets`, `book-streaming`, `book-protocol` and client presentation/UI modules.

## Runtime rules

- Android UI and Activity lifecycle remain in Kotlin.
- OpenGL ES calls run only on the GLSurfaceView render thread with a current EGL context.
- The Android JNI host owns a `GameSession` and a `GlesRenderer` as separate objects.
- Game simulation advances on a fixed 20 Hz clock; rendering can run at the device's display rate.
- Long frame stalls are bounded so the simulation cannot enter an unbounded catch-up loop.
- Book64 receives a snapshot produced by the simulation and never mutates gameplay state.
- The renderer must be recreated when Android recreates the EGL context.
- Game logic and world storage must not depend on Android APIs.
- Future Vulkan support must fit behind the Book64 rendering boundary.

## World data flow

```text
Seed + version
    ↓
book-world (block columns / sections)
    ↓ read
book-game (player, collision, rules, fixed ticks)
    ↓ immutable RenderState
Book64 ← book-meshing ← world snapshot
    ↓
OpenGL ES frame
```

The initial mesh is uploaded at renderer initialization. Mining and placement mark it dirty; Book64 rebuilds and uploads it on the GL render thread. The world remains a bounded 5 × 5 loaded region. Local save files preserve the loaded block sections, seed and generator version; player position, inventory, streaming and infinite-world loading are not yet implemented.

## Compatibility and validation

The project targets Bedrock 26.60, but the current terrain generator is Book Edition's own version 1 and is not a Bedrock seed port. Bedrock protocol, resource and world formats will be implemented in isolated versioned modules.

Every compatibility area needs fixtures and tests that compare observable results with legitimate reference data. Compilation and the existence of a rendered scene are not evidence of Bedrock parity.

## References and licensing

See [REFERENCE_STUDY.md](REFERENCE_STUDY.md) for the architecture research and license review. Horizon, Cinnabar and LeviLauncher are reference projects only; their code is not copied into Book Edition.
