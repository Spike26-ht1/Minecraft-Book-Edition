use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Parsed subset of a Bedrock-style terrain_texture.json file.
/// Texture declarations are retained as JSON because Bedrock supports multiple shapes.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct TerrainTextureManifest {
    #[serde(default)]
    pub resource_pack_name: Option<String>,
    #[serde(default)]
    pub texture_name: Option<String>,
    #[serde(default)]
    pub padding: Option<u32>,
    #[serde(default)]
    pub num_mip_levels: Option<u32>,
    #[serde(default)]
    pub texture_data: BTreeMap<String, TextureEntry>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct TextureEntry {
    pub textures: serde_json::Value,
}

impl TextureEntry {
    pub fn primary_path(&self) -> Option<&str> {
        match &self.textures {
            serde_json::Value::String(path) => Some(path),
            serde_json::Value::Object(object) => object.get("path")?.as_str(),
            serde_json::Value::Array(items) => items.first().and_then(|item| {
                item.as_str().or_else(|| item.get("path").and_then(serde_json::Value::as_str))
            }),
            _ => None,
        }
    }
}

impl TerrainTextureManifest {
    pub fn from_json(source: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(source)
    }

    pub fn texture(&self, name: &str) -> Option<&TextureEntry> {
        self.texture_data.get(name)
    }

    /// Resolves the common Bedrock terrain_texture declaration shapes to a
    /// resource path. Complex variant arrays remain available through texture().
    pub fn texture_path(&self, name: &str) -> Option<&str> {
        self.texture(name)?.primary_path()
    }
}


/// Decoded RGBA image supplied by a resource-pack or built-in asset provider.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextureImage {
    pub key: String,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextureDecodeError {
    pub message: String,
}

impl TextureImage {
    /// Decodes a PNG resource-pack image into normalized RGBA8 pixels.
    pub fn from_png(key: impl Into<String>, bytes: &[u8]) -> Result<Self, TextureDecodeError> {
        let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
            .map_err(|error| TextureDecodeError { message: error.to_string() })?
            .to_rgba8();
        let (width, height) = image.dimensions();
        Ok(Self {
            key: key.into(),
            width,
            height,
            rgba: image.into_raw(),
        })
    }
}

/// Errors encountered while loading a Bedrock resource-pack terrain atlas.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourcePackError {
    pub message: String,
}

impl std::fmt::Display for ResourcePackError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ResourcePackError {}

/// Loads the prototype grass/dirt/stone tiles from an extracted Bedrock resource pack.
/// The supplied directory may be a pack root or a resource_packs directory containing
/// one-level-deep extracted packs. ZIP/MC_PACK extraction is intentionally handled by
/// the launcher, not by this filesystem reader.
pub fn load_bedrock_terrain_atlas(root: &Path) -> Result<TextureAtlas, ResourcePackError> {
    let mut candidates = vec![root.to_path_buf()];
    let entries = fs::read_dir(root).map_err(|error| ResourcePackError {
        message: format!("Cannot read resource-pack directory {}: {error}", root.display()),
    })?;
    for entry in entries.flatten() {
        if entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            candidates.push(entry.path());
        }
    }

    let mut failures = Vec::new();
    for pack_root in candidates {
        let manifest_path = pack_root.join("textures").join("terrain_texture.json");
        let manifest_path = if manifest_path.is_file() {
            manifest_path
        } else {
            let direct = pack_root.join("terrain_texture.json");
            if direct.is_file() { direct } else { continue; }
        };
        match load_atlas_from_manifest(&pack_root, &manifest_path) {
            Ok(atlas) => return Ok(atlas),
            Err(error) => failures.push(error.message),
        }
    }

    Err(ResourcePackError {
        message: if failures.is_empty() {
            format!("No terrain_texture.json found in {} or its immediate pack folders", root.display())
        } else {
            failures.join("; ")
        },
    })
}

