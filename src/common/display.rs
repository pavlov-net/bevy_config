//! `Display` axis: window mode, monitor, resolution, vsync, fps cap, HDR.

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::caps::{AdapterCaps, PlatformTarget};

/// Window presentation mode.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowMode {
    Windowed,
    BorderlessFullscreen,
    Fullscreen,
}

/// Vsync intent. The binding maps these onto [`bevy_window::PresentMode`].
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VsyncMode {
    /// Driver-default vsync (maps to `AutoVsync`).
    Auto,
    /// Force vsync on (maps to `Fifo`).
    On,
    /// Force vsync off (maps to `Immediate` if available, else `AutoNoVsync`).
    Off,
    /// Triple-buffered vsync (maps to `Mailbox`).
    Mailbox,
}

/// Monitor selection. The binding maps onto [`bevy_window::MonitorSelection`].
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitorSelection {
    Primary,
    Index(u32),
}

/// Resolution choice. `Native` defers to the monitor's preferred mode.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resolution {
    Native,
    Custom { width: u32, height: u32 },
}

/// Frame-rate cap. `Unlimited` disables capping.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FpsCap {
    Unlimited,
    Capped(u32),
}

/// HDR display preference (slot — auto-detect on Windows DXGI is deferred).
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HdrPreference {
    Off,
    On,
    Auto,
}

/// Populated display config.
#[derive(Reflect, Debug, Clone, PartialEq, Eq)]
pub struct Display {
    pub window_mode: WindowMode,
    pub monitor: MonitorSelection,
    pub resolution: Resolution,
    pub refresh_rate: Option<u32>,
    pub vsync: VsyncMode,
    pub fps_cap: FpsCap,
    pub hdr: HdrPreference,
}

/// Sparse on-disk overrides for [`Display`].
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DisplayOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_mode: Option<WindowMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitor: Option<MonitorSelection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<Resolution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_rate: Option<Option<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vsync: Option<VsyncMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fps_cap: Option<FpsCap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hdr: Option<HdrPreference>,
}

impl Display {
    pub(crate) fn platform_default(_target: PlatformTarget) -> Self {
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

    pub(crate) fn merge(&mut self, o: &DisplayOverrides) {
        if let Some(v) = o.window_mode {
            self.window_mode = v;
        }
        if let Some(v) = o.monitor {
            self.monitor = v;
        }
        if let Some(v) = o.resolution {
            self.resolution = v;
        }
        if let Some(v) = o.refresh_rate {
            self.refresh_rate = v;
        }
        if let Some(v) = o.vsync {
            self.vsync = v;
        }
        if let Some(v) = o.fps_cap {
            self.fps_cap = v;
        }
        if let Some(v) = o.hdr {
            self.hdr = v;
        }
    }

    pub(crate) fn clamp_to(&mut self, _caps: &AdapterCaps) {
        // No display-axis caps clamping in v0.1.
    }

    pub(crate) fn diff(&self, default: &Self) -> DisplayOverrides {
        DisplayOverrides {
            window_mode: (self.window_mode != default.window_mode).then_some(self.window_mode),
            monitor: (self.monitor != default.monitor).then_some(self.monitor),
            resolution: (self.resolution != default.resolution).then_some(self.resolution),
            refresh_rate: (self.refresh_rate != default.refresh_rate).then_some(self.refresh_rate),
            vsync: (self.vsync != default.vsync).then_some(self.vsync),
            fps_cap: (self.fps_cap != default.fps_cap).then_some(self.fps_cap),
            hdr: (self.hdr != default.hdr).then_some(self.hdr),
        }
    }
}
