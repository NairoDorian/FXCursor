pub mod renderer;

use crate::config::AppConfig;
use crate::tracker::MouseTracker;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::Manager;

#[cfg(target_os = "windows")]
static mut PREV_WNDPROC: Option<
    unsafe extern "system" fn(
        windows_sys::Win32::Foundation::HWND,
        u32,
        windows_sys::Win32::Foundation::WPARAM,
        windows_sys::Win32::Foundation::LPARAM,
    ) -> windows_sys::Win32::Foundation::LRESULT,
> = None;

#[cfg(target_os = "windows")]
unsafe extern "system" fn overlay_wndproc(
    hwnd: windows_sys::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows_sys::Win32::Foundation::WPARAM,
    lparam: windows_sys::Win32::Foundation::LPARAM,
) -> windows_sys::Win32::Foundation::LRESULT {
    const WM_NCHITTEST: u32 = 0x0084;
    const WM_SETCURSOR: u32 = 0x0020;
    const WM_ERASEBKGND: u32 = 0x0014;
    const HTTRANSPARENT: isize = -1;

    if msg == WM_NCHITTEST {
        return HTTRANSPARENT;
    }
    if msg == WM_SETCURSOR {
        return 1;
    }
    if msg == WM_ERASEBKGND {
        return 1;
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

#[cfg(target_os = "windows")]
fn apply_overlay_window_styles(hwnd: windows_sys::Win32::Foundation::HWND) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, GWLP_WNDPROC, SetWindowPos,
        SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOACTIVATE,
        WS_EX_TRANSPARENT, WS_EX_LAYERED, WS_EX_TOPMOST, WS_EX_TOOLWINDOW,
        WS_EX_NOACTIVATE, WS_EX_APPWINDOW,
    };

    unsafe {
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
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style);

            SetWindowPos(
                hwnd,
                (-1isize) as *mut std::ffi::c_void,
                0,
                0,
                0,
                0,
                SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }

        let current_wndproc = GetWindowLongPtrW(hwnd, GWLP_WNDPROC);
        let target_wndproc = overlay_wndproc as *const () as isize;
        if current_wndproc != target_wndproc {
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

#[cfg(target_os = "windows")]
#[allow(non_camel_case_types)]
fn apply_nvidia_native_present_fix() {
    use std::ffi::c_void;
    use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

    type NvAPI_Status = i32;
    const NVAPI_OK: NvAPI_Status = 0;
    const NVAPI_ACCESS_DENIED: NvAPI_Status = -10;

    type NvDRSSessionHandle = *mut c_void;
    type NvDRSProfileHandle = *mut c_void;

    #[repr(C)]
    struct NvDRSSetting {
        version: u32,
        setting_name: [u16; 2048],
        setting_id: u32,
        setting_type: u32,
        setting_location: u32,
        is_current_predefined: u32,
        is_predefined_valid: u32,
        u32_predefined_value: u32,
        predefined_value_padding: [u16; 2048],
        u32_current_value: u32,
        current_value_padding: [u16; 2048],
    }

    type NvAPI_InitializeFn = unsafe extern "C" fn() -> NvAPI_Status;
    type NvAPI_DRS_CreateSessionFn =
        unsafe extern "C" fn(phSession: *mut NvDRSSessionHandle) -> NvAPI_Status;
    type NvAPI_DRS_DestroySessionFn =
        unsafe extern "C" fn(hSession: NvDRSSessionHandle) -> NvAPI_Status;
    type NvAPI_DRS_LoadSettingsFn =
        unsafe extern "C" fn(hSession: NvDRSSessionHandle) -> NvAPI_Status;
    type NvAPI_DRS_SaveSettingsFn =
        unsafe extern "C" fn(hSession: NvDRSSessionHandle) -> NvAPI_Status;
    type NvAPI_DRS_GetBaseProfileFn = unsafe extern "C" fn(
        hSession: NvDRSSessionHandle,
        phProfile: *mut NvDRSProfileHandle,
    ) -> NvAPI_Status;
    type NvAPI_DRS_SetSettingFn = unsafe extern "C" fn(
        hSession: NvDRSSessionHandle,
        hProfile: NvDRSProfileHandle,
        pSetting: *mut NvDRSSetting,
    ) -> NvAPI_Status;
    type NvAPI_DRS_GetSettingFn = unsafe extern "C" fn(
        hSession: NvDRSSessionHandle,
        hProfile: NvDRSProfileHandle,
        settingId: u32,
        pSetting: *mut NvDRSSetting,
    ) -> NvAPI_Status;

    unsafe {
        let dll_name: Vec<u16> = "nvapi64.dll\0".encode_utf16().collect();
        let h_module = LoadLibraryW(dll_name.as_ptr());
        if h_module.is_null() {
            return;
        }

        let proc_name = b"nvapi_QueryInterface\0";
        let query_interface_ptr = GetProcAddress(h_module, proc_name.as_ptr());
        if query_interface_ptr.is_none() {
            return;
        }

        type QueryInterfaceFn = unsafe extern "C" fn(id: u32) -> *mut c_void;
        let query_interface: QueryInterfaceFn =
            std::mem::transmute(query_interface_ptr.unwrap());

        let nvapi_initialize_ptr = query_interface(0x0150E828);
        let drs_create_session_ptr = query_interface(0x0694D52E);
        let drs_destroy_session_ptr = query_interface(0xDAD9CFF8);
        let drs_load_settings_ptr = query_interface(0x375DBD6B);
        let drs_save_settings_ptr = query_interface(0xFCBC7E14);
        let drs_get_base_profile_ptr = query_interface(0xDA8466A0);
        let drs_set_setting_ptr = query_interface(0x577DD202);
        let drs_get_setting_ptr = query_interface(0x73BF8338);

        if nvapi_initialize_ptr.is_null()
            || drs_create_session_ptr.is_null()
            || drs_destroy_session_ptr.is_null()
            || drs_load_settings_ptr.is_null()
            || drs_save_settings_ptr.is_null()
            || drs_get_base_profile_ptr.is_null()
            || drs_set_setting_ptr.is_null()
            || drs_get_setting_ptr.is_null()
        {
            log::warn!("Failed to resolve one or more NVAPI DRS functions.");
            return;
        }

        let nvapi_initialize: NvAPI_InitializeFn =
            std::mem::transmute(nvapi_initialize_ptr);
        let drs_create_session: NvAPI_DRS_CreateSessionFn =
            std::mem::transmute(drs_create_session_ptr);
        let drs_destroy_session: NvAPI_DRS_DestroySessionFn =
            std::mem::transmute(drs_destroy_session_ptr);
        let drs_load_settings: NvAPI_DRS_LoadSettingsFn =
            std::mem::transmute(drs_load_settings_ptr);
        let drs_save_settings: NvAPI_DRS_SaveSettingsFn =
            std::mem::transmute(drs_save_settings_ptr);
        let drs_get_base_profile: NvAPI_DRS_GetBaseProfileFn =
            std::mem::transmute(drs_get_base_profile_ptr);
        let drs_set_setting: NvAPI_DRS_SetSettingFn =
            std::mem::transmute(drs_set_setting_ptr);
        let drs_get_setting: NvAPI_DRS_GetSettingFn =
            std::mem::transmute(drs_get_setting_ptr);

        if nvapi_initialize() != NVAPI_OK {
            log::warn!("Failed to initialize NVAPI.");
            return;
        }

        let mut session: NvDRSSessionHandle = std::ptr::null_mut();
        if drs_create_session(&mut session) != NVAPI_OK {
            log::warn!("Failed to create NVAPI DRS session.");
            return;
        }

        if drs_load_settings(session) != NVAPI_OK {
            log::warn!("Failed to load NVAPI DRS settings.");
            let _ = drs_destroy_session(session);
            return;
        }

        let mut base_profile: NvDRSProfileHandle = std::ptr::null_mut();
        if drs_get_base_profile(session, &mut base_profile) != NVAPI_OK {
            log::warn!("Failed to retrieve base profile handle from NVAPI DRS.");
            let _ = drs_destroy_session(session);
            return;
        }

        let mut current_setting = NvDRSSetting {
            version: 0x00013020,
            setting_name: [0; 2048],
            setting_id: 0x20324987,
            setting_type: 0,
            setting_location: 0,
            is_current_predefined: 0,
            is_predefined_valid: 0,
            u32_predefined_value: 0,
            predefined_value_padding: [0; 2048],
            u32_current_value: 0xFFFFFFFF,
            current_value_padding: [0; 2048],
        };

        let mut needs_save = false;

        let get_status =
            drs_get_setting(session, base_profile, 0x20324987, &mut current_setting);
        if get_status == NVAPI_OK && current_setting.u32_current_value == 0 {
            log::info!(
                "NVAPI: Vulkan present method already set to Prefer Native globally."
            );
        } else {
            log::info!("NVAPI: Applying Vulkan present method fix (Prefer Native)...");
            let mut setting = NvDRSSetting {
                version: 0x00013020,
                setting_name: [0; 2048],
                setting_id: 0x20324987,
                setting_type: 1,
                setting_location: 0,
                is_current_predefined: 0,
                is_predefined_valid: 0,
                u32_predefined_value: 0,
                predefined_value_padding: [0; 2048],
                u32_current_value: 0,
                current_value_padding: [0; 2048],
            };

            let status = drs_set_setting(session, base_profile, &mut setting);
            if status != NVAPI_OK {
                log::warn!(
                    "Failed to set Vulkan present method setting. Status: {}",
                    status
                );
                let _ = drs_destroy_session(session);
                return;
            }
            needs_save = true;
        }

        if needs_save {
            let status = drs_save_settings(session);
            if status == NVAPI_OK {
                log::info!(
                    "Successfully applied NVIDIA Vulkan presentation fix (Prefer Native)."
                );
            } else if status == NVAPI_ACCESS_DENIED {
                log::warn!("NVIDIA driver profile write access denied. Run once as Administrator to apply the Vulkan transparency fix.");
            } else {
                log::warn!(
                    "Failed to save NVIDIA driver profile settings. Status: {}",
                    status
                );
            }
        } else {
            log::info!("NVAPI: Base profile settings already up-to-date.");
        }

        let _ = drs_destroy_session(session);
    }
}

#[cfg(not(target_os = "windows"))]
fn apply_nvidia_native_present_fix() {}

#[cfg(not(target_os = "windows"))]
fn apply_overlay_window_styles(_hwnd: usize) {}

pub struct OverlayState;

impl OverlayState {
    pub fn run_render_loop(
        app_handle: tauri::AppHandle,
        config: Arc<Mutex<AppConfig>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        apply_nvidia_native_present_fix();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });

        let adapters =
            pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()));
        log::info!("Available GPU adapters:");
        for adapter in &adapters {
            log::info!(
                "- {:?} (Backend: {:?}, Type: {:?})",
                adapter.get_info().name,
                adapter.get_info().backend,
                adapter.get_info().device_type,
            );
        }

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            }))
            .expect("Failed to acquire GPU adapter");

        log::info!(
            "Selected adapter: {:?} (Backend: {:?})",
            adapter.get_info().name,
            adapter.get_info().backend,
        );

        let (device, queue) = pollster::block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("shared gpu device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                    experimental_features: wgpu::ExperimentalFeatures::default(),
                    trace: wgpu::Trace::default(),
                },
            ))
            .expect("Failed to create GPU logical device");

        let monitor_refresh_rate: u32 = 60u32.clamp(30, 360);
        log::info!(
            "Default monitor refresh rate: {} Hz",
            monitor_refresh_rate
        );

        let active_interval = Duration::from_secs_f32(1.0 / monitor_refresh_rate as f32);
        let idle_interval = Duration::from_secs_f32(1.0 / 60.0);

        let mut surface: Option<wgpu::Surface<'_>> = None;
        let mut surface_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let mut surface_size = (1920u32, 1080u32);

        let mut last_frame_time = Instant::now();
        let mut current_interval = active_interval;
        let mut mouse_tracker = MouseTracker::new();
        let mut last_mouse_pos = (0.0f32, 0.0f32);
        let mut last_buttons = vec![false; 5];
        let mut is_animating = false;

        let mut renderer: Option<renderer::OverlayRenderer> = None;

        loop {
            let now = Instant::now();

            if surface.is_none() {
                if let Some(overlay_window) = app_handle.get_webview_window("overlay") {
                    use raw_window_handle::{HasWindowHandle, HasDisplayHandle};
                    let wh = overlay_window
                        .window_handle()
                        .map_err(|e| format!("Failed to get window handle: {}", e))?;
                    let dh = overlay_window
                        .display_handle()
                        .map_err(|e| format!("Failed to get display handle: {}", e))?;

                    let sfc = unsafe {
                        instance.create_surface_unsafe(
                            wgpu::SurfaceTargetUnsafe::RawHandle {
                                raw_display_handle: Some(dh.as_raw()),
                                raw_window_handle: wh.as_raw(),
                            },
                        )
                    }
                    .map_err(|e| format!("Failed to create surface: {}", e))?;

                    let caps = sfc.get_capabilities(&adapter);
                    surface_format = caps
                        .formats
                        .iter()
                        .copied()
                        .find(|f| f.is_srgb())
                        .unwrap_or(caps.formats[0]);

                    surface_size = {
                        let sz = overlay_window.inner_size().unwrap();
                        (sz.width, sz.height)
                    };

                    let alpha_mode = caps
                        .alpha_modes
                        .iter()
                        .copied()
                        .find(|m| {
                            *m == wgpu::CompositeAlphaMode::PostMultiplied
                                || *m == wgpu::CompositeAlphaMode::PreMultiplied
                        })
                        .unwrap_or(caps.alpha_modes[0]);

                    sfc.configure(
                        &device,
                        &wgpu::SurfaceConfiguration {
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            format: surface_format,
                            width: surface_size.0.max(1),
                            height: surface_size.1.max(1),
                            present_mode: wgpu::PresentMode::Fifo,
                            alpha_mode,
                            view_formats: vec![],
                            desired_maximum_frame_latency: 2,
                        },
                    );

                    renderer = Some(renderer::OverlayRenderer::new(
                        &device,
                        surface_format,
                    ));

                    let _ = overlay_window.set_ignore_cursor_events(true);

                    #[cfg(target_os = "windows")]
                    {
                        use raw_window_handle::RawWindowHandle;
                        if let RawWindowHandle::Win32(wh_raw) = wh.as_raw() {
                            apply_overlay_window_styles(wh_raw.hwnd.get() as _);
                        }
                    }

                    log::info!(
                        "Overlay surface created: {}x{}, format={:?}, alpha={:?}",
                        surface_size.0,
                        surface_size.1,
                        surface_format,
                        alpha_mode
                    );

                    surface = Some(sfc);
                } else {
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
            }

            if now.duration_since(last_frame_time) >= current_interval {
                last_frame_time = now;

                let (global_x, global_y, buttons) = mouse_tracker.update();
                let mouse_moved = (global_x - last_mouse_pos.0).abs() > 0.001
                    || (global_y - last_mouse_pos.1).abs() > 0.001;
                let buttons_changed = buttons != last_buttons.as_slice();

                last_mouse_pos = (global_x, global_y);
                last_buttons.clear();
                last_buttons.extend_from_slice(buttons);

                let config = config.lock().unwrap();
                let needs_redraw =
                    mouse_moved || buttons_changed || is_animating || config.enabled;

                if needs_redraw {
                    if let Some(ref sfc) = surface {
                        match sfc.get_current_texture() {
                            wgpu::CurrentSurfaceTexture::Success(frame)
                            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) =>
                        {
                            let view = frame
                                .texture
                                .create_view(&wgpu::TextureViewDescriptor::default());

                            if let Some(ref mut r) = renderer {
                                let overlay_pos = app_handle
                                    .get_webview_window("overlay")
                                    .and_then(|w| w.outer_position().ok())
                                    .unwrap_or(tauri::PhysicalPosition::new(
                                        0, 0,
                                    ));
                                let local_x = global_x - overlay_pos.x as f32;
                                let local_y = global_y - overlay_pos.y as f32;

                                r.update_physics(
                                    (local_x, local_y),
                                    &last_buttons,
                                    &config,
                                );

                                r.render(
                                    &device,
                                    &queue,
                                    &view,
                                    surface_size.0,
                                    surface_size.1,
                                    &config,
                                );

                                is_animating = r.is_animating(&config);
                            }

                            frame.present();
                            current_interval = active_interval;
                        }
                        wgpu::CurrentSurfaceTexture::Timeout
                        | wgpu::CurrentSurfaceTexture::Occluded => {
                            // skip frame
                        }
                        wgpu::CurrentSurfaceTexture::Outdated
                        | wgpu::CurrentSurfaceTexture::Lost
                        | wgpu::CurrentSurfaceTexture::Validation => {
                            log::warn!("Surface lost or outdated, recreating...");
                            surface = None;
                            renderer = None;
                            continue;
                        }
                    }
                    }

                    #[cfg(target_os = "windows")]
                    {
                        if let Some(overlay_window) =
                            app_handle.get_webview_window("overlay")
                        {
                            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
                            if let Ok(wh) = overlay_window.window_handle() {
                                if let RawWindowHandle::Win32(wh_raw) = wh.as_raw() {
                                    apply_overlay_window_styles(
                                        wh_raw.hwnd.get() as _,
                                    );
                                }
                            }
                        }
                    }
                } else {
                    current_interval = idle_interval;
                }
            }

            let next_frame_time = last_frame_time + current_interval;
            let now2 = Instant::now();
            if next_frame_time > now2 {
                std::thread::sleep(next_frame_time - now2);
            }
        }
    }
}
