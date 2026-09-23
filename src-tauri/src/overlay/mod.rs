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
/// Consecutive failed acquisitions (occluded / locked screen / timeout) after which a settling
/// frame counts as done anyway, so the loop can park instead of retrying forever.
const MAX_SKIPPED_FRAMES: u32 = 10;

/// Frame period for the current settings: `max_fps` when set, else the display refresh rate.
fn frame_period(max_fps: u32, refresh_hz: f32) -> Duration {
    let hz = if max_fps > 0 {
        max_fps.clamp(24, 1000) as f32
    } else {
        refresh_hz.clamp(24.0, 1000.0)
    };
    Duration::from_secs_f32(1.0 / hz)
}

/// Presentation mode for the current settings.
///
/// `Fifo` (vsync) by default, as in V3: sleep-based pacing alone is not phase-locked to the
/// display, so with `Mailbox` the sleep jitter around each vblank makes the compositor drop one
/// frame and repeat the next every so often. That reads as the trail stuttering *sometimes*.
/// `Mailbox` is only worth it when the user caps above the refresh rate (lower latency),
/// and only if the surface supports it (Metal, for one, does not).
fn choose_present_mode(
    supported: &[wgpu::PresentMode],
    max_fps: u32,
    refresh_hz: f32,
) -> wgpu::PresentMode {
    let above_refresh = max_fps > 0 && max_fps as f32 > refresh_hz + 1.0;
    if above_refresh && supported.contains(&wgpu::PresentMode::Mailbox) {
        wgpu::PresentMode::Mailbox
    } else {
        // Fifo is the one mode every surface is required to support.
        wgpu::PresentMode::Fifo
    }
}

/// True when the loop is paced by the swapchain itself: `Fifo` without a cap below the refresh
/// rate. The next backbuffer is then acquired *before* the pointer is sampled, so acquisition
/// blocks until the vblank that frees it and the frame shown one vblank later carries the
/// freshest possible input. Sleep-based pacing oversleeps by up to a timer tick; with a period
/// equal to the refresh interval that drift made the loop miss a vblank every ~20 frames at
/// 144 Hz (a repeated frame = visible judder).
fn vsync_paced(mode: wgpu::PresentMode, max_fps: u32, refresh_hz: f32) -> bool {
    let capped_below = max_fps > 0 && (max_fps as f32) < refresh_hz - 1.0;
    mode == wgpu::PresentMode::Fifo && !capped_below
}

/// Surface size for the overlay, clamped to what the device can allocate. A virtual desktop
/// larger than `max_texture_dimension_2d` (e.g. three 4K monitors side by side on an adapter
/// limited to 8192) is clipped instead of panicking in `Surface::configure`.
fn surface_extent(width: u32, height: u32, max_dim: u32) -> (u32, u32) {
    (width.clamp(1, max_dim), height.clamp(1, max_dim))
}

