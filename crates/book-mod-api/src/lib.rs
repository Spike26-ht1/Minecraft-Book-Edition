//! Stable, runtime-neutral contracts shared by native and WebAssembly mods.
//!
//! This crate intentionally does not load executable code. Runtime loading,
//! capability grants, and ABI/WASM integration belong to the host-side loader.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Manifest format version understood by this API.
pub const MANIFEST_SCHEMA_VERSION: u32 = 1;

/// Runtime requested by a mod package.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModRuntime {
    /// Native Rust module loaded through a separately versioned host ABI.
    RustNative,
    /// Sandboxed WebAssembly module.
    WebAssembly,
}

/// Capabilities a mod may request. A request is not a grant.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    RegisterBlocks,
    RegisterItems,
    RegisterEntities,
    SubscribeEvents,
    ReadWorld,
    ModifyWorld,
    RenderWorld,
    RegisterUi,
    ReadInput,
    NetworkClient,
    FileRead,
    FileWrite,
}

/// Declarative package metadata shared by both runtime types.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ModManifest {
    pub schema_version: u32,
    /// Stable namespaced identifier, for example `example:minimap`.
    pub id: String,
    pub version: String,
    pub name: String,
    /// Version of the public Book Mod API required by this package.
    pub api_version: String,
    pub runtime: ModRuntime,
    /// Package-relative executable module path, resolved by the runtime host.
    pub entrypoint: String,
    #[serde(default)]
    pub requested_capabilities: Vec<Capability>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ManifestError {
    UnsupportedSchema(u32),
    InvalidId,
    EmptyVersion,
    EmptyName,
    EmptyApiVersion,
    EmptyEntrypoint,
    DuplicateCapability(Capability),
}

impl fmt::Display for ManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchema(version) => write!(formatter, "unsupported manifest schema: {version}"),
            Self::InvalidId => formatter.write_str("mod id must use the namespace:name format"),
            Self::EmptyVersion => formatter.write_str("mod version cannot be empty"),
            Self::EmptyName => formatter.write_str("mod name cannot be empty"),
            Self::EmptyApiVersion => formatter.write_str("API version cannot be empty"),
            Self::EmptyEntrypoint => formatter.write_str("entrypoint cannot be empty"),
            Self::DuplicateCapability(capability) => write!(formatter, "duplicate capability: {capability:?}"),
        }
    }
}

impl std::error::Error for ManifestError {}

impl ModManifest {
    /// Checks manifest structure only. Compatibility resolution and permission
    /// grants must be performed by the loader before starting a runtime.
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.schema_version != MANIFEST_SCHEMA_VERSION {
            return Err(ManifestError::UnsupportedSchema(self.schema_version));
        }

        let mut parts = self.id.split(':');
        let namespace = parts.next().unwrap_or_default();
        let name = parts.next().unwrap_or_default();
        let valid_part = |part: &str| {
            !part.is_empty()
                && part.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || byte == b'_'
                        || byte == b'-'
                        || byte == b'.'
                })
        };
        if !valid_part(namespace) || !valid_part(name) || parts.next().is_some() {
            return Err(ManifestError::InvalidId);
        }
        if self.version.trim().is_empty() {
            return Err(ManifestError::EmptyVersion);
        }
        if self.name.trim().is_empty() {
            return Err(ManifestError::EmptyName);
        }
        if self.api_version.trim().is_empty() {
            return Err(ManifestError::EmptyApiVersion);
        }
        if self.entrypoint.trim().is_empty() {
            return Err(ManifestError::EmptyEntrypoint);
        }

        for (index, capability) in self.requested_capabilities.iter().enumerate() {
            if self.requested_capabilities[..index].contains(capability) {
                return Err(ManifestError::DuplicateCapability(*capability));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> ModManifest {
        ModManifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            id: "example:minimap".to_owned(),
            version: "1.0.0".to_owned(),
            name: "Example Minimap".to_owned(),
            api_version: "0.1".to_owned(),
            runtime: ModRuntime::WebAssembly,
            entrypoint: "modules/minimap.wasm".to_owned(),
            requested_capabilities: vec![Capability::ReadWorld, Capability::RegisterUi],
        }
    }

    #[test]
    fn accepts_a_valid_wasm_manifest() {
        assert!(manifest().validate().is_ok());
    }

    #[test]
    fn rejects_invalid_namespaced_ids() {
        let mut value = manifest();
        value.id = "Minimap".to_owned();
        assert_eq!(value.validate(), Err(ManifestError::InvalidId));
    }

    #[test]
    fn rejects_duplicate_capability_requests() {
        let mut value = manifest();
        value.requested_capabilities.push(Capability::ReadWorld);
        assert_eq!(
            value.validate(),
            Err(ManifestError::DuplicateCapability(Capability::ReadWorld))
        );
    }

    #[test]
    fn rejects_unknown_manifest_schema() {
        let mut value = manifest();
        value.schema_version = MANIFEST_SCHEMA_VERSION + 1;
        assert_eq!(
            value.validate(),
            Err(ManifestError::UnsupportedSchema(MANIFEST_SCHEMA_VERSION + 1))
        );
    }

    #[test]
    fn serializes_runtime_names_consistently() {
        let json = serde_json::to_string(&manifest()).unwrap();
        assert!(json.contains("\"web_assembly\"") || json.contains("\"webassembly\""));
    }
}
