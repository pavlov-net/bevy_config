//! `DisplaySettings` axis: window mode, monitor, resolution, vsync, fps cap, HDR.
//!
//! Ships the [`DisplaySettings`] resource (a `bevy_settings::SettingsGroup`), leaf
//! enums (`WindowMode`, `VsyncMode`, `MonitorSelection`, `Resolution`,
//! `FpsCap`, `HdrPreference`), and [`DisplaySettingsPlugin`] which adds
//! the primary-window binding system.

use bevy_app::{App, Plugin, PostUpdate};
use bevy_ecs::prelude::*;
use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::resource::Resource;
use bevy_reflect::Reflect;
use bevy_reflect::prelude::ReflectDefault;
use bevy_settings::{ReflectSettingsGroup, SettingsGroup};
use bevy_window::{
    MonitorSelection as BevyMonitorSelection, PresentMode, PrimaryWindow, VideoModeSelection,
    Window, WindowMode as BevyWindowMode,
};

use crate::ApplyBindings;
use crate::caps::{AdapterCaps, PlatformTarget};
use crate::caps_aware::{CapsAware, ReflectCapsAware};

/// Window presentation mode.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[reflect(Default)]
pub enum WindowMode {
    #[default]
    Windowed,
    BorderlessFullscreen,
    Fullscreen,
}

/// Vsync intent. The binding maps these onto [`bevy_window::PresentMode`].
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[reflect(Default)]
#[non_exhaustive]
pub enum VsyncMode {
    /// Driver-default vsync (maps to `AutoVsync`).
    #[default]
    Auto,
    /// Force vsync on (maps to `Fifo`).
    On,
    /// Force vsync off (maps to `Immediate` if available, else `AutoNoVsync`).
    Off,
    /// Triple-buffered vsync (maps to `Mailbox`).
    Mailbox,
}

/// Monitor selection. The binding maps onto [`bevy_window::MonitorSelection`].
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[reflect(Default)]
#[non_exhaustive]
pub enum MonitorSelection {
    #[default]
    Primary,
    Index(u32),
}

/// Resolution choice. `Native` defers to the monitor's preferred mode.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[reflect(Default)]
pub enum Resolution {
    #[default]
    Native,
    Custom {
        width: u32,
        height: u32,
    },
}

/// Frame-rate cap. `Unlimited` disables capping.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[reflect(Default)]
pub enum FpsCap {
    #[default]
    Unlimited,
    Capped(u32),
}

/// HDR display preference (slot — auto-detect on Windows DXGI is deferred).
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[reflect(Default)]
pub enum HdrPreference {
    #[default]
    Off,
    On,
    Auto,
}

/// DisplaySettings config resource. Discovered by `bevy_settings::SettingsPlugin`
/// via [`SettingsGroup`] and clamped to caps by
/// [`crate::CapsAwarePlugin`].
#[derive(Resource, SettingsGroup, Reflect, Debug, Clone, PartialEq, Eq)]
#[reflect(Resource, SettingsGroup, CapsAware, Default)]
#[settings_group(group = "display")]
pub struct DisplaySettings {
    pub window_mode: WindowMode,
    pub monitor: MonitorSelection,
    pub resolution: Resolution,
    pub refresh_rate: Option<u32>,
    pub vsync: VsyncMode,
    pub fps_cap: FpsCap,
    pub hdr: HdrPreference,
}

impl DisplaySettings {
    /// Conservative, platform-appropriate defaults. Used by the `Default`
    /// impl (which is what `bevy_settings` constructs from when no value is
    /// loaded from disk).
    pub fn platform_default(_target: PlatformTarget) -> Self {
        Self {
            window_mode: WindowMode::Windowed,
            monitor: MonitorSelection::Primary,
            resolution: Resolution::Native,
            refresh_rate: None,
            vsync: VsyncMode::Auto,
            fps_cap: FpsCap::Unlimited,
            hdr: HdrPreference::Off,
        }
    }
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self::platform_default(PlatformTarget::detect())
    }
}

impl CapsAware for DisplaySettings {
    fn clamp_to(&mut self, _caps: AdapterCaps) {
        // No display-axis caps clamping today.
    }
}

/// Wires the [`DisplaySettings`] axis: registers reflection types and adds
/// the primary-window binding system.
///
/// Add this *before* `bevy_settings::SettingsPlugin` so the type registry
/// is populated by the time it scans for [`SettingsGroup`] resources.
/// [`crate::SettingsPlusPlugins`] handles ordering for you.
pub struct DisplaySettingsPlugin;

impl Plugin for DisplaySettingsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DisplaySettings>();
        app.add_systems(
            PostUpdate,
            apply_display
                .in_set(ApplyBindings)
                .run_if(resource_changed::<DisplaySettings>),
        );
    }
}

pub(crate) fn apply_display(
    display: Res<DisplaySettings>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    let bevy_monitor = match display.monitor {
        MonitorSelection::Primary => BevyMonitorSelection::Primary,
        MonitorSelection::Index(i) => BevyMonitorSelection::Index(i as usize),
    };

    let target_mode = match display.window_mode {
        WindowMode::Windowed => BevyWindowMode::Windowed,
        WindowMode::BorderlessFullscreen => BevyWindowMode::BorderlessFullscreen(bevy_monitor),
        WindowMode::Fullscreen => {
            BevyWindowMode::Fullscreen(bevy_monitor, VideoModeSelection::Current)
        }
    };
    if window.mode != target_mode {
        window.mode = target_mode;
    }

    let target_present = match display.vsync {
        VsyncMode::Auto => PresentMode::AutoVsync,
        VsyncMode::On => PresentMode::Fifo,
        VsyncMode::Off => PresentMode::Immediate,
        VsyncMode::Mailbox => PresentMode::Mailbox,
    };
    if window.present_mode != target_present {
        window.present_mode = target_present;
    }

    if let Resolution::Custom { width, height } = display.resolution
        && (window.resolution.physical_width() != width
            || window.resolution.physical_height() != height)
    {
        window.resolution.set_physical_resolution(width, height);
    }
}