/// Acquires the next swapchain texture, reconfiguring on `Lost`/`Outdated`. `None` means
/// "skip this frame"; the reason is logged at debug level.
fn acquire_frame(
    surface: &wgpu::Surface<'_>,
    device: &wgpu::Device,
    surface_config: &wgpu::SurfaceConfiguration,
) -> Option<wgpu::SurfaceTexture> {
    match surface.get_current_texture() {
        wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
            Some(t)
        }
        wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
            log::debug!("[overlay] surface lost/outdated; reconfiguring");
            surface.configure(device, surface_config);
            None
        }
        other => {
            log::debug!("[overlay] frame skipped: {other:?}");
            None
        }
    }
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

        let mut refresh_hz = display::refresh_rate_hz().unwrap_or(DEFAULT_REFRESH_HZ);
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
            // Honour WGPU_* overrides (e.g. WGPU_DX12_PRESENTATION_SYSTEM=Visual, which gives a
            // DX12 swapchain the pre-multiplied alpha a transparent overlay needs).
            backend_options: wgpu::BackendOptions::from_env_or_default(),
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
                // The overlay spans the whole virtual desktop: the default 8192 px texture limit
                // is too small for 3×4K / 2×5K layouts, so ask for what the adapter offers.
                required_limits: adapter.limits(),
                memory_hints: Default::default(),
                trace: Default::default(),
                experimental_features: Default::default(),
            }))?;

        let max_dim = device.limits().max_texture_dimension_2d;
        let size = window
            .inner_size()
            .unwrap_or(tauri::PhysicalSize::new(bounds.2, bounds.3));
        let surface_caps = surface.get_capabilities(&adapter);
        // A plain UNORM target, as Windhawk's D3D mod (B8G8R8A8_UNORM): colours are authored in
        // sRGB and DWM composites pre-multiplied values in that same encoded space. On an *_Srgb
        // target the hardware encodes `c·a` after blending, so a half-transparent white glow edge
        // was stored as ~0.73 with alpha 0.5 (invalid pre-multiplied) and DWM drew it too bright.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .or_else(|| surface_caps.formats.first().copied())
            .ok_or("Surface reports no supported formats for this adapter")?;

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
        } else if surface_caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::Inherit)
        {
            wgpu::CompositeAlphaMode::Inherit
        } else {
            // An opaque surface would cover every monitor in black, above all windows.
            return Err(format!(
                "{:?} surface has no transparent composite alpha mode ({:?}); refusing to cover the desktop",
                adapter_info.backend, surface_caps.alpha_modes
            )
            .into());
        };
        log::info!("[overlay] surface {surface_format:?}, alpha {alpha_mode:?}");

        let initial_max_fps = config.lock().map(|c| c.general.max_fps).unwrap_or(0);
        let (surface_w, surface_h) = surface_extent(size.width, size.height, max_dim);
        if (surface_w, surface_h) != (size.width, size.height) {
            log::warn!(
                "[overlay] desktop {}x{} exceeds the GPU texture limit {max_dim}; clipping",
                size.width,
                size.height
            );
        }
        let mut surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: surface_w,
            height: surface_h,
            present_mode: choose_present_mode(
                &surface_caps.present_modes,
                initial_max_fps,
                refresh_hz,
            ),
            alpha_mode,
            view_formats: vec![],
            // One queued frame: with Fifo, acquisition then waits for the previous frame to
            // reach the screen, which is what phase-locks the loop to vblank.
            desired_maximum_frame_latency: 1,
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
        // True when the previous iteration parked in the idle wait (see step 3).
        let mut resumed_from_park = false;
        // Backbuffer acquired at the top of a vsync-paced iteration, presented in step 5.
        let mut pending_frame: Option<wgpu::SurfaceTexture> = None;
        // Effects were disabled and the overlay has been cleared: nothing to redraw on input.
        let mut cleared_while_disabled = false;
        // Consecutive frames that could not be acquired (see MAX_SKIPPED_FRAMES).
        let mut skipped_frames: u32 = 0;

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
            // ---- 0. Vsync pacing: wait for the next backbuffer before sampling the pointer ---
            if settle_frames > 0
                && pending_frame.is_none()
                && vsync_paced(surface_config.present_mode, last_config.general.max_fps, refresh_hz)
            {
                pending_frame = acquire_frame(&surface, &device, &surface_config);
            }
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

            let elapsed = frame_start.duration_since(last_frame).as_secs_f32();
            last_frame = frame_start;

            // ---- 2. Detect what changed -----------------------------------------------------
            let mut surface_changed = false;

            // Monitor layout / DPI changes: refit the window to the new virtual desktop. The
            // window queries go through the Tauri main thread (a blocking round-trip), so they
            // run once per DISPLAY_POLL, never per frame: a per-frame `inner_size()` stalled the
            // trail whenever the Studio kept the main thread busy (slider drags, diagnostics).
            if frame_start.duration_since(last_display_check) >= DISPLAY_POLL {
                last_display_check = frame_start;
                // Cursor schemes / pointer size change shapes behind unchanged handles.
                crate::cursor::invalidate_cache();
                // The refresh rate can change at runtime (display settings, a new primary).
                if let Some(hz) = display::refresh_rate_hz()
                    && (hz - refresh_hz).abs() > 0.5
                {
                    log::info!("[overlay] display refresh {refresh_hz:.0} → {hz:.0} Hz");
                    refresh_hz = hz;
                    let mode = choose_present_mode(
                        &surface_caps.present_modes,
                        last_config.general.max_fps,
                        refresh_hz,
                    );
                    if mode != surface_config.present_mode {
                        surface_config.present_mode = mode;
                        pending_frame = None;
                        surface.configure(&device, &surface_config);
                    }
                }
                let now_bounds = display::virtual_bounds(&app_handle);
                // A DPI change on the overlay's monitor makes the window system resize or move
                // the window even when the desktop bounds stay the same: refit then too.
                let window_rect = window
                    .outer_position()
                    .ok()
                    .zip(window.inner_size().ok())
                    .map(|(p, s)| (p.x, p.y, s.width, s.height));
                if now_bounds != bounds || window_rect.is_some_and(|r| r != bounds) {
                    log::info!(
                        "[overlay] desktop {now_bounds:?} / window {window_rect:?}; refitting"
                    );
                    bounds = now_bounds;
                    fit_overlay_window(&window, bounds);
                    renderer.virtual_origin = (bounds.0 as f32, bounds.1 as f32);
                    renderer.hud_rect = Some(display::primary_rect());
                    surface_changed = true;
                }
                if let Ok(current_size) = window.inner_size()
                    && current_size.width > 0
                    && current_size.height > 0
                {
                    let (w, h) = surface_extent(current_size.width, current_size.height, max_dim);
                    if (w, h) != (surface_config.width, surface_config.height) {
                        surface_config.width = w;
                        surface_config.height = h;
                        pending_frame = None; // cannot reconfigure while a frame is held
                        surface.configure(&device, &surface_config);
                        surface_changed = true;
                    }
                }
            }

            let current_config = config.lock().map(|c| c.clone()).unwrap_or_default();
            let config_changed = current_config != last_config;
            if config_changed {
                last_config = current_config.clone();
                // A new frame cap can move the loop above / below the refresh rate.
                let mode = choose_present_mode(
                    &surface_caps.present_modes,
                    current_config.general.max_fps,
                    refresh_hz,
                );
                if mode != surface_config.present_mode {
                    log::info!("[overlay] present mode {mode:?}");
                    surface_config.present_mode = mode;
                    pending_frame = None;
                    surface.configure(&device, &surface_config);
                    surface_changed = true;
                }
            }
            let period = frame_period(current_config.general.max_fps, refresh_hz);
            // While effects are off and the overlay is already clear, pointer motion changes
            // nothing on screen: do not wake the GPU for it.
            let input_changed = ((frame.x, frame.y) != last_mouse || !frame.clicks.is_empty())
                && !(cleared_while_disabled && !current_config.enabled);
            last_mouse = (frame.x, frame.y);
            if current_config.enabled {
                cleared_while_disabled = false;
            }

            // ---- 3a. GPU cursor bypass: refresh the extracted shape and mirror hide state --
            // Runs even while parked (the loop passes through here every IDLE_WAIT) so a
            // cursor-shape change under a still pointer repaints and the system-arrow hide
            // tracks config toggles without waiting for motion.
            let want_gpu_cursor = current_config.enabled && current_config.gpu_cursor.enabled;
            if want_gpu_cursor {
                let repaint = match crate::cursor::extract_current() {
                    crate::cursor::CursorSnapshot::Shape(shape) => {
                        renderer.set_cursor_shape(&device, &queue, &shape)
                            | renderer.set_cursor_visible(true)
                    }
                    crate::cursor::CursorSnapshot::Hidden => renderer.set_cursor_visible(false),
                    crate::cursor::CursorSnapshot::Unavailable => false,
                };
                if repaint {
                    settle_frames = SETTLE_FRAMES;
                }
                crate::cursor::set_arrow_hidden(current_config.gpu_cursor.hide_system_cursor);
            } else {
                renderer.clear_cursor_shape();
                crate::cursor::set_arrow_hidden(false);
            }

            // ---- 3. Simulate ----------------------------------------------------------------
            // Time spent parked is idle time, not motion time: the pointer position that woke
            // the loop has only just arrived. Integrating the whole park (up to IDLE_WAIT)
            // toward it would snap the trail forward on the first frame of every movement.
            let delta = if resumed_from_park {
                elapsed.min(period.as_secs_f32())
            } else {
                elapsed
            };
            resumed_from_park = false;
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
                pending_frame = None; // dropping an acquired frame without presenting discards it
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
                resumed_from_park = true;
                continue;
            }

            // ---- 5. Render ------------------------------------------------------------------
            let cpu_start = Instant::now();
            let frame_tex = pending_frame
                .take()
                .or_else(|| acquire_frame(&surface, &device, &surface_config));
            if let Some(frame_tex) = frame_tex {
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
                    cleared_while_disabled = true;
                }
                queue.present(frame_tex);
                // Only a presented frame counts toward settling: a skipped one (timeout,
                // occlusion, lost surface) must not leave a stale image on screen.
                settle_frames -= 1;
                skipped_frames = 0;
                stats_frames += 1;
                stats_cpu_ms += cpu_start.elapsed().as_secs_f32() * 1000.0;
            } else {
                // Nothing blocked in the failed acquire: back off for a frame instead of
                // spinning, and stop insisting once the surface keeps refusing (locked screen).
                skipped_frames += 1;
                if skipped_frames >= MAX_SKIPPED_FRAMES {
                    settle_frames = settle_frames.saturating_sub(1);
                }
                std::thread::sleep(period);
            }

            // ---- 6. Frame statistics ----------------------------------------------------------
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
            // Vsync-paced: the acquire at the top of the next iteration blocks until vblank.
            // Otherwise (a cap below the refresh rate, or Mailbox above it) sleep out the
            // period. Settling frames are paced the same way: waking on every hook event ran
            // the loop at the mouse report rate (up to 8 kHz).
            if !vsync_paced(surface_config.present_mode, current_config.general.max_fps, refresh_hz)
            {
                let deadline = frame_start + period;
                let now = Instant::now();
                if deadline > now {
                    std::thread::sleep(deadline - now);
                }
            }
            seen_generation = input.generation();
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

    #[test]
    fn present_mode_is_vsync_unless_capped_above_refresh() {
        use wgpu::PresentMode::{Fifo, Mailbox};
        let both = [Fifo, Mailbox];
        assert_eq!(choose_present_mode(&both, 0, 144.0), Fifo, "uncapped follows vblank");
        assert_eq!(choose_present_mode(&both, 60, 144.0), Fifo, "cap below refresh");
        assert_eq!(choose_present_mode(&both, 144, 144.0), Fifo, "cap at refresh");
        assert_eq!(choose_present_mode(&both, 240, 144.0), Mailbox, "cap above refresh");
        // Never request a mode the surface does not support.
        assert_eq!(choose_present_mode(&[Fifo], 240, 144.0), Fifo);
    }

    #[test]
    fn vsync_pacing_applies_unless_capped_below_refresh() {
        use wgpu::PresentMode::{Fifo, Mailbox};
        assert!(vsync_paced(Fifo, 0, 144.0), "uncapped default");
        assert!(vsync_paced(Fifo, 144, 144.0), "cap at the refresh rate");
        assert!(!vsync_paced(Fifo, 60, 144.0), "a lower cap sleeps out its own period");
        assert!(!vsync_paced(Mailbox, 240, 144.0), "mailbox never blocks");
    }

    #[test]
    fn surface_extent_is_clamped_to_the_device_limit() {
        assert_eq!(surface_extent(3840 * 3, 2160, 8192), (8192, 2160));
        assert_eq!(surface_extent(0, 0, 8192), (1, 1));
        assert_eq!(surface_extent(2560, 1440, 16384), (2560, 1440));
    }
}
