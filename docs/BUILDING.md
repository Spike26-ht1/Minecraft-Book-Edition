# Building the game test app

## Requirements

- JDK 17.
- Android SDK with API 35.
- Android NDK 27.2.12479018 (or a compatible NDK).
- Rust stable with target `aarch64-linux-android`.
- `cargo-ndk`.
- Gradle 8.11.1.

## Rust checks

From `game/native`:

```sh
cargo test --workspace
cargo fmt --all -- --check
```

## Android ARM64 test APK

From the repository root:

```sh
cd game/native
cargo ndk -t arm64-v8a -o ../android/app/src/main/jniLibs build --release -p book-android
cd ../android
gradle assembleDebug
```

The APK is written to:

`game/android/app/build/outputs/apk/debug/app-debug.apk`

The app is a landscape-only fullscreen graphics/world prototype. It currently renders a generated voxel test section. It is not yet a Bedrock-compatible client and does not load proprietary Minecraft assets.

## Device validation checklist

1. Install the debug APK on an ARM64 Android device with OpenGL ES 3.0 support.
2. Confirm that the app opens in landscape and hides the system bars.
3. Confirm that the Book Edition mark appears in the upper-left panel.
4. Confirm that the voxel terrain is visible and rotates smoothly.
5. Pause and resume the app; confirm that the surface is recreated without a crash.
6. Record device model, Android version, frame rate and any Book64/book_android errors.
