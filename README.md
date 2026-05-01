# bevy_settings_plus

Capability-aware extension to [`bevy_settings`](https://docs.rs/bevy_settings)
for [Bevy](https://bevy.org).

`bevy_settings` (in Bevy 0.19+) ships a polished settings persistence kernel:
TOML files, derive-driven `SettingsGroup` resources, change detection,
async + debounced saves. `bevy_settings_plus` layers the *graphics-aware* concerns
on top:

- **Adapter caps detection** — `AdapterCaps` is a stable, plain-data summary
  of the live wgpu adapter (backend, vendor, ray-query support, DLSS
  reachability, …). Detected once and inserted as a main-world resource.
- **Caps clamping** — settings types implement `CapsAware` and register
  `#[reflect(CapsAware)]`. `CapsAwarePlugin` discovers them by reflection
  (mirroring how `bevy_settings` discovers `SettingsGroup` types) and runs
  each type's `clamp_to(caps)` after settings have been loaded from disk.
- **Opinionated graphics schema** — `DisplaySettings`, `RenderSettings`, and
  `AccessibilitySettings` resources with platform-aware defaults and engine
  bindings that drive the primary window and tagged cameras.

## Usage

```rust
use bevy::prelude::*;
use bevy_settings_plus::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SettingsPlusPlugins::new("net.pavlov.my_game"))
        .add_systems(Startup, spawn_camera)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera3d::default(), SettingsCamera));
}
```

The camera marker tells the binding system which cameras to drive. Spawn it
whenever — at `Startup`, on `OnEnter(GameState::Playing)`, after async asset
load — the bindings apply on `Add<SettingsCamera>` so deferred spawns work
without ceremony.

To save user changes (e.g., from an "Apply" button or after dragging a
slider), use `bevy_settings`' commands directly:

```rust
fn save_button(mut commands: Commands) {
    // Debounced — coalesces bursts of changes into one write.
    commands.queue(SavePreferencesDeferred::default());
}

fn save_now(mut commands: Commands) {
    // Async, fire-once.
    commands.queue(SavePreferences::IfChanged);
}
```

For a robust save-on-exit, the upstream pattern is to issue
`SavePreferencesDeferred` whenever the user changes something *and* a
synchronous `SavePreferencesSync::IfChanged` immediately before app exit.

## Extending — your own caps-aware game settings

```rust
use bevy::prelude::*;
use bevy_settings_plus::prelude::*;

#[derive(Resource, SettingsGroup, Reflect, Default)]
#[reflect(Resource, SettingsGroup, CapsAware, Default)]
#[settings_group(group = "my_game")]
struct MyGameQuality {
    ray_tracing: bool,
    shadow_distance: f32,
}

impl CapsAware for MyGameQuality {
    fn clamp_to(&mut self, caps: AdapterCaps) {
        if !caps.ray_query {
            self.ray_tracing = false;
        }
        self.shadow_distance = self.shadow_distance.clamp(10.0, 1000.0);
    }
}

// Then, in your plugin's `build`:
//   app.register_type::<MyGameQuality>();
```

`bevy_settings::PreferencesPlugin` (added by `SettingsPlusPlugins`) discovers
the type from the registry and loads it from `<prefs_dir>/<app>/settings.toml`.
`CapsAwarePlugin` then clamps it after the wgpu adapter is known.

## Cargo features

- `dlss` — wires [DLSS](https://developer.nvidia.com/rtx/dlss) via
  `bevy_anti_alias`. Requires the NVIDIA DLSS SDK at build time; only
  enable for shipping NVIDIA-targeted builds.

## Status

`0.19-dev` is the active branch and tracks Bevy `main` (currently
`0.19.0-dev`). API may change between minor versions.

This crate replaces `bevy_config` (the predecessor on the `main` branch,
which targeted Bevy 0.18.1 with its own persistence kernel before
`bevy_settings` landed upstream).

See `examples/basic.rs` for the minimal setup and
`examples/deferred_camera.rs` for the post-`Startup` spawn pattern.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
