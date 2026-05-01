//! Opinionated graphics + accessibility schema.
//!
//! Three independent axes, each one a `Resource + SettingsGroup +
//! CapsAware + Default`:
//!
//! - [`DisplaySettings`] — window mode, monitor, resolution, vsync, …
//! - [`RenderSettings`] — anti-alias, MSAA, upscaler, render scale, RT toggle.
//! - [`AccessibilitySettings`] — subtitles, motion, colour-blind mode.
//!
//! Each axis is its own self-contained module: the resource type, its leaf
//! enums, `Default`/`platform_default`/[`CapsAware`](crate::caps_aware::CapsAware)
//! impls, the binding system (where applicable), and a per-axis
//! `*SettingsPlugin`.
//!
//! Audio is intentionally absent — the bus convention is backend-specific
//! (firewheel/seedling/kira/oddio all model differently). When the
//! firewheel/seedling integration lands it will ship as its own
//! [`SettingsGroup`](bevy_settings::SettingsGroup) resource behind a cargo
//! feature.

mod accessibility;
mod display;
mod render;

pub use accessibility::*;
pub use display::*;
pub use render::*;
