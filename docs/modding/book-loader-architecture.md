# Book Loader architecture

Book Loader is the Fabric-inspired mod loader for Minecraft Book Edition. Mods are
independent packages with stable namespaced IDs, explicit API compatibility,
dependency declarations, and capability requests.

## Package layout

A mod package uses the `.bookmod` extension and contains a `book.mod.json`
manifest plus its runtime entrypoint and optional assets:

```text
example-mod.bookmod/
  book.mod.json
  native/             # optional platform-specific native module
  scripts/            # optional high-level scripts
  assets/
  config/
```

The package format is a directory layout specification at this stage; archive
packing and signature verification are not implemented yet.

## Runtime model

- **Rust Native:** advanced extensions compiled for a specific Book Loader host
  ABI and target platform. Native modules are trusted code, not sandboxed.
- **Script:** high-level scripts run by a future embedded scripting runtime.
  Scripts receive only capabilities granted by the loader.
- **WebAssembly:** portable sandboxed modules. WASM is an execution target and
  isolation boundary, not a replacement for the scripting API.

All runtimes use the same `book-mod-api` manifest, namespaced identifiers,
dependency metadata, lifecycle concepts, registries, and event contracts.
The loader must resolve dependencies and permissions before executing any mod.

## Lifecycle

The planned lifecycle is:

1. Discover packages.
2. Parse and validate manifests.
3. Check Book Loader and Book Mod API compatibility.
4. Resolve required/optional dependencies and detect cycles.
5. Present requested capabilities for approval.
6. Load runtimes and invoke initialization in dependency order.
7. Register content and event handlers.
8. Invoke shutdown hooks and release runtime resources.

## Security and compatibility

Native Rust modules have full process privileges and must be treated as trusted,
platform-specific code. They require an explicit ABI version and matching target
build. Scripts and WASM should use capability-limited host APIs. A capability
request is never an automatic grant.

The current `book-mod-api` crate provides shared manifest types and structural
validation only. It does not yet discover packages, execute scripts, load native
libraries, resolve semantic-version ranges, or implement registries/events.


## Rust mod API surface

The shared API now defines a source-level Rust mod lifecycle:

- `BookMod::on_load` and `BookMod::on_unload` for initialization and cleanup.
- `BookMod::subscriptions` to declare event kinds the mod wants to receive.
- `BookMod::on_event` for client, world, tick, and block lifecycle events.
- `ModContext` for capability-checked block registration and world block access.

A host implementation of `ModContext` must enforce the owning mod's granted
capabilities on every operation. The API contract itself does not grant
permissions or expose unrestricted engine internals.

This Rust trait is a source-level API, not a stable ABI for independently
compiled dynamic libraries. Native modules still need a versioned host ABI;
WASM and script runtimes should call equivalent host functions through
capability-limited adapters. The loader/runtime host and concrete context
implementation are not yet implemented.
