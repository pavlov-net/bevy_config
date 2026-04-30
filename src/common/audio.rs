//! `Audio` axis: per-bus volumes, output device, channel mode.
//!
//! **Schema only in v0.1.** No engine binding — `bevy_seedling` integration is
//! deferred until the bus convention is decided.

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::caps::{AdapterCaps, PlatformTarget};

/// Audio output channel mode.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelMode {
    Headphones,
    Stereo,
    Surround51,
    Surround71,
}

/// Populated audio config. All volumes are in decibels (`0.0` = unity gain;
/// negative = quieter; positive = louder, clamp at the binding layer).
#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct Audio {
    pub master_db: f32,
    pub music_db: f32,
    pub sfx_db: f32,
    pub voice_db: f32,
    pub ui_db: f32,
    pub ambient_db: f32,
    /// Output device id; `None` = system default. Schema slot — enumeration
    /// API is dependent on the audio backend (e.g., `firewheel`) surface.
    pub output_device: Option<String>,
    pub channel_mode: ChannelMode,
}

/// Sparse on-disk overrides for [`Audio`].
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AudioOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master_db: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub music_db: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sfx_db: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_db: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_db: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambient_db: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_device: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_mode: Option<ChannelMode>,
}

impl Audio {
    pub(crate) fn platform_default(_target: PlatformTarget) -> Self {
        Self {
            master_db: 0.0,
            music_db: 0.0,
            sfx_db: 0.0,
            voice_db: 0.0,
            ui_db: 0.0,
            ambient_db: 0.0,
            output_device: None,
            channel_mode: ChannelMode::Stereo,
        }
    }

    pub(crate) fn merge(&mut self, o: &AudioOverrides) {
        if let Some(v) = o.master_db {
            self.master_db = v;
        }
        if let Some(v) = o.music_db {
            self.music_db = v;
        }
        if let Some(v) = o.sfx_db {
            self.sfx_db = v;
        }
        if let Some(v) = o.voice_db {
            self.voice_db = v;
        }
        if let Some(v) = o.ui_db {
            self.ui_db = v;
        }
        if let Some(v) = o.ambient_db {
            self.ambient_db = v;
        }
        if let Some(v) = o.output_device.clone() {
            self.output_device = v;
        }
        if let Some(v) = o.channel_mode {
            self.channel_mode = v;
        }
    }

    pub(crate) fn clamp_to(&mut self, _caps: &AdapterCaps) {
        // No audio-axis caps clamping in v0.1.
    }

    pub(crate) fn diff(&self, default: &Self) -> AudioOverrides {
        AudioOverrides {
            master_db: (self.master_db != default.master_db).then_some(self.master_db),
            music_db: (self.music_db != default.music_db).then_some(self.music_db),
            sfx_db: (self.sfx_db != default.sfx_db).then_some(self.sfx_db),
            voice_db: (self.voice_db != default.voice_db).then_some(self.voice_db),
            ui_db: (self.ui_db != default.ui_db).then_some(self.ui_db),
            ambient_db: (self.ambient_db != default.ambient_db).then_some(self.ambient_db),
            output_device: (self.output_device != default.output_device)
                .then(|| self.output_device.clone()),
            channel_mode: (self.channel_mode != default.channel_mode).then_some(self.channel_mode),
        }
    }
}
