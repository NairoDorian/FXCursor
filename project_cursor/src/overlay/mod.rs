pub mod renderer;

use std::sync::Arc;
use winit::window::Window;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use crate::config::AppConfig;
use crate::overlay::renderer::OverlayRenderer;

#[cfg(windows)]
static mut PREV_WNDPROC: Option<unsafe extern "system" fn(
    windows_sys::Win32::Foundation::HWND,
    u32,
    windows_sys::Win32::Foundation::WPARAM,
    windows_sys::Win32::Foundation::LPARAM,
) -> windows_sys::Win32::Foundation::LRESULT> = None;

#[cfg(windows)]
unsafe extern "system" fn overlay_wndproc(
    hwnd: windows_sys::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows_sys::Win32::Foundation::WPARAM,
    lparam: windows_sys::Win32::Foundation::LPARAM,
) -> windows_sys::Win32::Foundation::LRESULT {
    const WM_NCHITTEST: u32 = 0x0084;
    const WM_SETCURSOR: u32 = 0x0020;
    const HTTRANSPARENT: isize = -1;

    if msg == WM_NCHITTEST {
        return HTTRANSPARENT;
    }
    if msg == WM_SETCURSOR {
        return 1; // TRUE
    }

    let prev_opt = std::ptr::addr_of!(PREV_WNDPROC).read();
    if let Some(prev) = prev_opt {
        windows_sys::Win32::UI::WindowsAndMessaging::CallWindowProcW(
            Some(prev),
            hwnd,
            msg,
            wparam,
            lparam,
        )
    } else {
        windows_sys::Win32::UI::WindowsAndMessaging::DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

#[cfg(windows)]
pub fn configure_overlay_window(window: &Window) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    if let Ok(handle) = window.window_handle() {
        if let RawWindowHandle::Win32(win32_handle) = handle.as_raw() {
            let hwnd = win32_handle.hwnd.get() as windows_sys::Win32::Foundation::HWND;
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, GWLP_WNDPROC,
                    WS_EX_TRANSPARENT, WS_EX_LAYERED, WS_EX_TOPMOST, WS_EX_TOOLWINDOW, WS_EX_NOACTIVATE, WS_EX_APPWINDOW,
                    SetWindowPos, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOACTIVATE
                };
                let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                
                let required_flags = WS_EX_TRANSPARENT as isize
                    | WS_EX_LAYERED as isize
                    | WS_EX_TOPMOST as isize 
                    | WS_EX_TOOLWINDOW as isize 
                    | WS_EX_NOACTIVATE as isize;
                let forbidden_flags = WS_EX_APPWINDOW as isize;
                
                let has_required = (style & required_flags) == required_flags;
                let has_forbidden = (style & forbidden_flags) != 0;
                
                if !has_required || has_forbidden {
                    let new_style = (style & !forbidden_flags) | required_flags;
                    log::info!("Style correction triggered: style={:#X}, new_style={:#X}", style, new_style);
                    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style);
                    
                    const HWND_TOPMOST: isize = -1;
                    SetWindowPos(
                        hwnd,
                        HWND_TOPMOST,
                        0, 0, 0, 0,
                        SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE
                    );
                    
                    use winit::platform::windows::WindowExtWindows;
                    window.set_skip_taskbar(true);
                }
                
                // Monitor and re-apply window procedure subclassing if reset by the OS/graphics stack
                let current_wndproc = GetWindowLongPtrW(hwnd, GWLP_WNDPROC);
                let target_wndproc = overlay_wndproc as *const () as isize;
                if current_wndproc != target_wndproc {
                    log::info!("WndProc reset detected. Re-subclassing overlay window...");
                    let prev_ptr = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, target_wndproc);
                    if prev_ptr != 0 && prev_ptr != target_wndproc {
                        std::ptr::addr_of_mut!(PREV_WNDPROC).write(Some(std::mem::transmute::<
                            isize,
                            unsafe extern "system" fn(
                                windows_sys::Win32::Foundation::HWND,
                                u32,
                                windows_sys::Win32::Foundation::WPARAM,
                                windows_sys::Win32::Foundation::LPARAM,
                            ) -> windows_sys::Win32::Foundation::LRESULT,
                        >(prev_ptr)));
                    }
                }
            }
        }
    }
}




pub struct OverlayWindow {
    pub window: Arc<Window>,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    pub renderer: OverlayRenderer,
}

impl OverlayWindow {
    pub fn new(
        window: Window,
        instance: &wgpu::Instance,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
    ) -> Self {
        let window = Arc::new(window);
        
        let surface = instance.create_surface(window.clone())
            .expect("Failed to create overlay surface");

        let size = window.inner_size();
        let capabilities = surface.get_capabilities(adapter);
        
        // Find format that supports transparency composite alpha blending.
        // PostMultiplied or PreMultiplied are ideal for OS-level transparent windows.
        let alpha_mode = capabilities.alpha_modes.iter()
            .copied()
            .find(|m| {
                *m == wgpu::CompositeAlphaMode::PostMultiplied 
                || *m == wgpu::CompositeAlphaMode::PreMultiplied
            })
            .unwrap_or(capabilities.alpha_modes[0]);

        let format = capabilities.formats[0]; // Usually Bgra8UnormSrgb or similar

        let surface_config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo, // VSync-aligned
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(device, &surface_config);

        let renderer = OverlayRenderer::new(device, format);

        let this = Self {
            window,
            surface,
            surface_config,
            renderer,
        };

        // Enable click-through in winit's internal window procedure
        if let Err(e) = this.window.set_cursor_hittest(false) {
            log::warn!("Winit cursor hit-test configuration failed: {:?}", e);
        }

        #[cfg(windows)]
        configure_overlay_window(&this.window);

        this
    }

    pub fn resize(&mut self, device: &Device, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(device, &self.surface_config);

            let _ = self.window.set_cursor_hittest(false);

            #[cfg(windows)]
            configure_overlay_window(&self.window);
        }
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        config: &AppConfig,
    ) {
        #[cfg(windows)]
        configure_overlay_window(&self.window);

        let output_frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => {
                log::warn!("Dropped frame in overlay due to surface error: {:?}", e);
                return;
            }
        };
        let view = output_frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer.render(
            device,
            queue,
            &view,
            self.surface_config.width,
            self.surface_config.height,
            config,
        );

        output_frame.present();
    }
}
