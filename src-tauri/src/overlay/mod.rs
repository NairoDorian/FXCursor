use crate::display;
use crate::input::InputHub;
use fxcursor_protocol::AppConfig;
use fxcursor_render::OverlayRenderer;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

pub use crate::display::get_virtual_screen_bounds;

/// Upper bound on how long the loop parks while idle. Bounded so the `GetCursorPos` fallback
/// still runs when the low-level hook is muted (elevated window in the foreground) and so
/// surface-size / display changes are noticed.
const IDLE_WAIT: Duration = Duration::from_millis(100);
/// Frames rendered after the last change so every swapchain backbuffer holds the final image.
const SETTLE_FRAMES: u32 = 3;
/// How often the virtual-desktop bounds are re-checked (monitor plugged in, DPI change, ...).
const DISPLAY_POLL: Duration = Duration::from_secs(1);
/// Fallback when the OS does not report a refresh rate.
const DEFAULT_REFRESH_HZ: f32 = 60.0;

/// Frame period for the current settings: `max_fps` when set, else the display refresh rate.
fn frame_period(max_fps: u32, refresh_hz: f32) -> Duration {
    let hz = if max_fps > 0 {
        max_fps.clamp(24, 1000) as f32
    } else {
        refresh_hz.clamp(24.0, 1000.0)
    };
    Duration::from_secs_f32(1.0 / hz)
}

