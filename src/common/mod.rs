//! Universal `CommonConfig` schema: display, render, accessibility.
//!
//! Drop-in for projects that want a sensible baseline. Game-specific quality
//! dials live in a separate `Config`-implementing type the consumer defines
//! themselves.
//!
//! Audio is intentionally absent — the bus convention is backend-specific
//! (firewheel/seedling/kira/oddio all model differently), so audio config
//! lives behind its own cargo feature with a backend-specific `Config` type.

mod accessibility;
mod display;
mod render;

pub use accessibility::*;
pub use display::*;
pub use render::*;

use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::resource::Resource;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::caps::{AdapterCaps, PlatformTarget};
use crate::config::Config;

/// Aggregate of the universal axes. Implements [`Config`].
#[derive(Resource, Reflect, Debug, Clone, PartialEq)]
#[reflect(Resource)]
pub struct CommonConfig {
    pub display: Display,
    pub render: Render,
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
            accessibility: Accessibility::platform_default(target),
        }
    }

    fn merge(&mut self, overrides: &Self::Overrides) {
        self.display.merge(&overrides.display);
        self.render.merge(&overrides.render);
        self.accessibility.merge(&overrides.accessibility);
    }

    fn clamp_to(&mut self, caps: &AdapterCaps) {
        self.display.clamp_to(caps);
        self.render.clamp_to(caps);
        self.accessibility.clamp_to(caps);
    }

    fn current_overrides(&self, target: PlatformTarget) -> Self::Overrides {
        let default = Self::platform_default(target);
        CommonConfigOverrides {
            display: self.display.diff(&default.display),
            render: self.render.diff(&default.render),
            accessibility: self.accessibility.diff(&default.accessibility),
        }
    }
}
