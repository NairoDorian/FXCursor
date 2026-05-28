pub mod panel;

use std::sync::Arc;
use winit::window::Window;
use egui_winit::State;
use egui_wgpu::Renderer;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use crate::config::AppConfig;
use crate::gui::panel::show_settings_panel;

#[cfg(windows)]
pub fn configure_config_window(window: &Window) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    if let Ok(handle) = window.window_handle() {
        if let RawWindowHandle::Win32(win32_handle) = handle.as_raw() {
            let hwnd = win32_handle.hwnd.get() as windows_sys::Win32::Foundation::HWND;
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE,
                    WS_EX_TOOLWINDOW, SetWindowPos,
                    SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_NOACTIVATE
                };
                let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                let new_style = style | WS_EX_TOOLWINDOW as isize;
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style);
                
                SetWindowPos(
                    hwnd,
                    0,
                    0, 0, 0, 0,
                    SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE
                );
            }
        }
    }
}



pub struct GuiWindow {
    pub window: Arc<Window>,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    egui_ctx: egui::Context,
    egui_state: State,
    renderer: Renderer,
}

impl GuiWindow {
    pub fn new(
        window: Window,
        instance: &wgpu::Instance,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> Self {
        let window = Arc::new(window);
        
        // Use instance.create_surface which accepts Arc<Window> under wgpu 0.19/winit 0.29
        let surface = instance.create_surface(window.clone())
            .expect("Failed to create GUI surface");

        let size = window.inner_size();
        let capabilities = surface.get_capabilities(adapter);
        let format = capabilities.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        let surface_config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(device, &surface_config);

        let egui_ctx = egui::Context::default();
        let egui_state = State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
        );

        let renderer = Renderer::new(device, format, None, 1);

        let this = Self {
            window,
            surface,
            surface_config,
            egui_ctx,
            egui_state,
            renderer,
        };

        #[cfg(windows)]
        configure_config_window(&this.window);

        this
    }

    pub fn handle_event(&mut self, event: &winit::event::WindowEvent) -> egui_winit::EventResponse {
        self.egui_state.on_window_event(&self.window, event)
    }

    pub fn resize(&mut self, device: &Device, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(device, &self.surface_config);
            #[cfg(windows)]
            configure_config_window(&self.window);
        }
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        config: &mut AppConfig,
        config_changed: &mut bool,
    ) {
        let raw_input = self.egui_state.take_egui_input(&self.window);
        self.egui_ctx.begin_frame(raw_input);
        
        show_settings_panel(&self.egui_ctx, config, config_changed);
        
        let full_output = self.egui_ctx.end_frame();
        self.egui_state.handle_platform_output(&self.window, full_output.platform_output);

        let tris = self.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

        let output_frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => {
                log::warn!("Dropped frame in GUI due to surface error: {:?}", e);
                return;
            }
        };
        let view = output_frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("egui encoder"),
        });

        // Upload egui textures to GPU
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer.update_texture(device, queue, *id, image_delta);
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.surface_config.width, self.surface_config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        self.renderer.update_buffers(device, queue, &mut encoder, &tris, &screen_descriptor);

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.05,
                            b: 0.07,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.renderer.render(&mut rpass, &tris, &screen_descriptor);
        }

        // Free textures that are no longer needed
        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        queue.submit(std::iter::once(encoder.finish()));
        output_frame.present();
    }
}