fn load_atlas_from_manifest(root: &Path, manifest_path: &Path) -> Result<TextureAtlas, ResourcePackError> {
    let json = fs::read_to_string(manifest_path).map_err(|error| ResourcePackError {
        message: format!("Cannot read {}: {error}", manifest_path.display()),
    })?;
    let manifest = TerrainTextureManifest::from_json(&json).map_err(|error| ResourcePackError {
        message: format!("Invalid terrain_texture.json: {error}"),
    })?;

    let mut images = Vec::with_capacity(3);
    for (key, aliases) in [
        ("book:grass_block", &["grass_top", "grass", "grass_block"][..]),
        ("book:dirt", &["dirt"][..]),
        ("book:stone", &["stone"][..]),
    ] {
        let entry = aliases.iter().find_map(|alias| manifest.texture_path(alias).map(|path| (*alias, path)))
            .ok_or_else(|| ResourcePackError { message: format!("Required terrain texture missing: {}", aliases.join(" / ")) })?;
        let relative = Path::new(entry.1);
        if relative.is_absolute() || relative.components().any(|part| matches!(part, Component::ParentDir | Component::RootDir | Component::Prefix(_))) {
            return Err(ResourcePackError { message: format!("Unsafe resource-pack texture path: {}", entry.1) });
        }
        let mut texture_path = root.join(relative);
        if texture_path.extension().is_none() { texture_path.set_extension("png"); }
        let bytes = fs::read(&texture_path).map_err(|error| ResourcePackError {
            message: format!("Cannot read texture {}: {error}", texture_path.display()),
        })?;
        let image = TextureImage::from_png(key, &bytes).map_err(|error| ResourcePackError {
            message: format!("Cannot decode texture {}: {}", texture_path.display(), error.message),
        })?;
        images.push(image);
    }

    TextureAtlas::build_grid(images, 16, 16).map_err(|error| ResourcePackError {
        message: format!("Could not build Bedrock terrain atlas: {error:?}"),
    })
}

/// CPU-side atlas data. The graphics backend is responsible for uploading it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextureAtlas {
    pub width: u32,
    pub height: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub columns: u16,
    pub rows: u16,
    pub rgba: Vec<u8>,
    regions: BTreeMap<String, book_core::AtlasTile>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextureAtlasError {
    Empty,
    InvalidGrid,
    TooManyTextures,
    InvalidDimensions(String),
    InvalidPixelLength(String),
    DuplicateKey(String),
    SizeOverflow,
}

impl TextureAtlas {
    /// Packs images into a fixed tile grid. Unused cells remain transparent.
    /// All source images are placed at the top-left of equally sized cells.
    pub fn build_grid(
        images: Vec<TextureImage>,
        columns: u16,
        rows: u16,
    ) -> Result<Self, TextureAtlasError> {
        if images.is_empty() { return Err(TextureAtlasError::Empty); }
        if columns == 0 || rows == 0 { return Err(TextureAtlasError::InvalidGrid); }
        if images.len() > columns as usize * rows as usize {
            return Err(TextureAtlasError::TooManyTextures);
        }

        let mut tile_width = 0_u32;
        let mut tile_height = 0_u32;
        let mut seen = BTreeMap::new();
        for image in &images {
            if image.width == 0 || image.height == 0 {
                return Err(TextureAtlasError::InvalidDimensions(image.key.clone()));
            }
            let expected = (image.width as usize)
                .checked_mul(image.height as usize)
                .and_then(|value| value.checked_mul(4))
                .ok_or(TextureAtlasError::SizeOverflow)?;
            if image.rgba.len() != expected {
                return Err(TextureAtlasError::InvalidPixelLength(image.key.clone()));
            }
            if seen.insert(image.key.clone(), ()).is_some() {
                return Err(TextureAtlasError::DuplicateKey(image.key.clone()));
            }
            tile_width = tile_width.max(image.width);
            tile_height = tile_height.max(image.height);
        }

        let width = tile_width.checked_mul(columns as u32).ok_or(TextureAtlasError::SizeOverflow)?;
        let height = tile_height.checked_mul(rows as u32).ok_or(TextureAtlasError::SizeOverflow)?;
        let byte_len = (width as usize)
            .checked_mul(height as usize)
            .and_then(|value| value.checked_mul(4))
            .ok_or(TextureAtlasError::SizeOverflow)?;
        let mut rgba = vec![0_u8; byte_len];
        let mut regions = BTreeMap::new();

        for (index, image) in images.iter().enumerate() {
            let tile_x = index % columns as usize;
            let tile_y = index / columns as usize;
            let offset_x = tile_x as u32 * tile_width;
            let offset_y = tile_y as u32 * tile_height;

            for row in 0..image.height {
                let source_start = row as usize * image.width as usize * 4;
                let source_end = source_start + image.width as usize * 4;
                let destination_start = ((offset_y as usize + row as usize) * width as usize
                    + offset_x as usize) * 4;
                let destination_end = destination_start + image.width as usize * 4;
                rgba[destination_start..destination_end]
                    .copy_from_slice(&image.rgba[source_start..source_end]);
            }

            regions.insert(
                image.key.clone(),
                book_core::AtlasTile { x: tile_x as u16, y: tile_y as u16 },
            );
        }

        Ok(Self { width, height, tile_width, tile_height, columns, rows, rgba, regions })
    }

