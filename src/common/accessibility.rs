//! `Accessibility` axis: subtitles, motion, colour-blind modes.

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::caps::{AdapterCaps, PlatformTarget};

/// Colour-blind compensation mode (binding TBD — slot only in v0.1; consumers
/// can read the value and adapt their UI shaders/palettes themselves).
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ColorBlindMode {
    None,
    Protanopia,
    Deuteranopia,
    Tritanopia,
}

/// Populated accessibility config.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct Accessibility {
    pub subtitles_on: bool,
    pub subtitle_scale: f32,
    pub reduce_motion: bool,
    pub color_blind_mode: ColorBlindMode,
}

/// Sparse on-disk overrides for [`Accessibility`].
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AccessibilityOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitles_on: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle_scale: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_motion: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_blind_mode: Option<ColorBlindMode>,
}

impl Accessibility {
    pub(crate) fn platform_default(_target: PlatformTarget) -> Self {
        Self {
            subtitles_on: false,
            subtitle_scale: 1.0,
            reduce_motion: false,
            color_blind_mode: ColorBlindMode::None,
        }
    }

    pub(crate) fn merge(&mut self, o: &AccessibilityOverrides) {
        if let Some(v) = o.subtitles_on {
            self.subtitles_on = v;
        }
        if let Some(v) = o.subtitle_scale {
            self.subtitle_scale = v;
        }
        if let Some(v) = o.reduce_motion {
            self.reduce_motion = v;
        }
        if let Some(v) = o.color_blind_mode {
            self.color_blind_mode = v;
        }
    }

    pub(crate) fn clamp_to(&mut self, _caps: &AdapterCaps) {
        self.subtitle_scale = self.subtitle_scale.clamp(0.5, 3.0);
    }

    pub(crate) fn diff(&self, default: &Self) -> AccessibilityOverrides {
        AccessibilityOverrides {
            subtitles_on: (self.subtitles_on != default.subtitles_on).then_some(self.subtitles_on),
            subtitle_scale: (self.subtitle_scale != default.subtitle_scale)
                .then_some(self.subtitle_scale),
            reduce_motion: (self.reduce_motion != default.reduce_motion)
                .then_some(self.reduce_motion),
            color_blind_mode: (self.color_blind_mode != default.color_blind_mode)
                .then_some(self.color_blind_mode),
        }
    }
}
