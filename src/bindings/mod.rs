//! Engine bindings for [`CommonConfig`].
//!
//! [`CommonBindingsPlugin`] owns the binding systems for the *graphics* axes
//! (display + render). They run in `PostUpdate.in_set(BevyConfigSet::ApplyBindings)`,
//! gated on `resource_changed::<CommonConfig>`. The first-frame run picks up
//! the initial insert+clamp.
//!
//! Audio bindings are deferred to a later release — see the crate-level
//! README.

mod display;
mod render;

use bevy_app::{App, Plugin, PostUpdate};
use bevy_ecs::prelude::*;

use crate::common::CommonConfig;
use crate::config::BevyConfigSet;

/// Marker component on cameras whose anti-alias / MSAA / upscaler should be
/// driven by `bevy_config`.
///
/// Spawn alongside `Camera3d` (or `Camera2d`):
///
/// ```ignore
/// commands.spawn((Camera3d::default(), BevyConfigCamera));
/// ```
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct BevyConfigCamera;

/// Wires the graphics binding systems for [`CommonConfig`].
///
/// Add this *after* [`crate::plugin::ConfigPlugin`] for [`CommonConfig`].
pub struct CommonBindingsPlugin;

impl Plugin for CommonBindingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (display::apply_display, render::apply_render)
                .in_set(BevyConfigSet::ApplyBindings)
                .run_if(resource_changed::<CommonConfig>),
        );
        // `apply_display` doesn't need a sibling observer — display config is
        // window-scoped, not camera-scoped, and the primary window already
        // exists by the time `Res<CommonConfig>` first changes.
        app.add_observer(render::apply_render_on_camera_add);
    }
}
