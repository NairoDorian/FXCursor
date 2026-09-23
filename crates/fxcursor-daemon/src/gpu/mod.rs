//! GPU initialization, wgpu instance, adapter, device and swapchain surface configuration.

use std::sync::Arc;
use wgpu::{
    Backends, CompositeAlphaMode, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits,
    PowerPreference, PresentMode, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration,
    TextureUsages,
};
use winit::window::Window;

/// Encapsulates the wgpu graphics context and presentation swapchain.
pub struct GpuContext {
    pub instance: Instance,
    pub surface: Surface<'static>,
    pub adapter: wgpu::Adapter,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
}

impl GpuContext {
    pub async fn new(window: Arc<dyn Window>) -> Self {
        let backends = if cfg!(target_os = "windows") {
            Backends::DX12 | Backends::VULKAN
        } else if cfg!(target_os = "macos") {
            Backends::METAL
        } else {
            Backends::VULKAN
        };

        let instance = Instance::new(InstanceDescriptor {
            backends,
            flags: wgpu::InstanceFlags::default(),
            backend_options: Default::default(),
            display: None,
            memory_budget_thresholds: Default::default(),
        });

        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create wgpu surface");

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .expect("Failed to find suitable graphics adapter");

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("fxcursor_device"),
                required_features: Features::empty(),
                required_limits: Limits::default(),
                memory_hints: Default::default(),
                trace: Default::default(),
                experimental_features: Default::default(),
            })
            .await
            .expect("Failed to create wgpu device");

        let surface_caps = surface.get_capabilities(&adapter);
        // Same choices as the app's overlay (src-tauri/src/overlay/mod.rs): a plain UNORM target
        // (pre-multiplied colours stay valid for the compositor) and vsync presentation.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let alpha_mode = [CompositeAlphaMode::PreMultiplied, CompositeAlphaMode::PostMultiplied]
            .into_iter()
            .find(|m| surface_caps.alpha_modes.contains(m))
            .unwrap_or(surface_caps.alpha_modes[0]);

        let size = window.surface_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: PresentMode::Fifo,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: Default::default(),
        };

        surface.configure(&device, &config);

        Self {
            instance,
            surface,
            adapter,
            device,
            queue,
            config,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}
