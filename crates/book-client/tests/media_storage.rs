#[path = "../src/media_storage.rs"]
mod media_storage;

use media_storage::derive_media_root;
use std::path::Path;

#[test]
fn resolves_the_regular_application_media_directory() {
    let external_files = Path::new(
        "/storage/emulated/0/Android/data/com.spike26.minecraftbookedition/files",
    );

    assert_eq!(
        derive_media_root(external_files).as_deref(),
        Some(Path::new(
            "/storage/emulated/0/Android/media/com.spike26.minecraftbookedition"
        ))
    );
}

#[test]
fn resolves_side_by_side_diagnostic_package_ids() {
    let external_files = Path::new(
        "/storage/emulated/0/Android/data/com.spike26.minecraftbookedition.fullscreenfix/files",
    );

    assert_eq!(
        derive_media_root(external_files).as_deref(),
        Some(Path::new(
            "/storage/emulated/0/Android/media/com.spike26.minecraftbookedition.fullscreenfix"
        ))
    );
}

#[test]
fn rejects_paths_outside_the_expected_android_layout() {
    for invalid in [
        "/storage/emulated/0/Android/data/com.example.game",
        "/storage/emulated/0/Android/obb/com.example.game/files",
        "/storage/emulated/0/Android/data/com.example.game/cache",
        "/storage/emulated/0/Android/data/com..example/files",
        "/storage/emulated/0/Android/data/not-a-package/files",
    ] {
        assert!(
            derive_media_root(Path::new(invalid)).is_none(),
            "unexpectedly accepted {invalid}"
        );
    }
}
