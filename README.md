# Minecraft Book Edition

**Minecraft Book Edition** is an independent, open-source voxel sandbox project written in Rust.

Its goal is to build a lightweight, modular game that combines ideas from the Java and Bedrock content ecosystems:

- Bedrock-inspired resource packs and texture atlases.
- A community-led path toward importing selected Java content mods.
- A native Rust engine with a strong focus on performance and low memory use.
- A codebase designed to be easy to understand, extend, and contribute to.

> This is a new, independent implementation. It does not contain or redistribute Minecraft source code or proprietary game assets.

## Project status

The project is at its foundation stage. The current workspace establishes the first reusable pieces of the engine:

- `book-core`: block identifiers, atlas coordinates, and a block registry.
- `book-world`: fixed-size chunk sections and block storage.
- `book-client`: an initial executable entry point.

There is no playable renderer yet. Rendering, input, world generation, saving, networking, and compatibility layers will be developed incrementally.

## Build

Install a stable Rust toolchain, then run:

```sh
cargo build --workspace
cargo test --workspace
cargo run -p book-client
```

The initial core avoids platform-specific graphics dependencies. This makes the data layer easier to build and test across targets, including 32-bit and 64-bit x86 where the Rust toolchain and operating system support them.

## Compatibility strategy

Compatibility is incremental, not automatic. The project will first define its own stable content model. Community contributors can then implement and test importers for Bedrock resource packs, Bedrock add-ons, and selected Java mod content.

Java mods that replace renderer internals or depend on Java-specific engine hooks are outside the initial compatibility target.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## License

The project is licensed under the MIT License. See [LICENSE](LICENSE).
