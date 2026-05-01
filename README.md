# bevy_config

Capability-aware configuration system for [Bevy](https://bevy.org).

The shape: **detect adapter caps → pick platform default → layer user overrides → bind to engine resources**.

A single hardcoded set of graphics defaults is wrong on at least one platform. Wasm/WebGPU first-time visitors should get conservative defaults. Desktop with an RT-capable GPU should get the full picture. Adapter features (ray query, acceleration structures, DLSS support) decide whether features are even *available* before user preference comes in.

`bevy_config` solves this with a small kernel:

- `AdapterCaps` — wgpu features and limits → stable, plain bools/ints/enums in the main world
- `PlatformTarget` — Wasm / Windows / Linux / macOS / Other
- `Config` trait — `platform_default(target)`, `merge(&overrides)`, `clamp_to(&caps)`, `current_overrides()`
- `ConfigBackend` trait — load/store; default impls for native RON files (atomic write) and wasm `localStorage`
- `ConfigPlugin<C>` — wires the lifecycle: load → merge → clamp → apply, on top of `Plugin::build`/`finish`

…plus an opinionated universal schema (`CommonConfig`) covering display, render, and accessibility, and engine bindings for the graphics axes (window mode, anti-alias, MSAA, DLSS — feature-gated).

Audio configuration is intentionally not part of `CommonConfig` because the bus convention is backend-specific (firewheel/seedling/kira/oddio all model differently). When the firewheel/seedling integration lands it will ship as its own `Config` type behind a cargo feature, registered alongside `CommonConfig` via a separate `ConfigPlugin`.

Game-specific quality dials use the same `Config` trait with a custom type and a separate `ConfigPlugin<MyGameConfig>` registration.

## Status

`v0.1.x` — API may change between minor versions. Tracks Bevy `main` until the next Bevy release; will move to versioned deps when that lands.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
