# Architecture

## Current workspace

```text
crates/
├── book-core/    # Shared identifiers and content definitions
├── book-world/   # Chunk and world data structures
└── book-client/  # Executable entry point
```

## Dependency direction

The intended dependency direction is from higher-level systems toward the small shared core:

```text
book-client
    └── book-world
          └── book-core
```

The core data model must not depend on a renderer, operating system, or mod loader.

## Initial world representation

The first world primitive is a 16 × 16 × 16 chunk section. Each cell stores a compact 16-bit block identifier. A full chunk column, height system, lighting, biome storage, and serialization format will be designed separately.

## Texture atlas direction

The resource pipeline is planned to support Bedrock-style atlas metadata, including named texture entries and tile coordinates. The atlas importer is not implemented yet. Atlas metadata should remain separate from world storage so renderers can consume it without changing block data.

## Rendering direction

The renderer will be selected after a small platform matrix and minimum hardware profile are documented. The project targets x86 and x86_64, but 32-bit graphics-driver and dependency support must be verified independently. No renderer or FPS guarantee is claimed at this stage.

## Compatibility direction

External formats should be imported into a documented Book Edition content API. Compatibility is implemented format by format and mod by mod, with tests and explicit support levels. Arbitrary Java bytecode, mixins, and engine replacement mods are not part of the initial scope.
