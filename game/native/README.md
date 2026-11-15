# Native game workspace

This directory contains the platform-independent Rust game code and the Android JNI bridge.

## Crates

- `book-core`: shared coordinates and game primitives.
- `book-world`: block identifiers and 16 × 16 × 16 chunk-section storage.
- `book-meshing`: CPU mesh generation with shared-face removal.
- `book64-gles`: Book64 OpenGL ES 3.0 backend.
- `book-android`: Android JNI entry points that bind the host surface to Book64.

## Checks

```sh
cargo test --workspace
cargo fmt --all -- --check
```

The Android host supplies the current EGL context. GLES calls must remain on the GLSurfaceView render thread. Do not move renderer calls to the UI thread.
