use std::path::{Path, PathBuf};

/// Resolve the app-specific Android/media directory from Android's
/// Android/data/<application-id>/files path.
///
/// The package ID is read from the running app's real data path instead of
/// being hard-coded. This also supports side-by-side diagnostic builds whose
/// application ID has a suffix such as ".fullscreenfix".
pub(crate) fn derive_media_root(external_files_path: &Path) -> Option<PathBuf> {
    if external_files_path.file_name()?.to_str()? != "files" {
        return None;
    }

    let package_dir = external_files_path.parent()?;
    let package_id = package_dir.file_name()?.to_str()?;
    if !is_valid_application_id(package_id) {
        return None;
    }

    let data_dir = package_dir.parent()?;
    if data_dir.file_name()?.to_str()? != "data" {
        return None;
    }

    let android_dir = data_dir.parent()?;
    if android_dir.file_name()?.to_str()? != "Android" {
        return None;
    }

    Some(android_dir.join("media").join(package_id))
}

fn is_valid_application_id(value: &str) -> bool {
    let mut segments = value.split('.');
    let mut count = 0;

    for segment in &mut segments {
        if segment.is_empty()
            || !segment
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return false;
        }
        count += 1;
    }

    count >= 2 && value.len() <= 255
}
