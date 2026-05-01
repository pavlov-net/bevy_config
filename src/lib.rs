//! Capability-aware extension to [`bevy_settings`].
//!
//! `bevy_settings` already provides a polished settings persistence kernel
//! (TOML files, change detection, async + debounced saves). This crate
//! layers the *graphics-aware* concerns on top:
//!
//! - **Adapter caps detection** — [`AdapterCaps`](caps::AdapterCaps) is a
//!   stable, plain-data summary of the live wgpu adapter (backend, vendor,
//!   ray-query support, DLSS reachability, …). It is detected once during
//!   [`Plugin::finish`](bevy_app::Plugin::finish) on
//!   [`CapsAwarePlugin`] and inserted as a main-world resource.
//! - **Caps clamping** — settings types implement [`CapsAware`] and
//!   register `#[reflect(CapsAware)]`. [`CapsAwarePlugin`] discovers them by
//!   reflection and runs each type's `clamp_to(&caps)` after settings have
//!   been loaded.
//! - **Opinionated graphics schema** — [`DisplaySettings`](common::DisplaySettings),
//!   [`RenderSettings`](common::RenderSettings), and [`AccessibilitySettings`](common::AccessibilitySettings)
//!   resources with platform-aware defaults and the engine bindings that
//!   drive the primary window and tagged cameras.
//!
//! ## Quickstart
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_settings_plus::prelude::*;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(SettingsPlusPlugins::new("net.pavlov.my_game"))
//!     .add_systems(Startup, |mut commands: Commands| {
//!         commands.spawn((Camera3d::default(), SettingsCamera));
//!     })
//!     .run();
//! ```
//!
//! To save user changes (e.g., from an "Apply" button):
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_settings_plus::prelude::*;
//!
//! fn save_button(mut commands: Commands) {
//!     commands.queue(SavePreferencesDeferred::default());
//! }
//! ```
//!
//! ## Cargo features
//!
//! - `dlss` — wires [DLSS](https://developer.nvidia.com/rtx/dlss) via
//!   `bevy_anti_alias`. Requires the NVIDIA DLSS SDK at build time; only
//!   enable for shipping NVIDIA-targeted builds.

pub mod caps;
pub mod caps_aware;
pub mod common;

use bevy_app::{PluginGroup, PluginGroupBuilder};
use bevy_ecs::schedule::SystemSet;
use bevy_settings::PreferencesPlugin;

use crate::caps_aware::CapsAwarePlugin;
use crate::common::{AccessibilitySettingsPlugin, DisplaySettingsPlugin, RenderSettingsPlugin};

/// Run-order anchor for the binding systems that translate settings
/// resources onto engine resources, in `PostUpdate`.
///
/// Use `.in_set(ApplyBindings)` (or `.before(ApplyBindings)`) to slot your
/// own systems into the lifecycle.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ApplyBindings;

/// Convenience [`PluginGroup`] that wires the whole stack in the right order:
///
/// 1. [`DisplaySettingsPlugin`] / [`RenderSettingsPlugin`] /
///    [`AccessibilitySettingsPlugin`] — register reflection types and
///    binding systems before `bevy_settings` scans the registry.
/// 2. [`bevy_settings::PreferencesPlugin`] — discover registered
///    [`SettingsGroup`](bevy_settings::SettingsGroup) resources and load
///    their values from the on-disk TOML.
/// 3. [`CapsAwarePlugin`] — runs in `finish()`: detects [`AdapterCaps`](caps::AdapterCaps)
///    and clamps every registered [`CapsAware`](caps_aware::CapsAware)
///    resource against it.
///
/// Add this *after* `DefaultPlugins` (so `RenderApp` is initialized by the
/// time `CapsAwarePlugin::finish` runs).
///
/// To opt out of any layer, use the standard `.disable::<T>()` pattern:
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_settings_plus::prelude::*;
/// # use bevy_app::PluginGroup;
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(
///         SettingsPlusPlugins::new("net.pavlov.my_game")
///             .build()
///             .disable::<AccessibilitySettingsPlugin>(),
///     );
/// ```
pub struct SettingsPlusPlugins {
    app_name: String,
}

impl SettingsPlusPlugins {
    /// Wires the group with the given reverse-domain application id (e.g.
    /// `"net.pavlov.my_game"`). Passed through to
    /// [`PreferencesPlugin::new`] for on-disk path derivation.
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
        }
    }
}

impl PluginGroup for SettingsPlusPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            // Schema plugins first: they register their reflection types.
            .add(DisplaySettingsPlugin)
            .add(RenderSettingsPlugin)
            .add(AccessibilitySettingsPlugin)
            // Then the upstream loader, which scans the registry.
            .add(PreferencesPlugin::new(&self.app_name))
            // Finally, caps detection + the reflection-driven clamp pass.
            .add(CapsAwarePlugin)
    }
}

pub mod prelude {
    //! Re-exports of the most commonly used types.

    pub use crate::caps::{AdapterCaps, Backend, GpuVendor, PlatformTarget};
    pub use crate::caps_aware::{CapsAware, CapsAwarePlugin, ReflectCapsAware};
    pub use crate::common::{
        AccessibilitySettings, AccessibilitySettingsPlugin, AntiAlias, ColorBlindMode,
        DisplaySettings, DisplaySettingsPlugin, FpsCap, HdrPreference, MonitorSelection, MsaaLevel,
        RenderSettings, RenderSettingsPlugin, Resolution, SettingsCamera, Upscaler, UpscalerPreset,
        VsyncMode, WindowMode,
    };
    pub use crate::{ApplyBindings, SettingsPlusPlugins};

    // Re-export the most commonly used `bevy_settings` surface so callers
    // only need `bevy_settings_plus::prelude::*`.
    pub use bevy_settings::{
        PreferencesPlugin, SavePreferences, SavePreferencesDeferred, SavePreferencesSync,
        SettingsGroup,
    };
}