/// Moves and resizes the overlay window to cover `bounds` (physical pixels), then waits briefly
/// for the window system to apply it so the swapchain is created at the right size.
fn fit_overlay_window(window: &tauri::WebviewWindow, bounds: (i32, i32, u32, u32)) {
    let (vx, vy, vw, vh) = bounds;
    let _ = window.set_position(tauri::PhysicalPosition::new(vx, vy));
    let _ = window.set_size(tauri::PhysicalSize::new(vw, vh));
    let deadline = Instant::now() + Duration::from_millis(500);
    while Instant::now() < deadline {
        if let Ok(size) = window.inner_size()
            && size.width == vw
            && size.height == vh
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    match window.inner_size() {
        Ok(size) => log::info!(
            "[overlay] window at ({vx}, {vy}) {}x{} physical (requested {vw}x{vh})",
            size.width,
            size.height
        ),
        Err(e) => log::warn!("[overlay] cannot query window size: {e}"),
    }
}

pub struct OverlayState;

impl OverlayState {
    pub fn run_render_loop(
        app_handle: AppHandle,
        config: Arc<Mutex<AppConfig>>,
        input: Arc<InputHub>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let window = app_handle
            .get_webview_window("overlay")
            .ok_or("Overlay window not found")?;

        let _ = window.set_ignore_cursor_events(true);
        display::raise_timer_resolution();
        struct TimerResolutionGuard;
        impl Drop for TimerResolutionGuard {
            fn drop(&mut self) {
                display::restore_timer_resolution();
            }
        }
        let _timer_guard = TimerResolutionGuard;

        // The builder only accepts logical sizes; enforce the physical virtual-desktop rect here.
        let mut bounds = display::virtual_bounds(&app_handle);
        fit_overlay_window(&window, bounds);

        let refresh_hz = display::refresh_rate_hz().unwrap_or(DEFAULT_REFRESH_HZ);
        log::info!("[overlay] display refresh {refresh_hz:.0} Hz (frame pacing target)");

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: if cfg!(target_os = "windows") {
                wgpu::Backends::DX12 | wgpu::Backends::VULKAN
            } else if cfg!(target_os = "macos") {
                wgpu::Backends::METAL
            } else {
                wgpu::Backends::VULKAN
            },
            flags: wgpu::InstanceFlags::default(),
            backend_options: Default::default(),
            display: None,
            memory_budget_thresholds: Default::default(),
        });

        let target = unsafe {
            wgpu::SurfaceTargetUnsafe::from_display_and_window(&window, &window)
                .map_err(|e| format!("Failed to create surface target: {:?}", e))?
        };

        let surface = unsafe {
            instance
                .create_surface_unsafe(target)
                .map_err(|e| format!("Failed to create surface: {:?}", e))?
        };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        }))?;
        let adapter_info = adapter.get_info();
        log::info!(
            "[gpu] adapter '{}' backend {:?} type {:?}",
            adapter_info.name,
            adapter_info.backend,
            adapter_info.device_type
        );
        if let Some(runtime) = app_handle.try_state::<crate::RuntimeInfo>()
            && let Ok(mut slot) = runtime.gpu.lock()
        {
            *slot = Some(crate::GpuInfo {
                adapter: adapter_info.name.clone(),
                backend: format!("{:?}", adapter_info.backend),
                device_type: format!("{:?}", adapter_info.device_type),
            });
        }

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("overlay_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: Default::default(),
                experimental_features: Default::default(),
            }))?;

        let size = window
            .inner_size()
            .unwrap_or(tauri::PhysicalSize::new(bounds.2, bounds.3));
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let alpha_mode = if surface_caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
        {
            wgpu::CompositeAlphaMode::PreMultiplied
        } else if surface_caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::PostMultiplied)
        {
            wgpu::CompositeAlphaMode::PostMultiplied
        } else {
            surface_caps.alpha_modes[0]
        };

        let mut surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Mailbox,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: Default::default(),
        };

        surface.configure(&device, &surface_config);

        let mut renderer = OverlayRenderer::new(&device, surface_format);
        renderer.virtual_origin = (bounds.0 as f32, bounds.1 as f32);
        renderer.hud_rect = Some(display::primary_rect());

        // Non-Windows platforms have no OS hook yet: poll with device_query.
        #[cfg(not(target_os = "windows"))]
        let mut poller = crate::tracker::MouseTracker::new();

        let mut last_frame = Instant::now();
        let mut settle_frames = SETTLE_FRAMES;
        let mut last_config = AppConfig::default();
        let mut last_mouse = (f32::NAN, f32::NAN);
        let mut seen_generation = input.generation();
        let mut last_display_check = Instant::now();

        // Frame statistics window (published to `RuntimeInfo` every ~500 ms).
        let runtime = app_handle.try_state::<crate::RuntimeInfo>();
        let mut stats_window_start = Instant::now();
        let mut hud_last_update = Instant::now();
        let mut stats_frames: u32 = 0;
        let mut stats_cpu_ms: f32 = 0.0;
        let mut stats_last_state: &'static str = "idle";
        let publish_stats = |runtime: &Option<tauri::State<'_, crate::RuntimeInfo>>,
                             window: Duration,
                             frames: u32,
                             cpu_ms: f32,
                             state: &'static str,
                             counts: (u32, u32)| {
            let secs = window.as_secs_f32().max(1e-3);
            let fps = if frames > 0 {
                frames as f32 / secs
            } else {
                0.0
            };
            let frame_ms = if frames > 0 {
                cpu_ms / frames as f32
            } else {
                0.0
            };
            log::debug!(
                target: "fxcursor_lib::overlay::stats",
                "state={state} fps={fps:.1} cpu_ms={frame_ms:.3} capsules={} billboards={}",
                counts.0,
                counts.1
            );
            if let Some(rt) = runtime
                && let Ok(mut slot) = rt.frame.lock()
            {
                slot.fps = fps;
                slot.frame_ms = frame_ms;
                slot.state = state.to_string();
                slot.ribbon_vertices = counts.0;
                slot.instances = counts.1;
            }
        };

        loop {
            let frame_start = Instant::now();

            // ---- 1. Gather input ------------------------------------------------------------
            #[cfg(target_os = "windows")]
            crate::input::windows::poll_cursor(&input);
            #[cfg(not(target_os = "windows"))]
            {
                let (px, py, pbuttons) = poller.update();
                input.push_move(px, py);
                input.push_polled_buttons(pbuttons);
            }
            let frame = input.take_frame();

            let delta = frame_start.duration_since(last_frame).as_secs_f32();
            last_frame = frame_start;

            // ---- 2. Detect what changed -----------------------------------------------------
            let mut surface_changed = false;

            // Monitor layout / DPI changes: refit the window to the new virtual desktop.
            if frame_start.duration_since(last_display_check) >= DISPLAY_POLL {
                last_display_check = frame_start;
                let now_bounds = display::virtual_bounds(&app_handle);
                if now_bounds != bounds {
                    log::info!("[overlay] virtual desktop changed to {now_bounds:?}; refitting");
                    bounds = now_bounds;
                    fit_overlay_window(&window, bounds);
                    renderer.virtual_origin = (bounds.0 as f32, bounds.1 as f32);
                    surface_changed = true;
                }
            }

            if let Ok(current_size) = window.inner_size()
                && current_size.width > 0
                && current_size.height > 0
                && (current_size.width != surface_config.width
                    || current_size.height != surface_config.height)
            {
                surface_config.width = current_size.width;
                surface_config.height = current_size.height;
                surface.configure(&device, &surface_config);
                surface_changed = true;
            }

            let current_config = config.lock().map(|c| c.clone()).unwrap_or_default();
            let config_changed = current_config != last_config;
            if config_changed {
                last_config = current_config.clone();
            }
            let input_changed = (frame.x, frame.y) != last_mouse || !frame.clicks.is_empty();
            last_mouse = (frame.x, frame.y);

            // ---- 3. Simulate ----------------------------------------------------------------
            for click in &frame.clicks {
                renderer.spawn_click(click.button, click.x, click.y, &current_config);
            }
            renderer.update_mouse(frame.x, frame.y, delta, &current_config);
            let is_animating = renderer.is_animating(&current_config);

            // ---- 3b. Overlay snapshots requested by IPC / CLI -------------------------------
            let pending: Vec<crate::capture::CaptureRequest> = runtime
                .as_ref()
                .and_then(|rt| rt.captures.lock().ok().map(|mut q| std::mem::take(&mut *q)))
                .unwrap_or_default();
            let mut still_pending = Vec::new();
            let now = Instant::now();
            for mut req in pending {
                if !req.is_due(now) {
                    still_pending.push(req);
                    continue;
                }
                let (vx, vy, vw, vh) = bounds;
                let region = crate::capture::crop_region(
                    (frame.x, frame.y),
                    (vx as f32, vy as f32),
                    (vw, vh),
                    req.size,
                );
                let out = req.frame_path();
                match crate::capture::capture_frame(
                    &device,
                    &queue,
                    &mut renderer,
                    &current_config,
                    surface_format,
                    region,
                ) {
                    Ok(frame) => {
                        // PNG encoding is slow (especially in debug builds); doing it here would
                        // stall the physics and distort exactly the transient a burst is meant
                        // to record, so it happens on a worker thread. The caller is answered
                        // once the last file is on disk.
                        let done = req.frame_done(out.display().to_string(), now);
                        let reply = done.then(|| (req.reply.clone(), req.summary()));
                        let builder = std::thread::Builder::new().name("capture-png-worker".into());
                        let _ = builder.spawn(move || match (frame.write_png(&out), reply) {
                            (Ok(_), Some((reply, summary))) => {
                                let _ = reply.send(Ok(summary));
                            }
                            (Err(err), Some((reply, _))) => {
                                let _ = reply.send(Err(err));
                            }
                            (Err(err), None) => log::warn!("[capture] {}: {err}", out.display()),
                            (Ok(_), None) => {}
                        });
                        if !done {
                            still_pending.push(req);
                        }
                    }
                    Err(err) => {
                        let _ = req.reply.send(Err(err));
                    }
                }
                // The capture rendered into its own target; make sure the swapchain is refreshed.
                settle_frames = SETTLE_FRAMES;
            }
            let burst_pending = !still_pending.is_empty();
            if burst_pending
                && let Some(mut q) = runtime.as_ref().and_then(|rt| rt.captures.lock().ok())
            {
                q.extend(still_pending);
            }

            // A burst keeps the loop ticking at frame rate (even with the pointer still) so its
            // frames land on time.
            if is_animating || input_changed || config_changed || surface_changed || burst_pending {
                settle_frames = SETTLE_FRAMES;
            }

            // ---- 4. Idle: park until input/config activity (bounded) ------------------------
            if settle_frames == 0 {
                if stats_last_state != "idle"
                    || stats_window_start.elapsed() > Duration::from_millis(500)
                {
                    stats_last_state = "idle";
                    publish_stats(
                        &runtime,
                        stats_window_start.elapsed(),
                        0,
                        0.0,
                        "idle",
                        renderer.last_counts,
                    );
                    stats_window_start = Instant::now();
                    stats_frames = 0;
                    stats_cpu_ms = 0.0;
                }
                seen_generation = input.wait_for_activity(seen_generation, IDLE_WAIT);
                continue;
            }
            settle_frames -= 1;

            // ---- 5. Render ------------------------------------------------------------------
            let cpu_start = Instant::now();
            match surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(frame_tex)
                | wgpu::CurrentSurfaceTexture::Suboptimal(frame_tex) => {
                    let view = frame_tex
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    if current_config.enabled {
                        renderer.render(
                            &device,
                            &queue,
                            &view,
                            surface_config.width,
                            surface_config.height,
                            &current_config,
                        );
                    } else {
                        renderer.clear_active_state();
                        renderer.render_clear(&device, &queue, &view);
                    }
                    queue.present(frame_tex);
                }
                wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                    surface.configure(&device, &surface_config);
                }
                _ => {}
            }

            // ---- 6. Frame statistics ----------------------------------------------------------
            stats_frames += 1;
            stats_cpu_ms += cpu_start.elapsed().as_secs_f32() * 1000.0;
            stats_last_state = if is_animating { "active" } else { "settling" };
            let window_elapsed = stats_window_start.elapsed();
            if window_elapsed >= Duration::from_millis(500) {
                publish_stats(
                    &runtime,
                    window_elapsed,
                    stats_frames,
                    stats_cpu_ms,
                    stats_last_state,
                    renderer.last_counts,
                );
                // On-overlay HUD: refreshed at the configured rate (never faster than the
                // 500 ms statistics window).
                let hud_period = Duration::from_millis(u64::from(
                    current_config.fps_counter.refresh_rate_ms.max(100),
                ));
                if current_config.fps_counter.enabled && hud_last_update.elapsed() >= hud_period {
                    renderer
                        .set_hud_fps(stats_frames as f32 / window_elapsed.as_secs_f32().max(1e-3));
                    hud_last_update = Instant::now();
                }
                stats_window_start = Instant::now();
                stats_frames = 0;
                stats_cpu_ms = 0.0;
            }

            // ---- 7. Pace to the display (or the configured cap) -----------------------------
            let period = frame_period(current_config.general.max_fps, refresh_hz);
            let deadline = frame_start + period;
            let now = Instant::now();
            if is_animating {
                if deadline > now {
                    std::thread::sleep(deadline - now);
                }
            } else {
                // Settling: wait for more input rather than spinning.
                let wait = deadline
                    .saturating_duration_since(now)
                    .max(Duration::from_millis(1));
                seen_generation = input.wait_for_activity(seen_generation, wait);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn period_follows_display_unless_capped() {
        assert!((frame_period(0, 144.0).as_secs_f32() - 1.0 / 144.0).abs() < 1e-6);
        assert!((frame_period(60, 144.0).as_secs_f32() - 1.0 / 60.0).abs() < 1e-6);
        // Out-of-range values are clamped instead of producing absurd periods.
        assert!((frame_period(5, 60.0).as_secs_f32() - 1.0 / 24.0).abs() < 1e-6);
        assert!((frame_period(0, 0.0).as_secs_f32() - 1.0 / 24.0).abs() < 1e-6);
    }
}
