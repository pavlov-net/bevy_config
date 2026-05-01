//! Sparse-overrides merge correctness, caps clamping, platform defaults.
//!
//! These tests do not touch the Bevy `App` — they exercise the `Config`
//! trait directly to keep the kernel's contract verifiable in isolation.

use bevy_config::prelude::*;

fn nvidia_vulkan_caps() -> AdapterCaps {
    AdapterCaps {
        backend: Backend::Vulkan,
        vendor: GpuVendor::Nvidia,
        max_texture_dim_2d: 16384,
        ray_query: true,
        timestamp_query: true,
        supports_dlss: true,
    }
}

fn integrated_intel_caps() -> AdapterCaps {
    AdapterCaps {
        backend: Backend::Vulkan,
        vendor: GpuVendor::Intel,
        max_texture_dim_2d: 8192,
        ray_query: false,
        timestamp_query: true,
        supports_dlss: false,
    }
}

#[test]
fn platform_default_wasm_is_conservative() {
    let cfg = CommonConfig::platform_default(PlatformTarget::Wasm);
    assert!(matches!(cfg.render.anti_alias, AntiAlias::None));
    assert!(matches!(cfg.render.upscaler, Upscaler::Native));
    assert!(matches!(cfg.render.msaa, MsaaLevel::Off));
    assert!(!cfg.render.ray_tracing);
}

#[test]
fn platform_default_desktop_uses_taa() {
    let cfg = CommonConfig::platform_default(PlatformTarget::LinuxDesktop);
    assert!(matches!(cfg.render.anti_alias, AntiAlias::Taa));
}

#[test]
fn merge_applies_sparse_overrides() {
    let mut cfg = CommonConfig::platform_default(PlatformTarget::LinuxDesktop);

    let mut overrides = CommonConfigOverrides::default();
    overrides.render.anti_alias = Some(AntiAlias::Smaa);
    overrides.render.msaa = Some(MsaaLevel::Sample4);
    overrides.accessibility.subtitles_on = Some(true);

    cfg.merge(&overrides);

    assert!(matches!(cfg.render.anti_alias, AntiAlias::Smaa));
    assert!(matches!(cfg.render.msaa, MsaaLevel::Sample4));
    assert!(cfg.accessibility.subtitles_on);
    // Untouched fields keep platform defaults.
    assert!(matches!(cfg.render.upscaler, Upscaler::Native));
}

#[test]
fn clamp_strips_ray_tracing_when_caps_lack_ray_query() {
    let mut cfg = CommonConfig::platform_default(PlatformTarget::LinuxDesktop);
    cfg.render.ray_tracing = true;

    cfg.clamp_to(&integrated_intel_caps());

    assert!(!cfg.render.ray_tracing);
}

#[test]
fn clamp_keeps_ray_tracing_when_caps_support_it() {
    let mut cfg = CommonConfig::platform_default(PlatformTarget::LinuxDesktop);
    cfg.render.ray_tracing = true;

    cfg.clamp_to(&nvidia_vulkan_caps());

    assert!(cfg.render.ray_tracing);
}

#[test]
fn clamp_falls_back_to_native_when_dlss_unavailable() {
    let mut cfg = CommonConfig::platform_default(PlatformTarget::LinuxDesktop);
    cfg.render.upscaler = Upscaler::Dlss(UpscalerPreset::Quality);

    cfg.clamp_to(&integrated_intel_caps());

    assert!(matches!(cfg.render.upscaler, Upscaler::Native));
}

#[test]
fn clamp_keeps_dlss_when_supported() {
    let mut cfg = CommonConfig::platform_default(PlatformTarget::LinuxDesktop);
    cfg.render.upscaler = Upscaler::Dlss(UpscalerPreset::Balanced);

    cfg.clamp_to(&nvidia_vulkan_caps());

    assert!(matches!(cfg.render.upscaler, Upscaler::Dlss(_)));
}

#[test]
fn clamp_clamps_render_scale_and_sharpness() {
    let mut cfg = CommonConfig::platform_default(PlatformTarget::LinuxDesktop);
    cfg.render.render_scale = 1.5; // > 1.0
    cfg.render.sharpness = -0.2; // < 0.0

    cfg.clamp_to(&nvidia_vulkan_caps());

    assert!(cfg.render.render_scale <= 1.0 && cfg.render.render_scale >= 0.25);
    assert!(cfg.render.sharpness >= 0.0 && cfg.render.sharpness <= 1.0);
}

#[test]
fn current_overrides_is_empty_when_unchanged() {
    let cfg = CommonConfig::platform_default(PlatformTarget::LinuxDesktop);
    let overrides = cfg.current_overrides(PlatformTarget::LinuxDesktop);
    assert_eq!(overrides, CommonConfigOverrides::default());
}

#[test]
fn current_overrides_captures_only_changes() {
    let mut cfg = CommonConfig::platform_default(PlatformTarget::LinuxDesktop);
    cfg.render.msaa = MsaaLevel::Sample4;
    cfg.accessibility.subtitles_on = true;

    let overrides = cfg.current_overrides(PlatformTarget::LinuxDesktop);
    assert_eq!(overrides.render.msaa, Some(MsaaLevel::Sample4));
    assert_eq!(overrides.accessibility.subtitles_on, Some(true));
    // Untouched fields are None in the diff.
    assert_eq!(overrides.render.anti_alias, None);
    assert_eq!(overrides.accessibility.reduce_motion, None);
}

#[test]
fn roundtrip_merge_after_diff() {
    let target = PlatformTarget::LinuxDesktop;
    let mut original = CommonConfig::platform_default(target);
    original.render.msaa = MsaaLevel::Sample8;
    original.accessibility.subtitle_scale = 1.5;

    let overrides = original.current_overrides(target);

    let mut rebuilt = CommonConfig::platform_default(target);
    rebuilt.merge(&overrides);

    assert_eq!(rebuilt, original);
}
