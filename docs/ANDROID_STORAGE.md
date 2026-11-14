# Android storage layout — Pre-Alpha 1.2.0

The app creates its user-visible storage under `Android/media/com.spike26.minecraftbookedition/`.

```text
game/
├── minecraftWorlds/
├── resource_packs/
├── behavior_packs/
├── development_resource_packs/
├── development_behavior_packs/
├── minecraftpe/
└── logs/
    └── latest.log

launcher/
├── config/
├── profiles/
├── instances/
├── downloads/
├── cache/
└── logs/
    └── launcher.log
```

Both logs receive a timestamped entry whenever the Android native entry point starts. Directories are created at startup. This is only the initial storage foundation: world saving/loading, pack discovery/import, launcher profiles, downloads, and settings persistence are not implemented yet.

The path is derived from AndroidApp's app-specific external files directory and validated before use. If the expected Android directory layout is unavailable or creation fails, the app continues without external storage and reports the failure in Logcat.
