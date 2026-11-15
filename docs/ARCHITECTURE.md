# Architecture

## Application relationship

The launcher is the single Android application installed by the user. The game runtime is an Android library module packaged into the same APK. The launcher starts `dev.bookedition.game.MainActivity` with an explicit in-app Intent.

```text
Minecraft Book Edition APK
  ├── Launcher Activity (Compose)
  │    ├── Home
  │    ├── Versions
  │    ├── Settings
  │    └── Logs
  └── Game Activity (landscape, immersive)
       ├── Main menu
       │    ├── Play
       │    ├── Settings
       │    └── Exit
       └── Book64 / OpenGL ES 3.0 scene
            └── JNI
                 └── Rust game crates
```

## Game layers

- **Book Core:** platform-independent world coordinates and primitives.
- **Book World:** block identifiers and 16 × 16 × 16 section storage.
- **Book Meshing:** CPU-side mesh construction and shared-face removal.
- **Book64:** renderer boundary; OpenGL ES 3.0 is the first backend.
- **Book Android:** JNI entry points and native renderer lifecycle.

## Runtime rules

- Android UI and Activity lifecycle remain in Kotlin.
- OpenGL ES calls run only on the GLSurfaceView render thread with a current EGL context.
- The renderer must be recreated when Android recreates the EGL context.
- Game logic and world storage must not depend on Android APIs.
- Future Vulkan support must fit behind the Book64 rendering boundary.

## Compatibility direction

The project is an independent client implementation, architecturally inspired by independent clients such as Cinnabar. No Cinnabar source is copied or linked. Bedrock protocol, resource and world formats will be implemented in isolated modules and verified against explicit versioned fixtures before compatibility is claimed.

## Current limitations

The current world is only a generated rendering fixture. Cross-chunk neighbor lookup, persistent storage, player controller, collision, input, texture loading, entities, inventory, networking, authentication and Bedrock compatibility are not implemented.