    pub fn region(&self, key: &str) -> Option<book_core::AtlasTile> {
        self.regions.get(key).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn png_decoder_rejects_invalid_resource_data() {
        let error = TextureImage::from_png("example:invalid", b"not a png").unwrap_err();
        assert!(!error.message.is_empty());
    }

    #[test]
    fn atlas_builder_packs_images_into_fixed_grid() {
        let red = TextureImage {
            key: "book:red".to_owned(), width: 1, height: 1,
            rgba: vec![255, 0, 0, 255],
        };
        let green = TextureImage {
            key: "book:green".to_owned(), width: 1, height: 1,
            rgba: vec![0, 255, 0, 255],
        };
        let atlas = TextureAtlas::build_grid(vec![red, green], 4, 4).unwrap();
        assert_eq!((atlas.width, atlas.height), (4, 4));
        assert_eq!(atlas.region("book:red"), Some(book_core::AtlasTile { x: 0, y: 0 }));
        assert_eq!(atlas.region("book:green"), Some(book_core::AtlasTile { x: 1, y: 0 }));
    }

    #[test]
    fn atlas_builder_rejects_invalid_pixel_data() {
        let image = TextureImage {
            key: "book:broken".to_owned(), width: 2, height: 2,
            rgba: vec![0, 0, 0, 255],
        };
        assert_eq!(
            TextureAtlas::build_grid(vec![image], 1, 1),
            Err(TextureAtlasError::InvalidPixelLength("book:broken".to_owned()))
        );
    }

    #[test]
    fn resolves_common_bedrock_texture_path_shapes() {
        let json = r#"{
            "texture_data": {
                "stone": { "textures": "textures/blocks/stone" },
                "grass": { "textures": { "path": "textures/blocks/grass_top" } },
                "sand": { "textures": [{ "path": "textures/blocks/sand" }] }
            }
        }"#;
        let manifest = TerrainTextureManifest::from_json(json).unwrap();
        assert_eq!(manifest.texture_path("stone"), Some("textures/blocks/stone"));
        assert_eq!(manifest.texture_path("grass"), Some("textures/blocks/grass_top"));
        assert_eq!(manifest.texture_path("sand"), Some("textures/blocks/sand"));
    }

    #[test]
    fn parses_simple_texture_entry() {
        let json = r#"{
            "resource_pack_name": "Example",
            "texture_name": "atlas.terrain",
            "texture_data": {
                "stone": { "textures": "textures/blocks/stone" }
            }
        }"#;

        let manifest = TerrainTextureManifest::from_json(json).unwrap();

        assert_eq!(manifest.resource_pack_name.as_deref(), Some("Example"));
        assert_eq!(
            manifest.texture("stone").unwrap().textures,
            serde_json::json!("textures/blocks/stone")
        );
    }

    #[test]
    fn preserves_object_texture_declarations() {
        let json = r#"{
            "texture_data": {
                "grass": {
                    "textures": { "path": "textures/blocks/grass_top" }
                }
            }
        }"#;

        let manifest = TerrainTextureManifest::from_json(json).unwrap();

        assert_eq!(
            manifest.texture("grass").unwrap().textures["path"],
            "textures/blocks/grass_top"
        );
    }
}
