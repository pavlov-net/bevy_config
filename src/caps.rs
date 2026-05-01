//! Adapter capabilities and platform detection.
//!
//! [`AdapterCaps`] is a stable, plain-data summary of the wgpu adapter — bools, ints,
//! enums. It is built from `RenderAdapterInfo` + `RenderDevice` once during
//! [`Plugin::finish`](bevy_app::Plugin::finish) on
//! [`crate::caps_aware::CapsAwarePlugin`] and inserted into the main world as a
//! resource.

use bevy_app::App;
use bevy_ecs::reflect::ReflectResource;
use bevy_ecs::resource::Resource;
use bevy_reflect::Reflect;
use bevy_render::RenderApp;
use bevy_render::renderer::{RenderAdapterInfo, RenderDevice};

/// Stable summary of the active wgpu adapter, in the main world.
///
/// Construct via [`detect_caps`]; published as a resource by
/// [`crate::caps_aware::CapsAwarePlugin`].
#[derive(Resource, Reflect, Debug, Clone, Copy)]
#[reflect(Resource)]
pub struct AdapterCaps {
    /// Selected graphics backend (Vulkan, DX12, Metal, …).
    pub backend: Backend,
    /// GPU vendor, derived from the PCI vendor id.
    pub vendor: GpuVendor,
    /// `wgpu::Limits::max_texture_dimension_2d`.
    pub max_texture_dim_2d: u32,
    /// `EXPERIMENTAL_RAY_QUERY` is supported. Implies acceleration structures
    /// in wgpu 29+; consumers gating on RT support only need this flag.
    pub ray_query: bool,
    /// `TIMESTAMP_QUERY` is supported (GPU profiling).
    pub timestamp_query: bool,
    /// DLSS is reachable on this adapter (Vulkan + Nvidia). The `dlss` cargo
    /// feature must also be enabled at build time for the binding to actually
    /// attach the `bevy_anti_alias::Dlss` component.
    pub supports_dlss: bool,
}

/// Selected graphics backend.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Backend {
    Vulkan,
    Dx12,
    Metal,
    Gl,
    BrowserWebGpu,
    Other,
}

impl Backend {
    fn from_wgpu(b: wgpu_types::Backend) -> Self {
        match b {
            wgpu_types::Backend::Vulkan => Self::Vulkan,
            wgpu_types::Backend::Dx12 => Self::Dx12,
            wgpu_types::Backend::Metal => Self::Metal,
            wgpu_types::Backend::Gl => Self::Gl,
            wgpu_types::Backend::BrowserWebGpu => Self::BrowserWebGpu,
            _ => Self::Other,
        }
    }
}

/// GPU vendor inferred from the PCI vendor id.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Apple,
    Qualcomm,
    Arm,
    Other,
}

impl GpuVendor {
    fn from_pci_id(vendor: u32) -> Self {
        // Standard PCI vendor IDs.
        match vendor {
            0x10DE => Self::Nvidia,
            0x1002 | 0x1022 => Self::Amd,
            0x8086 => Self::Intel,
            0x106B => Self::Apple,
            0x5143 => Self::Qualcomm,
            0x13B5 => Self::Arm,
            _ => Self::Other,
        }
    }
}

/// Where the application is running.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlatformTarget {
    Wasm,
    WindowsDesktop,
    LinuxDesktop,
    MacDesktop,
    Other,
}

impl PlatformTarget {
    /// Detect at compile time via `cfg`.
    pub const fn detect() -> Self {
        if cfg!(target_family = "wasm") {
            Self::Wasm
        } else if cfg!(target_os = "windows") {
            Self::WindowsDesktop
        } else if cfg!(target_os = "linux") {
            Self::LinuxDesktop
        } else if cfg!(target_os = "macos") {
            Self::MacDesktop
        } else {
            Self::Other
        }
    }
}

/// Read the active adapter from the [`RenderApp`] sub-app and project it onto
/// [`AdapterCaps`]. Returns `None` for headless / `MinimalPlugins` apps where
/// no `RenderApp` is present.
///
/// Call from [`bevy_app::Plugin::finish`] — sub-app resources aren't populated
/// during `build`.
pub fn detect_caps(app: &mut App) -> Option<AdapterCaps> {
    let render_app = app.get_sub_app(RenderApp)?;
    let info = render_app.world().get_resource::<RenderAdapterInfo>()?;
    let device = render_app.world().get_resource::<RenderDevice>()?;

    let features = device.features();
    let limits = device.limits();

    let backend = Backend::from_wgpu(info.backend);
    let vendor = GpuVendor::from_pci_id(info.vendor);

    Some(AdapterCaps {
        backend,
        vendor,
        max_texture_dim_2d: limits.max_texture_dimension_2d,
        ray_query: features.contains(wgpu_types::Features::EXPERIMENTAL_RAY_QUERY),
        timestamp_query: features.contains(wgpu_types::Features::TIMESTAMP_QUERY),
        supports_dlss: matches!((backend, vendor), (Backend::Vulkan, GpuVendor::Nvidia)),
    })
}
