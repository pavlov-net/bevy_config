//! `RenderSettings` axis: anti-alias, MSAA, upscaler, render scale, sharpness, RT toggle.
//!
//! Ships the [`RenderSettings`] resource (a `bevy_settings::SettingsGroup`), leaf
//! enums (`AntiAlias`, `MsaaLevel`, `Upscaler`, `UpscalerPreset`), the
//! [`SettingsCamera`] marker, and [`RenderSettingsPlugin`] which adds the
//! binding system + an `Add<SettingsCamera>` observer.

use bevy_anti_alias::fxaa::Fxaa;
use bevy_anti_alias::smaa::Smaa;
use bevy_anti_alias::taa::TemporalAntiAliasing;
use bevy_app::{App, Plugin, PostUpdate};
use bevy_ecs::prelude::*;
use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::resource::Resource;
use bevy_log::warn;
use bevy_reflect::Reflect;
use bevy_reflect::prelude::ReflectDefault;
use bevy_render::view::Msaa;
use bevy_settings::{ReflectSettingsGroup, SettingsGroup};

use crate::ApplyBindings;
use crate::caps::{AdapterCaps, PlatformTarget};
use crate::caps_aware::{CapsAware, ReflectCapsAware};

/// Anti-aliasing technique. Mutually exclusive with the technique components
/// — the binding inserts/removes the right one on tagged cameras.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[reflect(Default)]
#[non_exhaustive]
pub enum AntiAlias {
    #[default]
    None,
    Fxaa,
    Smaa,
    Taa,
}

/// MSAA sample count.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[reflect(Default)]
#[non_exhaustive]
pub enum MsaaLevel {
    #[default]
    Off,
    Sample2,
    Sample4,
    Sample8,
}

/// Upscaler choice. Only `Native` and `Dlss` are wired today; the others are
/// schema slots — selecting them logs a warning and falls back to `Native`.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[reflect(Default)]
#[non_exhaustive]
pub enum Upscaler {
    #[default]
    Native,
    Dlss(UpscalerPreset),
    Fsr(UpscalerPreset),
    Xess(UpscalerPreset),
    Tsr(UpscalerPreset),
}

/// Quality preset for upscalers (DLSS / FSR / XeSS / TSR).
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[reflect(Default)]
#[non_exhaustive]
pub enum UpscalerPreset {
    #[default]
    Quality,
    Balanced,
    Performance,
    UltraPerformance,
}

/// Marker component on cameras whose anti-alias / MSAA / upscaler should be
/// driven by `bevy_settings_plus`.
///
/// Spawn alongside the camera type you want:
///
/// ```ignore
/// commands.spawn((Camera3d::default(), SettingsCamera));
/// ```
///
/// The bindings apply on `Add<SettingsCamera>` so cameras spawned after
/// the initial settings-loaded tick still get the current settings.
///
/// **2D caveat**: the desktop platform default is `AntiAlias::Taa`, which
/// requires the depth + motion-vector prepasses and is 3D-only. If you
/// tag a `Camera2d` with `SettingsCamera`, override the AA setting to
/// `AntiAlias::Fxaa` or `AntiAlias::Smaa` before spawn — TAA on a 2D
/// camera will mis-render or fail Bevy's required-component checks.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct SettingsCamera;

