//! Capability-aware configuration system for Bevy.
//!
//! See the crate-level README for the model: detect adapter caps → platform default
//! → merge user overrides → clamp to caps → apply to engine resources.

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
        Accessibility, AccessibilityOverrides, AntiAlias, Audio, AudioOverrides, ChannelMode,
        ColorBlindMode, CommonConfig, CommonConfigOverrides, Display, DisplayOverrides, FpsCap,
        HdrPreference, MonitorSelection, MsaaLevel, Render, RenderOverrides, Resolution, Upscaler,
        UpscalerPreset, VsyncMode, WindowMode,
    };
    pub use crate::config::{BevyConfigSet, Config, ConfigApplied};
    pub use crate::plugin::{ConfigPlugin, SaveConfig};
}
