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

#[cfg(test)]
mod tests {
    use super::*;

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
