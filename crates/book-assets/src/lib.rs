use serde::Deserialize;
use std::collections::BTreeMap;

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

impl TerrainTextureManifest {
    pub fn from_json(source: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(source)
    }

    pub fn texture(&self, name: &str) -> Option<&TextureEntry> {
        self.texture_data.get(name)
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