/// RenderSettings config resource. Discovered by `bevy_settings::PreferencesPlugin`
/// via [`SettingsGroup`] and clamped to caps by
/// [`crate::CapsAwarePlugin`].
#[derive(Resource, SettingsGroup, Reflect, Debug, Clone, PartialEq)]
#[reflect(Resource, SettingsGroup, CapsAware, Default)]
#[settings_group(group = "render")]
pub struct RenderSettings {
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

impl RenderSettings {
    /// Conservative, platform-appropriate defaults.
    pub fn platform_default(target: PlatformTarget) -> Self {
        match target {
            PlatformTarget::Wasm => Self {
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
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self::platform_default(PlatformTarget::detect())
    }
}

impl CapsAware for RenderSettings {
    fn clamp_to(&mut self, caps: AdapterCaps) {
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
}

/// Wires the [`RenderSettings`] axis: registers reflection types and adds
/// the binding system + the camera-add observer.
pub struct RenderSettingsPlugin;

impl Plugin for RenderSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RenderSettings>();
        app.add_systems(
            PostUpdate,
            apply_render
                .in_set(ApplyBindings)
                .run_if(resource_changed::<RenderSettings>),
        );
        app.add_observer(apply_render_on_camera_add);
    }
}

fn apply_render_to_entity(
    commands: &mut Commands,
    entity: Entity,
    render: &RenderSettings,
    caps: Option<&AdapterCaps>,
) {
    apply_anti_alias(commands, entity, render.anti_alias);
    apply_msaa(commands, entity, render.msaa);
    apply_upscaler(commands, entity, render.upscaler, caps);
}

// Per-axis change detection: only re-apply the AA / MSAA / upscaler axes
// that actually changed since the last run. Re-inserting `TemporalAntiAliasing`
// every frame would otherwise discard accumulated TAA history when the user
// touches an unrelated field like `sharpness`.
pub(crate) fn apply_render(
    render: Res<RenderSettings>,
    caps: Option<Res<AdapterCaps>>,
    cameras: Query<Entity, With<SettingsCamera>>,
    mut commands: Commands,
    mut prev: Local<Option<RenderSettings>>,
) {
    let aa_changed = prev
        .as_ref()
        .is_none_or(|p| p.anti_alias != render.anti_alias);
    let msaa_changed = prev.as_ref().is_none_or(|p| p.msaa != render.msaa);
    let upscaler_changed = prev.as_ref().is_none_or(|p| p.upscaler != render.upscaler);

    if aa_changed || msaa_changed || upscaler_changed {
        for entity in &cameras {
            if aa_changed {
                apply_anti_alias(&mut commands, entity, render.anti_alias);
            }
            if msaa_changed {
                apply_msaa(&mut commands, entity, render.msaa);
            }
            if upscaler_changed {
                apply_upscaler(&mut commands, entity, render.upscaler, caps.as_deref());
            }
        }
    }

    *prev = Some(render.clone());
}

// Observer for the deferred-spawn case: cameras tagged after the initial
// `resource_changed::<RenderSettings>` pulse still get their settings on frame 0.
//
// `RenderSettings` is `Option`al because the observer can fire before
// `PreferencesPlugin::build` inserts the resource (e.g., a user plugin's
// `build` synchronously spawns a `SettingsCamera` between when this
// observer is registered and when the resource is loaded).
pub(crate) fn apply_render_on_camera_add(
    add: On<Add, SettingsCamera>,
    render: Option<Res<RenderSettings>>,
    caps: Option<Res<AdapterCaps>>,
    mut commands: Commands,
) {
    let Some(render) = render else { return };
    apply_render_to_entity(&mut commands, add.entity, &render, caps.as_deref());
}

fn apply_anti_alias(commands: &mut Commands, entity: Entity, mode: AntiAlias) {
    let mut e = commands.entity(entity);
    match mode {
        AntiAlias::None => {
            e.remove::<TemporalAntiAliasing>();
            e.remove::<Fxaa>();
            e.remove::<Smaa>();
        }
        AntiAlias::Fxaa => {
            e.remove::<TemporalAntiAliasing>();
            e.remove::<Smaa>();
            e.insert(Fxaa::default());
        }
        AntiAlias::Smaa => {
            e.remove::<TemporalAntiAliasing>();
            e.remove::<Fxaa>();
            e.insert(Smaa::default());
        }
        AntiAlias::Taa => {
            e.remove::<Fxaa>();
            e.remove::<Smaa>();
            e.insert(TemporalAntiAliasing::default());
        }
    }
}

fn apply_msaa(commands: &mut Commands, entity: Entity, level: MsaaLevel) {
    let msaa = match level {
        MsaaLevel::Off => Msaa::Off,
        MsaaLevel::Sample2 => Msaa::Sample2,
        MsaaLevel::Sample4 => Msaa::Sample4,
        MsaaLevel::Sample8 => Msaa::Sample8,
    };
    commands.entity(entity).insert(msaa);
}

fn apply_upscaler(
    commands: &mut Commands,
    entity: Entity,
    upscaler: Upscaler,
    caps: Option<&AdapterCaps>,
) {
    clear_dlss_if_present(commands, entity);
    match upscaler {
        Upscaler::Native => {}
        Upscaler::Dlss(preset) => apply_dlss(commands, entity, preset, caps),
        Upscaler::Fsr(_) | Upscaler::Xess(_) | Upscaler::Tsr(_) => {
            warn!(
                "Upscaler {:?} is a schema slot today and is not yet wired; \
                 falling back to Native",
                upscaler
            );
        }
    }
}

#[cfg(feature = "dlss")]
fn clear_dlss_if_present(commands: &mut Commands, entity: Entity) {
    use bevy_anti_alias::dlss::{Dlss, DlssSuperResolutionFeature};
    commands
        .entity(entity)
        .remove::<Dlss<DlssSuperResolutionFeature>>();
}

#[cfg(not(feature = "dlss"))]
fn clear_dlss_if_present(_commands: &mut Commands, _entity: Entity) {}

#[cfg(feature = "dlss")]
fn apply_dlss(
    commands: &mut Commands,
    entity: Entity,
    preset: UpscalerPreset,
    caps: Option<&AdapterCaps>,
) {
    if !caps.is_some_and(|c| c.supports_dlss) {
        warn!(
            "Upscaler::Dlss selected but adapter does not support DLSS \
             (Vulkan + NVIDIA required); falling back to Native"
        );
        return;
    }
    use bevy_anti_alias::dlss::{Dlss, DlssPerfQualityMode, DlssSuperResolutionFeature};
    use core::marker::PhantomData;
    let perf_quality_mode = match preset {
        UpscalerPreset::Quality => DlssPerfQualityMode::Quality,
        UpscalerPreset::Balanced => DlssPerfQualityMode::Balanced,
        UpscalerPreset::Performance => DlssPerfQualityMode::Performance,
        UpscalerPreset::UltraPerformance => DlssPerfQualityMode::UltraPerformance,
    };
    commands
        .entity(entity)
        .insert(Dlss::<DlssSuperResolutionFeature> {
            perf_quality_mode,
            reset: false,
            _phantom_data: PhantomData,
        });
}

#[cfg(not(feature = "dlss"))]
fn apply_dlss(
    _commands: &mut Commands,
    _entity: Entity,
    _preset: UpscalerPreset,
    _caps: Option<&AdapterCaps>,
) {
    warn!(
        "Upscaler::Dlss selected but the `dlss` cargo feature is disabled; \
         falling back to Native"
    );
}
