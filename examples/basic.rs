//! Minimal `bevy_config` example.
//!
//! Adds the kernel + common bindings, observes `ConfigApplied`, and prints the
//! detected `AdapterCaps` and resolved `CommonConfig` once at startup.
//!
//! Run with:
//!
//! ```bash
//! cargo run --example basic
//! ```

use bevy::prelude::*;
use bevy_config::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ConfigPlugin::<CommonConfig>::new(
            FileBackend::<CommonConfig>::new("net", "pavlov", "bevy_config_basic_example"),
        ))
        .add_plugins(CommonBindingsPlugin)
        .add_observer(on_config_applied)
        .add_systems(Startup, spawn_camera)
        .run();
}

fn on_config_applied(
    _: On<ConfigApplied<CommonConfig>>,
    config: Res<CommonConfig>,
    caps: Option<Res<AdapterCaps>>,
) {
    info!("=== bevy_config: ConfigApplied ===");
    if let Some(caps) = caps {
        info!(
            "AdapterCaps: backend={:?} vendor={:?} ray_query={} dlss_supported={}",
            caps.backend, caps.vendor, caps.ray_query, caps.supports_dlss
        );
    } else {
        info!("AdapterCaps not yet inserted (running headless?).");
    }
    info!(
        "Display: window_mode={:?} vsync={:?}",
        config.display.window_mode, config.display.vsync
    );
    info!(
        "Render: anti_alias={:?} msaa={:?} upscaler={:?} ray_tracing={}",
        config.render.anti_alias,
        config.render.msaa,
        config.render.upscaler,
        config.render.ray_tracing
    );
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        BevyConfigCamera,
    ));
}
