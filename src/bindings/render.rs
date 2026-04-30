//! Apply [`Render`](crate::common::Render) to cameras carrying the
//! [`BevyConfigCamera`](super::BevyConfigCamera) marker.

use bevy_anti_alias::fxaa::Fxaa;
use bevy_anti_alias::smaa::Smaa;
use bevy_anti_alias::taa::TemporalAntiAliasing;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::*;
use bevy_log::warn;
use bevy_render::view::Msaa;

use crate::caps::AdapterCaps;
use crate::common::{AntiAlias, CommonConfig, MsaaLevel, Upscaler};

use super::BevyConfigCamera;

pub(super) fn apply_render(
    config: Res<CommonConfig>,
    caps: Option<Res<AdapterCaps>>,
    cameras: Query<Entity, With<BevyConfigCamera>>,
    mut commands: Commands,
) {
    for entity in &cameras {
        apply_anti_alias(&mut commands, entity, config.render.anti_alias);
        apply_msaa(&mut commands, entity, config.render.msaa);
        apply_upscaler(
            &mut commands,
            entity,
            config.render.upscaler,
            caps.as_deref(),
        );
    }
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
    // Always start by clearing any DLSS component; we re-insert below if the
    // current selection demands it.
    #[cfg(feature = "dlss")]
    {
        commands
            .entity(entity)
            .remove::<bevy_anti_alias::dlss::Dlss<bevy_anti_alias::dlss::DlssSuperResolutionFeature>>();
    }

    match upscaler {
        Upscaler::Native => {
            // Nothing to insert. (DLSS already removed above.)
            let _ = (commands, entity, caps);
        }
        Upscaler::Dlss(_preset) => {
            #[cfg(feature = "dlss")]
            {
                let supported = caps.is_some_and(|c| c.supports_dlss);
                if !supported {
                    warn!(
                        "Upscaler::Dlss selected but adapter does not support DLSS \
                         (Vulkan + NVIDIA required); falling back to Native"
                    );
                    return;
                }
                use bevy_anti_alias::dlss::{
                    Dlss, DlssPerfQualityMode, DlssSuperResolutionFeature,
                };
                use core::marker::PhantomData;
                let perf_quality_mode = match _preset {
                    crate::common::UpscalerPreset::Quality => DlssPerfQualityMode::Quality,
                    crate::common::UpscalerPreset::Balanced => DlssPerfQualityMode::Balanced,
                    crate::common::UpscalerPreset::Performance => DlssPerfQualityMode::Performance,
                    crate::common::UpscalerPreset::UltraPerformance => {
                        DlssPerfQualityMode::UltraPerformance
                    }
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
            {
                let _ = (commands, entity, caps);
                warn!(
                    "Upscaler::Dlss selected but the `dlss` cargo feature is disabled; \
                     falling back to Native"
                );
            }
        }
        Upscaler::Fsr(_) | Upscaler::Xess(_) | Upscaler::Tsr(_) => {
            let _ = (commands, entity, caps);
            warn!(
                "Upscaler {:?} is a schema slot in v0.1 and is not yet wired; \
                 falling back to Native",
                upscaler
            );
        }
    }
}
