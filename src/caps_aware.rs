//! Caps-aware extension to `bevy_settings`.
//!
//! [`CapsAware`] is a small trait that any settings [`Resource`] can implement
//! to declare a `clamp_to(caps)` step. [`CapsAwarePlugin`] runs once in
//! [`Plugin::finish`] — after `RenderApp` is populated and after
//! `bevy_settings::PreferencesPlugin` has loaded its files — and:
//!
//! 1. detects [`AdapterCaps`] from the live wgpu adapter and inserts the
//!    resource into the main world;
//! 2. scans the type registry for every type carrying [`ReflectCapsAware`]
//!    type-data and calls its `clamp_to` against the just-detected caps.
//!
//! ## Implementing
//!
//! ```ignore
//! use bevy::prelude::*;
//! use bevy_settings::SettingsGroup;
//! use bevy_settings_plus::prelude::*;
//!
//! #[derive(Resource, SettingsGroup, Reflect, Default)]
//! #[reflect(Resource, SettingsGroup, CapsAware, Default)]
//! struct MyGameQuality {
//!     ray_tracing: bool,
//! }
//!
//! impl CapsAware for MyGameQuality {
//!     fn clamp_to(&mut self, caps: AdapterCaps) {
//!         if !caps.ray_query { self.ray_tracing = false; }
//!     }
//! }
//! ```
//!
//! Then `app.register_type::<MyGameQuality>()` (typically inside the type's
//! own `Plugin::build`) and add [`CapsAwarePlugin`] once.

use bevy_app::{App, Plugin};
use bevy_ecs::component::Mutable;
use bevy_ecs::reflect::AppTypeRegistry;
use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::resource::Resource;
use bevy_ecs::world::World;
use bevy_log::warn;
use bevy_reflect::{CreateTypeData, FromReflect, Reflect, TypePath, TypeRegistration};

use crate::caps::{AdapterCaps, detect_caps};

/// Settings types that need to be clamped against the active adapter's caps
/// implement this. Register the type with `#[reflect(CapsAware)]` so
/// [`CapsAwarePlugin`] discovers it.
///
/// `Clone + PartialEq` are required so the clamp pass can avoid bumping the
/// resource's change tick when clamping is a no-op — otherwise the next
/// `bevy_settings::SavePreferencesDeferred` would re-write the file with
/// the clamped values, silently overwriting the user's stored intent on
/// hardware that doesn't support all features.
pub trait CapsAware:
    Resource<Mutability = Mutable> + Reflect + FromReflect + TypePath + Clone + PartialEq
{
    /// Mutate `self` to fit `caps`. Called once after settings are loaded
    /// from disk and the wgpu adapter has been queried.
    fn clamp_to(&mut self, caps: AdapterCaps);
}

/// Reflection type-data for [`CapsAware`]. Add `CapsAware` to your type's
/// `#[reflect(...)]` list to install this.
#[derive(Clone)]
pub struct ReflectCapsAware {
    clamp_in_world: fn(&mut World, AdapterCaps),
}

impl<T: CapsAware> CreateTypeData<T> for ReflectCapsAware {
    fn create_type_data(_input: ()) -> Self {
        Self {
            clamp_in_world: |world, caps| {
                let candidate: T = match world.get_resource::<T>() {
                    Some(current) => {
                        let mut c = current.clone();
                        c.clamp_to(caps);
                        if *current == c {
                            return;
                        }
                        c
                    }
                    None => return,
                };
                // Immutable borrow of world released; safe to take mutable now.
                if let Some(mut r) = world.get_resource_mut::<T>() {
                    *r = candidate;
                }
            },
        }
    }

    // Mirror `ReflectSettingsGroup`'s convention: a `CapsAware` type is always
    // a `Resource`, so installing this type-data implies `ReflectResource`
    // is available too.
    fn insert_dependencies(type_registration: &mut TypeRegistration) {
        type_registration.register_type_data::<ReflectResource, T>();
    }
}

/// Detects [`AdapterCaps`] in [`Plugin::finish`] and runs the clamp pass
/// over every type registered with [`ReflectCapsAware`].
///
/// Order matters: this plugin must be added *after*
/// `bevy_render::RenderPlugin` (so `RenderApp` is populated by the time
/// `finish` runs) and *after* `bevy_settings::PreferencesPlugin` (so
/// settings have been loaded from disk before clamping). The convenience
/// [`crate::SettingsPlusPlugins`] group wires the order correctly.
///
/// If no `RenderApp` is present (headless / `MinimalPlugins`), the clamp
/// pass is skipped — settings remain as loaded from disk.
pub struct CapsAwarePlugin;

impl Plugin for CapsAwarePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AdapterCaps>();
    }

    fn finish(&self, app: &mut App) {
        let Some(caps) = detect_caps(app) else {
            warn!("AdapterCaps not detected (no RenderApp); skipping clamp pass.");
            return;
        };
        app.insert_resource(caps);

        let registry = app.world().resource::<AppTypeRegistry>().clone();
        let types = registry.read();
        let world = app.world_mut();

        for ty in types.iter() {
            if let Some(rca) = ty.data::<ReflectCapsAware>() {
                (rca.clamp_in_world)(world, caps);
            }
        }
    }
}
