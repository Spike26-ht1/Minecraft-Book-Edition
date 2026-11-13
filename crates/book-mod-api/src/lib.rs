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
    /// High-level script executed by the Book scripting runtime.
    Script,
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
    /// Other mods required by this package. Version strings are interpreted by
    /// the loader's dependency resolver, not by manifest validation.
    #[serde(default)]
    pub dependencies: Vec<ModDependency>,
    #[serde(default)]
    pub requested_capabilities: Vec<Capability>,
}

/// A required or optional dependency on another namespaced mod identifier.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ModDependency {
    pub id: String,
    /// Semantic-version requirement, for example `^1.2`.
    pub version: String,
    #[serde(default)]
    pub optional: bool,
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
    InvalidDependencyId(String),
    EmptyDependencyVersion(String),
    DuplicateDependency(String),
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
            Self::InvalidDependencyId(id) => write!(formatter, "invalid dependency id: {id}"),
            Self::EmptyDependencyVersion(id) => write!(formatter, "dependency version cannot be empty: {id}"),
            Self::DuplicateDependency(id) => write!(formatter, "duplicate dependency: {id}"),
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

        for (index, dependency) in self.dependencies.iter().enumerate() {
            let mut parts = dependency.id.split(':');
            let namespace = parts.next().unwrap_or_default();
            let name = parts.next().unwrap_or_default();
            if !valid_part(namespace) || !valid_part(name) || parts.next().is_some() {
                return Err(ManifestError::InvalidDependencyId(dependency.id.clone()));
            }
            if dependency.version.trim().is_empty() {
                return Err(ManifestError::EmptyDependencyVersion(dependency.id.clone()));
            }
            if self.dependencies[..index].iter().any(|item| item.id == dependency.id) {
                return Err(ManifestError::DuplicateDependency(dependency.id.clone()));
            }
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
            dependencies: vec![ModDependency { id: "book:core".to_owned(), version: "^1.0".to_owned(), optional: false }],
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
    fn rejects_duplicate_dependencies() {
        let mut value = manifest();
        value.dependencies.push(value.dependencies[0].clone());
        assert_eq!(value.validate(), Err(ManifestError::DuplicateDependency("book:core".to_owned())));
    }

    #[test]
    fn rejects_empty_dependency_version() {
        let mut value = manifest();
        value.dependencies[0].version.clear();
        assert_eq!(value.validate(), Err(ManifestError::EmptyDependencyVersion("book:core".to_owned())));
    }

    #[test]
    fn accepts_script_runtime() {
        let mut value = manifest();
        value.runtime = ModRuntime::Script;
        value.entrypoint = "scripts/main.rhai".to_owned();
        assert!(value.validate().is_ok());
    }

    #[test]
    fn serializes_runtime_names_consistently() {
        let json = serde_json::to_string(&manifest()).unwrap();
        assert!(json.contains("\"web_assembly\"") || json.contains("\"webassembly\""));
    }
}
