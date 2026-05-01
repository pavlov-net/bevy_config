//! `Render` axis: anti-alias, MSAA, upscaler, render scale, sharpness, RT toggle.

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::caps::{AdapterCaps, PlatformTarget};

/// Anti-aliasing technique. Mutually exclusive with the technique components
/// — the binding inserts/removes the right one on tagged cameras.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AntiAlias {
    None,
    Fxaa,
    Smaa,
    Taa,
}

/// MSAA sample count.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MsaaLevel {
    Off,
    Sample2,
    Sample4,
    Sample8,
}

/// Upscaler choice. Only `Native` and `Dlss` are wired in v0.1; the others are
/// schema slots — selecting them logs a warning and falls back to `Native` for
/// now.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Upscaler {
    Native,
    Dlss(UpscalerPreset),
    Fsr(UpscalerPreset),
    Xess(UpscalerPreset),
    Tsr(UpscalerPreset),
}

/// Quality preset for upscalers (DLSS / FSR / XeSS / TSR).
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum UpscalerPreset {
    Quality,
    Balanced,
    Performance,
    UltraPerformance,
}

/// Populated render config.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct Render {
    pub anti_alias: AntiAlias,
    pub msaa: MsaaLevel,
    pub upscaler: Upscaler,
    /// Render-target downscale factor in `(0, 1]`. 1.0 = native render
    /// resolution; lower = render at fewer pixels and upscale.
    pub render_scale: f32,
    /// Post-upscale sharpening strength in `[0, 1]`.
    pub sharpness: f32,
    /// Master ray-tracing toggle (e.g., for `bevy_solari`). The bindings do
    /// not consume this directly — consumers gate their RT plugins on it.
    pub ray_tracing: bool,
}

/// Sparse on-disk overrides for [`Render`].
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct RenderOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anti_alias: Option<AntiAlias>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msaa: Option<MsaaLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upscaler: Option<Upscaler>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub render_scale: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sharpness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ray_tracing: Option<bool>,
}

impl Render {
    pub(crate) fn platform_default(target: PlatformTarget) -> Self {
        match target {
            PlatformTarget::Wasm => Self {
                // Conservative wasm defaults: no temporal AA, no MSAA, no RT.
                anti_alias: AntiAlias::None,
                msaa: MsaaLevel::Off,
                upscaler: Upscaler::Native,
                render_scale: 1.0,
                sharpness: 0.0,
                ray_tracing: false,
            },
            _ => Self {
                anti_alias: AntiAlias::Taa,
                msaa: MsaaLevel::Off,
                upscaler: Upscaler::Native,
                render_scale: 1.0,
                sharpness: 0.0,
                ray_tracing: false,
            },
        }
    }

    pub(crate) fn merge(&mut self, o: &RenderOverrides) {
        if let Some(v) = o.anti_alias {
            self.anti_alias = v;
        }
        if let Some(v) = o.msaa {
            self.msaa = v;
        }
        if let Some(v) = o.upscaler {
            self.upscaler = v;
        }
        if let Some(v) = o.render_scale {
            self.render_scale = v;
        }
        if let Some(v) = o.sharpness {
            self.sharpness = v;
        }
        if let Some(v) = o.ray_tracing {
            self.ray_tracing = v;
        }
    }

    pub(crate) fn clamp_to(&mut self, caps: &AdapterCaps) {
        // No RT support → force the master toggle off, regardless of override.
        if self.ray_tracing && !caps.ray_query {
            self.ray_tracing = false;
        }
        // DLSS without adapter support → fall back to Native. (Cargo-feature
        // gating happens at binding time; this clamp covers the
        // wrong-vendor/wrong-backend case even if the feature is compiled in.)
        if matches!(self.upscaler, Upscaler::Dlss(_)) && !caps.supports_dlss {
            self.upscaler = Upscaler::Native;
        }
        self.render_scale = self.render_scale.clamp(0.25, 1.0);
        self.sharpness = self.sharpness.clamp(0.0, 1.0);
    }

    pub(crate) fn diff(&self, default: &Self) -> RenderOverrides {
        RenderOverrides {
            anti_alias: (self.anti_alias != default.anti_alias).then_some(self.anti_alias),
            msaa: (self.msaa != default.msaa).then_some(self.msaa),
            upscaler: (self.upscaler != default.upscaler).then_some(self.upscaler),
            render_scale: (self.render_scale != default.render_scale).then_some(self.render_scale),
            sharpness: (self.sharpness != default.sharpness).then_some(self.sharpness),
            ray_tracing: (self.ray_tracing != default.ray_tracing).then_some(self.ray_tracing),
        }
    }
}
