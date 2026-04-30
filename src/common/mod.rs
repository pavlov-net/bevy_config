//! Universal `CommonConfig` schema: display, render, audio, accessibility.
//!
//! Drop-in for projects that want a sensible baseline. Game-specific quality
//! dials live in a separate `Config`-implementing type the consumer defines
//! themselves.

mod accessibility;
mod audio;
mod display;
mod render;

pub use accessibility::*;
pub use audio::*;
pub use display::*;
pub use render::*;

use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::resource::Resource;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::caps::{AdapterCaps, PlatformTarget};
use crate::config::Config;

/// Aggregate of all four universal axes. Implements [`Config`].
#[derive(Resource, Reflect, Debug, Clone, PartialEq)]
#[reflect(Resource)]
pub struct CommonConfig {
    pub display: Display,
    pub render: Render,
    pub audio: Audio,
    pub accessibility: Accessibility,
}

/// Sparse on-disk overrides for [`CommonConfig`].
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct CommonConfigOverrides {
    #[serde(skip_serializing_if = "is_default_overrides")]
    pub display: DisplayOverrides,
    #[serde(skip_serializing_if = "is_default_overrides")]
    pub render: RenderOverrides,
    #[serde(skip_serializing_if = "is_default_overrides")]
    pub audio: AudioOverrides,
    #[serde(skip_serializing_if = "is_default_overrides")]
    pub accessibility: AccessibilityOverrides,
}

fn is_default_overrides<T: Default + PartialEq>(v: &T) -> bool {
    *v == T::default()
}

impl Config for CommonConfig {
    type Overrides = CommonConfigOverrides;

    fn platform_default(target: PlatformTarget) -> Self {
        Self {
            display: Display::platform_default(target),
            render: Render::platform_default(target),
            audio: Audio::platform_default(target),
            accessibility: Accessibility::platform_default(target),
        }
    }

    fn merge(&mut self, overrides: &Self::Overrides) {
        self.display.merge(&overrides.display);
        self.render.merge(&overrides.render);
        self.audio.merge(&overrides.audio);
        self.accessibility.merge(&overrides.accessibility);
    }

    fn clamp_to(&mut self, caps: &AdapterCaps) {
        self.display.clamp_to(caps);
        self.render.clamp_to(caps);
        self.audio.clamp_to(caps);
        self.accessibility.clamp_to(caps);
    }

    fn current_overrides(&self, target: PlatformTarget) -> Self::Overrides {
        let default = Self::platform_default(target);
        CommonConfigOverrides {
            display: self.display.diff(&default.display),
            render: self.render.diff(&default.render),
            audio: self.audio.diff(&default.audio),
            accessibility: self.accessibility.diff(&default.accessibility),
        }
    }
}
