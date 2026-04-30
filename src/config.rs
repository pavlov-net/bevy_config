//! The [`Config`] trait and associated lifecycle types.

use core::marker::PhantomData;

use bevy_ecs::event::Event;
use bevy_ecs::resource::Resource;
use bevy_ecs::schedule::SystemSet;
use serde::{Serialize, de::DeserializeOwned};

use crate::caps::{AdapterCaps, PlatformTarget};

/// Trait every configuration type implements.
///
/// Resolution order, applied by [`crate::plugin::ConfigPlugin`]:
///
/// 1. [`Self::platform_default`] — start with platform-appropriate defaults.
/// 2. [`Self::merge`] — layer cached user overrides on top.
/// 3. [`Self::clamp_to`] — strip features the adapter doesn't support.
///
/// Clamping last (post-merge) means a user override loaded from disk cannot
/// re-enable a feature their hardware lacks.
pub trait Config: Resource + Send + Sync + 'static {
    /// On-disk shape: every field is `Option<T>` so the file only contains
    /// values that diverge from the platform default. Mirrors the populated
    /// type, but with optional fields.
    type Overrides: Default + Serialize + DeserializeOwned + Send + Sync + 'static;

    /// Conservative, platform-appropriate defaults.
    fn platform_default(target: PlatformTarget) -> Self;

    /// Apply any explicitly-set fields from `overrides` onto `self`.
    fn merge(&mut self, overrides: &Self::Overrides);

    /// Mutate `self` to fit `caps` (e.g., disable ray tracing if the adapter
    /// lacks `EXPERIMENTAL_RAY_QUERY`). Called after `merge`.
    fn clamp_to(&mut self, caps: &AdapterCaps);

    /// Project the current state into a sparse `Overrides` containing only
    /// values that differ from `platform_default(target)`. Used when saving:
    /// a fresh install with no changes serialises to an effectively-empty
    /// file.
    fn current_overrides(&self, target: PlatformTarget) -> Self::Overrides;
}

/// Fired once at startup, after the platform default has been merged with
/// cached overrides and clamped against [`AdapterCaps`].
///
/// Add an observer to react one-shot:
///
/// ```ignore
/// app.add_observer(|_: On<ConfigApplied<MyConfig>>, /* ... */| {});
/// ```
#[derive(Event, Default)]
pub struct ConfigApplied<C: Config>(pub PhantomData<C>);

/// Centralized ordering for `bevy_config` systems. Drop your own systems into
/// these sets via `.in_set(BevyConfigSet::ApplyBindings)` (or `.before(...)`)
/// to slot them into the lifecycle.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum BevyConfigSet {
    /// Bindings systems that translate `Res<C>` → engine resources, in
    /// `PostUpdate`.
    ApplyBindings,
}
