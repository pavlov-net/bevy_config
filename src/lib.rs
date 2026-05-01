//! Capability-aware configuration system for Bevy.
//!
//! The model: **detect adapter caps → pick platform default → merge user
//! overrides → clamp to caps → apply to engine resources.**
//!
//! ## Modules
//!
//! - [`caps`] — [`AdapterCaps`](caps::AdapterCaps) and [`PlatformTarget`](caps::PlatformTarget)
//! - [`config`] — the [`Config`](config::Config) trait and lifecycle event
//! - [`backend`] — [`ConfigBackend`](backend::ConfigBackend) trait + native/wasm impls
//! - [`plugin`] — [`ConfigPlugin<C>`](plugin::ConfigPlugin) wiring + [`SaveConfig`](plugin::SaveConfig) command
//! - [`common`] — universal schema [`CommonConfig`](common::CommonConfig) (display, render, accessibility)
//! - [`bindings`] — [`CommonBindingsPlugin`](bindings::CommonBindingsPlugin) and [`BevyConfigCamera`](bindings::BevyConfigCamera)
//!
//! Most consumers `use bevy_config::prelude::*;` and reach for everything from there.
//!
//! ## Quickstart
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_config::prelude::*;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(ConfigPlugin::<CommonConfig>::new(
//!         FileBackend::<CommonConfig>::new("com", "example", "my_game"),
//!     ))
//!     .add_plugins(CommonBindingsPlugin)
//!     .add_systems(Startup, |mut commands: Commands| {
//!         commands.spawn((Camera3d::default(), BevyConfigCamera));
//!     })
//!     .run();
//! ```
//!
//! ## Cargo features
//!
//! - `ron` (default) — RON serialisation for the native [`FileBackend`](backend::FileBackend).
//! - `dlss` — wires [DLSS](https://developer.nvidia.com/rtx/dlss) via `bevy_anti_alias`. Requires the NVIDIA DLSS SDK at build time; only enable for shipping NVIDIA-targeted builds.

pub mod backend;
pub mod bindings;
pub mod caps;
pub mod common;
pub mod config;
pub mod plugin;

pub mod prelude {
    //! Re-exports of the most commonly used types.

    #[cfg(not(target_family = "wasm"))]
    pub use crate::backend::FileBackend;
    #[cfg(target_family = "wasm")]
    pub use crate::backend::LocalStorageBackend;
    pub use crate::backend::{BackendError, ConfigBackend};
    pub use crate::bindings::{BevyConfigCamera, CommonBindingsPlugin};
    pub use crate::caps::{AdapterCaps, Backend, GpuVendor, PlatformTarget};
    pub use crate::common::{
        Accessibility, AccessibilityOverrides, AntiAlias, ColorBlindMode, CommonConfig,
        CommonConfigOverrides, Display, DisplayOverrides, FpsCap, HdrPreference, MonitorSelection,
        MsaaLevel, Render, RenderOverrides, Resolution, Upscaler, UpscalerPreset, VsyncMode,
        WindowMode,
    };
    pub use crate::config::{BevyConfigSet, Config, ConfigApplied};
    pub use crate::plugin::{ConfigPlugin, SaveConfig};
}
