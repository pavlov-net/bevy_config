//! [`ConfigPlugin`] — wires the lifecycle for a [`Config`].

use alloc::sync::Arc;
use core::marker::PhantomData;

use bevy_app::{App, Plugin, Startup};
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Command, Commands};
use bevy_ecs::world::World;
use bevy_log::warn;

use crate::backend::ConfigBackend;
use crate::caps::{PlatformTarget, detect_caps};
use crate::config::{Config, ConfigApplied};

extern crate alloc;

/// Wires a [`Config`] type into the app.
///
/// In `build()`:
/// - read the cached overrides from the backend (none on first run);
/// - merge them onto `C::platform_default(target)`;
/// - insert the resulting populated `C` as a resource.
///
/// In `finish()`:
/// - read [`crate::caps::AdapterCaps`] from the [`bevy_render::RenderApp`] sub-app;
/// - insert it as a main-world resource;
/// - call `C::clamp_to(&caps)` against the live resource.
///
/// At `Startup`:
/// - trigger [`ConfigApplied<C>`] so observers run.
pub struct ConfigPlugin<C: Config> {
    backend: Arc<dyn ConfigBackend<C>>,
}

impl<C: Config> ConfigPlugin<C> {
    /// Wrap a backend implementation. Most callers will want
    /// [`crate::backend::FileBackend`] (native) or
    /// [`crate::backend::LocalStorageBackend`] (wasm).
    pub fn new<B: ConfigBackend<C>>(backend: B) -> Self {
        Self {
            backend: Arc::new(backend),
        }
    }
}

impl<C: Config> Plugin for ConfigPlugin<C> {
    fn build(&self, app: &mut App) {
        let target = PlatformTarget::detect();
        let mut config = C::platform_default(target);
        if let Some(overrides) = self.backend.load() {
            config.merge(&overrides);
        }
        app.insert_resource(config);
        app.insert_resource(BackendStore::<C> {
            backend: self.backend.clone(),
        });
        app.add_systems(Startup, trigger_config_applied::<C>);
    }

    fn finish(&self, app: &mut App) {
        let caps = detect_caps(app);
        app.insert_resource(caps.clone());
        if let Some(mut config) = app.world_mut().get_resource_mut::<C>() {
            config.clamp_to(&caps);
        }
    }
}

fn trigger_config_applied<C: Config>(mut commands: Commands) {
    commands.trigger(ConfigApplied::<C>(PhantomData));
}

/// Internal: holds the backend so [`SaveConfig`] can reach it from a system.
#[derive(Resource)]
pub(crate) struct BackendStore<C: Config> {
    pub(crate) backend: Arc<dyn ConfigBackend<C>>,
}

/// Persist the current state of `Res<C>` via the registered backend.
///
/// Queue with `commands.queue(SaveConfig::<MyConfig>::default())`. Errors are
/// logged via `warn!` and do not panic — saving config should never crash a
/// running game.
pub struct SaveConfig<C: Config>(PhantomData<C>);

// Manual `Default` impl: `#[derive(Default)]` would add a bogus `C: Default`
// bound that `PhantomData<C>` doesn't actually need.
impl<C: Config> Default for SaveConfig<C> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<C: Config> Command for SaveConfig<C> {
    type Out = ();

    fn apply(self, world: &mut World) {
        let target = PlatformTarget::detect();
        let overrides = match world.get_resource::<C>() {
            Some(c) => c.current_overrides(target),
            None => {
                warn!("SaveConfig: resource not present (was ConfigPlugin added?)");
                return;
            }
        };
        let backend = match world.get_resource::<BackendStore<C>>() {
            Some(b) => b.backend.clone(),
            None => {
                warn!("SaveConfig: backend store not present (was ConfigPlugin added?)");
                return;
            }
        };
        if let Err(e) = backend.store(&overrides) {
            warn!("SaveConfig: store failed: {e}");
        }
    }
}
