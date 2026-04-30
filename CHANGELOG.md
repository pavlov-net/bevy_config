# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial release.
- `AdapterCaps` resource — wgpu features and limits → plain bools/ints/enums in the main world.
- `PlatformTarget` enum and detection.
- `Config` trait with `platform_default`, `merge`, `clamp_to`, `current_overrides`.
- `ConfigBackend` trait with native (RON, atomic write) and wasm (`localStorage`) default impls.
- `ConfigPlugin<C>` lifecycle: build (load + merge), finish (caps detect + clamp), startup (`ConfigApplied` trigger).
- Universal schema `CommonConfig` covering display, render, audio (schema only), accessibility.
- `CommonBindingsPlugin` graphics bindings — window mode/vsync/resolution, anti-alias (FXAA/SMAA/TAA), MSAA, optional DLSS via cargo feature.
- `BevyConfigCamera` marker component for binding-targeted cameras.
- `BevyConfigSet` SystemSet enum centralizing ordering.
