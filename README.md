# bevy_config

Capability-aware configuration system for [Bevy](https://bevy.org).

The shape: **detect adapter caps → pick platform default → layer user overrides → bind to engine resources**.

A single hardcoded set of graphics defaults is wrong on at least one platform. Wasm/WebGPU first-time visitors should get conservative defaults. Desktop with an RT-capable GPU should get the full picture. Adapter features (ray query, DLSS support, vendor, backend) decide whether features are even *available* before user preference comes in.

`bevy_config` solves this with a small kernel:

- `AdapterCaps` — wgpu features and limits → stable, plain bools/ints/enums in the main world
- `PlatformTarget` — Wasm / Windows / Linux / macOS / Other
- `Config` trait — `platform_default(target)`, `merge(&overrides)`, `clamp_to(&caps)`, `current_overrides()`
- `ConfigBackend` trait — load/store; default impls for native RON files (atomic write) and wasm `localStorage`
- `ConfigPlugin<C>` — wires the lifecycle: load → merge → clamp → apply, on top of `Plugin::build`/`finish`

…plus an opinionated universal schema (`CommonConfig`) covering display, render, and accessibility, and engine bindings for the graphics axes (window mode, anti-alias, MSAA, DLSS — feature-gated).

Audio configuration is intentionally not part of `CommonConfig` because the bus convention is backend-specific (firewheel/seedling/kira/oddio all model differently). When the firewheel/seedling integration lands it will ship as its own `Config` type behind a cargo feature, registered alongside `CommonConfig` via a separate `ConfigPlugin`.

Game-specific quality dials use the same `Config` trait with a custom type and a separate `ConfigPlugin<MyGameConfig>` registration.

## Usage

```rust
use bevy::prelude::*;
use bevy_config::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ConfigPlugin::<CommonConfig>::new(
            FileBackend::<CommonConfig>::new("com", "example", "my_game"),
        ))
        .add_plugins(CommonBindingsPlugin)
        .add_systems(Startup, spawn_camera)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera3d::default(), BevyConfigCamera));
}
```

The camera marker tells the binding system which cameras to drive. Spawn it whenever — at `Startup`, on `OnEnter(GameState::Playing)`, after async asset load — the bindings apply on `Add<BevyConfigCamera>` so deferred spawns work without ceremony.

To save a user's settings (e.g., from an "Apply" button in a menu):

```rust
fn save_button(mut commands: Commands) {
    commands.queue(SaveConfig::<CommonConfig>::default());
}
```

See `examples/basic.rs` for the minimal setup and `examples/deferred_camera.rs` for the post-`Startup` spawn pattern.

## Status

You're on the `0.19-dev` branch — tracks Bevy `main` (currently `0.19.0-dev`) for the next-release line. The published `v0.1.x` line on `main` targets Bevy 0.18.1; that's what `crates.io` ships and what consumers should pin against until the next minor release. API may change between minor versions on either line.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
