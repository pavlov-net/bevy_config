//! Demonstrates deferred camera spawning.
//!
//! This is the realistic pattern for non-trivial games: the camera is
//! spawned after a state transition (e.g., `OnEnter(AppState::InGame)`)
//! or after async asset load — *not* at `Startup`. The
//! [`BevyConfigCamera`] marker still gets the right anti-alias / MSAA /
//! upscaler on its first frame, because [`CommonBindingsPlugin`]
//! registers an `On<Add, BevyConfigCamera>` observer.
//!
//! Run with:
//!
//! ```bash
//! cargo run --example deferred_camera
//! ```

use bevy::prelude::*;
use bevy_anti_alias::taa::TemporalAntiAliasing;
use bevy_config::prelude::*;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    #[default]
    Loading,
    InGame,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ConfigPlugin::<CommonConfig>::new(
            FileBackend::<CommonConfig>::new("net", "pavlov", "bevy_config_deferred_example"),
        ))
        .add_plugins(CommonBindingsPlugin)
        .init_state::<AppState>()
        .add_systems(Startup, kick_off_loading)
        .add_systems(OnEnter(AppState::InGame), spawn_camera)
        .add_observer(verify_taa_attached)
        .run();
}

/// Simulate "loading is done" by transitioning to InGame after the first
/// real frame has rendered. In a real game this would be gated on asset
/// load completion, network handshake, etc.
fn kick_off_loading(mut next: ResMut<NextState<AppState>>) {
    info!("Loading… (will spawn camera on next state transition)");
    next.set(AppState::InGame);
}

fn spawn_camera(mut commands: Commands) {
    info!("Entering InGame: spawning camera with BevyConfigCamera marker");
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        BevyConfigCamera,
    ));
}

/// Confirm the binding observer attached `TemporalAntiAliasing` (the
/// desktop platform default) the moment the deferred camera was tagged
/// with `BevyConfigCamera`, despite the camera spawning *after* the
/// initial `ConfigApplied` pulse.
fn verify_taa_attached(
    add: On<Add, TemporalAntiAliasing>,
    cameras: Query<(), With<BevyConfigCamera>>,
) {
    if cameras.contains(add.entity) {
        info!(
            "Observer applied TemporalAntiAliasing to the deferred camera \
             on its first frame — deferred-spawn race fixed."
        );
    }
}
