//! Demonstrates deferred camera spawning.
//!
//! The realistic pattern for non-trivial games: the camera is spawned after
//! a state transition (e.g., `OnEnter(AppState::InGame)`) or after async
//! asset load — *not* at `Startup`. The [`SettingsCamera`] marker still
//! gets the right anti-alias / MSAA / upscaler on its first frame, because
//! [`RenderSettingsPlugin`] registers an `On<Add, SettingsCamera>` observer.
//!
//! Run with:
//!
//! ```bash
//! cargo run --example deferred_camera
//! ```

use bevy::prelude::*;
use bevy_anti_alias::taa::TemporalAntiAliasing;
use bevy_settings_plus::prelude::*;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    #[default]
    Loading,
    InGame,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SettingsPlusPlugins::new(
            "net.pavlov.bevy_settings_plus_deferred_example",
        ))
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
    info!("Entering InGame: spawning camera with SettingsCamera marker");
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        SettingsCamera,
    ));
}

/// Confirm the binding observer attached `TemporalAntiAliasing` (the
/// desktop platform default) the moment the deferred camera was tagged
/// with `SettingsCamera`, despite the camera spawning *after* the
/// initial `RenderSettings`-changed pulse.
fn verify_taa_attached(
    add: On<Add, TemporalAntiAliasing>,
    cameras: Query<(), With<SettingsCamera>>,
) {
    if cameras.contains(add.entity) {
        info!(
            "Observer applied TemporalAntiAliasing to the deferred camera \
             on its first frame — deferred-spawn race fixed."
        );
    }
}
