# Build instructions

## Requirements

- JDK 17.
- Android SDK API 35 and build tools 35.0.0.
- Android NDK 27.2.12479018.
- Rust stable with target `aarch64-linux-android`.
- `cargo-ndk`.
- Gradle 8.11.1.

## Rust tests

From `game/native`:

```sh
cargo test --workspace
cargo fmt --all -- --check
```

## Build the integrated launcher APK

From the repository root, first build the Rust JNI library:

```sh
cd game/native
cargo ndk -t arm64-v8a -o ../android/app/src/main/jniLibs build --release -p book-android
cd ../..
gradle --no-daemon :launcher:app:assembleDebug
```

The combined APK is written to:

`launcher/app/build/outputs/apk/debug/app-debug.apk`

The APK contains the launcher and embedded game runtime. It is not necessary to install a separate game APK.

## Device validation

1. Install the combined debug APK on an ARM64 Android device with OpenGL ES 3.0 support.
2. Open the launcher and press **Jogar**.
3. Confirm the game menu appears.
4. Press **JOGAR** and confirm the voxel test terrain is rendered.
5. Return to the game menu and back to the launcher.
6. Pause/resume and rotate/lifecycle test while verifying landscape remains enforced.
7. Record device model, Android version, frame rate and any Book64/book_android errors.

A successful Gradle build is necessary but does not replace physical-device validation.
