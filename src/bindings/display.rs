//! Apply [`Display`](crate::common::Display) to the
//! [`PrimaryWindow`](bevy_window::PrimaryWindow).

use bevy_ecs::prelude::*;
use bevy_window::{
    MonitorSelection as BevyMonitorSelection, PresentMode, PrimaryWindow, VideoModeSelection,
    Window, WindowMode as BevyWindowMode,
};

use crate::common::{
    CommonConfig, MonitorSelection, Resolution, VsyncMode, WindowMode as CfgWindowMode,
};

pub(super) fn apply_display(
    config: Res<CommonConfig>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    let display = &config.display;

    let bevy_monitor = match display.monitor {
        MonitorSelection::Primary => BevyMonitorSelection::Primary,
        MonitorSelection::Index(i) => BevyMonitorSelection::Index(i as usize),
    };

    let target_mode = match display.window_mode {
        CfgWindowMode::Windowed => BevyWindowMode::Windowed,
        CfgWindowMode::BorderlessFullscreen => BevyWindowMode::BorderlessFullscreen(bevy_monitor),
        CfgWindowMode::Fullscreen => {
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
