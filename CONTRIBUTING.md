# Contributing

Minecraft Book Edition is intended to be community-driven.

## Before opening a pull request

1. Check existing issues and discussions to avoid duplicate work.
2. Keep changes focused and explain the reason for them.
3. Add tests for new core behavior.
4. Run:

   ```sh
   cargo fmt --all -- --check
   cargo test --workspace
   ```

5. Document public APIs and compatibility limitations.

## Design principles

- Keep the engine modular.
- Prefer safe Rust; unsafe code is forbidden in the workspace by default.
- Avoid unnecessary dependencies in foundational crates.
- Do not add proprietary Minecraft code, assets, or decompiled content.
- Do not claim compatibility with a mod or pack until it has been tested.

## Suggested contribution areas

- Core block and item data model.
- Chunk storage and meshing.
- Bedrock texture-atlas importer.
- Resource-pack metadata parser.
- Renderer backends and platform support.
- World persistence and generation.
- Java content-mod importers.
