//! Persistence: load and store sparse `Overrides` blobs.
//!
//! `bevy_config` ships two default backends:
//!
//! - [`FileBackend`] (native) — RON file at `<config_dir>/<app_id>/config.ron`,
//!   written atomically (tmp + `sync_all` + rename).
//! - [`LocalStorageBackend`] (wasm) — JSON in `web_sys` localStorage under the
//!   key `<app_id>.bevy_config.<type>`.
//!
//! Both implement [`ConfigBackend`]. Consumers can substitute their own
//! implementation by passing it to [`crate::plugin::ConfigPlugin::new`].

use core::marker::PhantomData;

use thiserror::Error;

use crate::config::Config;

/// Errors a backend may surface from `store()`. `load()` returns `None` on any
/// error (no config + no override = "use platform defaults"), which is the
/// behavior expected on first launch.
#[derive(Debug, Error)]
pub enum BackendError {
    #[error("io error: {0}")]
    Io(String),
    #[error("serialize error: {0}")]
    Serialize(String),
    /// Deserialisation error (reading). Surfaced from `store()` only when
    /// re-reading; `load()` swallows and returns `None`.
    #[error("deserialize error: {0}")]
    Deserialize(String),
    /// localStorage unavailable / not authorized (wasm).
    #[error("unsupported: {0}")]
    Unsupported(String),
}

/// Persistence trait. Implementations are stored as a `Box<dyn ConfigBackend<C>>`
/// inside [`crate::plugin::ConfigPlugin`].
pub trait ConfigBackend<C: Config>: Send + Sync + 'static {
    /// Read the cached overrides. `None` on first run, missing file, or any
    /// failure — the plugin falls back to `<C as Config>::platform_default`.
    fn load(&self) -> Option<C::Overrides>;

    /// Write `overrides` to the underlying storage.
    #[must_use = "persisting config should not silently drop a save error"]
    fn store(&self, overrides: &C::Overrides) -> Result<(), BackendError>;
}

// ─── Native: RON file with atomic write ──────────────────────────────────────

#[cfg(not(target_family = "wasm"))]
mod native {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write as _;
    use std::path::PathBuf;

    /// File-backed RON storage at `<config_dir>/<app_id>/config.ron`.
    ///
    /// Path discovery uses [`directories::ProjectDirs`]. Writes go to
    /// `config.ron.new`, `sync_all`, then `rename` — so an interrupted write
    /// never leaves a half-written file in place of a previously-valid one.
    pub struct FileBackend<C: Config> {
        path: PathBuf,
        _marker: PhantomData<fn() -> C>,
    }

    impl<C: Config> FileBackend<C> {
        /// Create a backend rooted at the platform's config dir for
        /// `qualifier.organization.application` (the [`directories::ProjectDirs`]
        /// triple). On Linux this is `$XDG_CONFIG_HOME/<application>/`, on
        /// Windows `%APPDATA%\<organization>\<application>\config\`, on macOS
        /// `~/Library/Application Support/<qualifier>.<organization>.<application>/`.
        pub fn new(qualifier: &str, organization: &str, application: &str) -> Self {
            let dirs = directories::ProjectDirs::from(qualifier, organization, application)
                .expect("could not determine a config directory for this platform");
            let path = dirs.config_dir().join("config.ron");
            Self {
                path,
                _marker: PhantomData,
            }
        }

        /// Create a backend at an explicit path (escape hatch / tests).
        pub fn at_path(path: impl Into<PathBuf>) -> Self {
            Self {
                path: path.into(),
                _marker: PhantomData,
            }
        }
    }

    impl<C: Config> ConfigBackend<C> for FileBackend<C> {
        fn load(&self) -> Option<C::Overrides> {
            let bytes = fs::read(&self.path).ok()?;
            #[cfg(feature = "ron")]
            {
                ron::de::from_bytes::<C::Overrides>(&bytes).ok()
            }
            #[cfg(not(feature = "ron"))]
            {
                let _ = bytes;
                None
            }
        }

        fn store(&self, overrides: &C::Overrides) -> Result<(), BackendError> {
            #[cfg(feature = "ron")]
            {
                let serialized =
                    ron::ser::to_string_pretty(overrides, ron::ser::PrettyConfig::default())
                        .map_err(|e| BackendError::Serialize(e.to_string()))?;

                if let Some(dir) = self.path.parent() {
                    fs::create_dir_all(dir).map_err(|e| BackendError::Io(e.to_string()))?;
                }

                // Atomic-write: tmp → sync_all → rename.
                let tmp = self.path.with_extension("ron.new");
                {
                    let mut f = File::create(&tmp).map_err(|e| BackendError::Io(e.to_string()))?;
                    f.write_all(serialized.as_bytes())
                        .map_err(|e| BackendError::Io(e.to_string()))?;
                    f.sync_all().map_err(|e| BackendError::Io(e.to_string()))?;
                }
                fs::rename(&tmp, &self.path).map_err(|e| BackendError::Io(e.to_string()))?;
                Ok(())
            }
            #[cfg(not(feature = "ron"))]
            {
                let _ = overrides;
                Err(BackendError::Unsupported(
                    "FileBackend requires the `ron` feature".into(),
                ))
            }
        }
    }
}

#[cfg(not(target_family = "wasm"))]
pub use native::FileBackend;

// ─── Wasm: localStorage with JSON ────────────────────────────────────────────

#[cfg(target_family = "wasm")]
mod wasm {
    use super::*;

    /// Browser localStorage backend. Stores JSON under
    /// `<app_id>.bevy_config.<type>`.
    pub struct LocalStorageBackend<C: Config> {
        key: String,
        _marker: PhantomData<fn() -> C>,
    }

    impl<C: Config> LocalStorageBackend<C> {
        pub fn new(app_id: &str) -> Self {
            let type_name = core::any::type_name::<C>();
            // Sanitise type name for the storage key.
            let suffix: String = type_name
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                .collect();
            Self {
                key: format!("{app_id}.bevy_config.{suffix}"),
                _marker: PhantomData,
            }
        }

        fn storage(&self) -> Result<web_sys::Storage, BackendError> {
            web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
                .ok_or_else(|| BackendError::Unsupported("localStorage unavailable".into()))
        }
    }

    impl<C: Config> ConfigBackend<C> for LocalStorageBackend<C> {
        fn load(&self) -> Option<C::Overrides> {
            let storage = self.storage().ok()?;
            let raw = storage.get_item(&self.key).ok().flatten()?;
            serde_json::from_str(&raw).ok()
        }

        fn store(&self, overrides: &C::Overrides) -> Result<(), BackendError> {
            let storage = self.storage()?;
            let json = serde_json::to_string(overrides)
                .map_err(|e| BackendError::Serialize(e.to_string()))?;
            storage
                .set_item(&self.key, &json)
                .map_err(|e| BackendError::Io(format!("{e:?}")))
        }
    }
}

#[cfg(target_family = "wasm")]
pub use wasm::LocalStorageBackend;
