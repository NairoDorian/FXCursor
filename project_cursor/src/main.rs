#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod gui;
mod overlay;
mod tracker;
mod tray;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use crate::config::AppConfig;
use crate::gui::GuiWindow;
use crate::overlay::OverlayWindow;
use crate::tracker::MouseTracker;
use crate::tray::{SystemTray, TrayAction};

fn main() {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    log::info!("Starting Cursor FX...");

    // Create the Event Loop
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);

    // 1. Spawning Window A: Config GUI
    let gui_builder = WindowBuilder::new()
        .with_title("Cursor FX Config")
        .with_inner_size(winit::dpi::LogicalSize::new(420.0, 520.0))
        .with_resizable(true);
    let gui_window_raw = gui_builder.build(&event_loop).expect("Failed to build GUI window");


    // Get screen dimensions of primary monitor for Window B (Overlay)
    let primary_monitor = event_loop.primary_monitor();
    
    let monitor_refresh_rate = primary_monitor
        .as_ref()
        .and_then(|m| m.refresh_rate_millihertz())
        .map(|mhz| (mhz as f32 / 1000.0).round() as u32)
        .unwrap_or(60)
        .clamp(30, 360);
    log::info!("Detected primary monitor refresh rate: {} Hz", monitor_refresh_rate);

    let active_interval = std::time::Duration::from_secs_f32(1.0 / monitor_refresh_rate as f32);
    let idle_interval = std::time::Duration::from_secs_f32(1.0 / 60.0);

    let (overlay_size, overlay_pos) = if let Some(ref monitor) = primary_monitor {
        (monitor.size(), monitor.position())
    } else {
        (
            winit::dpi::PhysicalSize::new(1920, 1080),
            winit::dpi::PhysicalPosition::new(0, 0),
        )
    };

    // 2. Spawning Window B: Transparent Click-Through Overlay
    #[allow(unused_mut)]
    let mut overlay_builder = WindowBuilder::new()
        .with_title("Cursor FX Overlay")
        .with_inner_size(overlay_size)
        .with_position(overlay_pos)
        .with_transparent(true)
        .with_decorations(false)
        .with_window_level(winit::window::WindowLevel::AlwaysOnTop);

    #[cfg(target_os = "windows")]
    {
        use winit::platform::windows::WindowBuilderExtWindows;
        overlay_builder = overlay_builder.with_skip_taskbar(true);
    }

    let overlay_window_raw = overlay_builder.build(&event_loop).expect("Failed to build overlay window");



    // 3. Initialize Shared WebGPU Context
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None, // No window locked to adapter
        force_fallback_adapter: false,
    })).expect("Failed to acquire GPU adapter");

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("shared gpu device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        },
        None,
    )).expect("Failed to create GPU logical device");

    // 4. Initialize Windows managers
    let mut gui_window = GuiWindow::new(gui_window_raw, &instance, &adapter, &device, &queue);
    let mut overlay_window = OverlayWindow::new(overlay_window_raw, &instance, &adapter, &device);

    // 5. Initialize State, Tracker & Tray
    let mut config = AppConfig::load_or_default();
    let mut config_changed = false;
    let mut mouse_tracker = MouseTracker::new();
    let tray = SystemTray::new();
    let mut config_window_visible = true;

    // Track active window IDs
    let gui_id = gui_window.window.id();
    let overlay_id = overlay_window.window.id();

    log::info!("Initialization complete. Running event loop...");

    let mut last_frame_time = std::time::Instant::now();
    let mut current_interval = active_interval;
    let mut last_mouse_pos = (0.0f32, 0.0f32);
    let mut last_buttons = vec![false; 5];
    let mut last_config_change_time: Option<std::time::Instant> = None;
    let mut is_animating = false;

    let run_result = event_loop.run(move |event, elwt| {
        match event {
            // Menu / System Tray event processing
            Event::AboutToWait => {
                // Poll system tray click actions
                while let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
                    if let Some(action) = tray.handle_menu_event(&event) {
                        match action {
                            TrayAction::Quit => {
                                log::info!("Quitting application via system tray...");
                                elwt.exit();
                            }
                            TrayAction::ToggleConfig => {
                                config_window_visible = !config_window_visible;
                                gui_window.window.set_visible(config_window_visible);
                                if config_window_visible {
                                    gui_window.window.focus_window();
                                }
                            }
                        }
                    }
                }

                // Smooth physics pacing (Adaptive FPS)
                let now = std::time::Instant::now();
                if now.duration_since(last_frame_time) >= current_interval {
                    last_frame_time = now;

                    // Update global cursor coordinates
                    let (global_x, global_y, buttons) = mouse_tracker.update();

                    // Check if mouse moved or buttons changed
                    let mouse_moved = (global_x - last_mouse_pos.0).abs() > 0.001 || (global_y - last_mouse_pos.1).abs() > 0.001;
                    let buttons_changed = buttons != last_buttons.as_slice();

                    last_mouse_pos = (global_x, global_y);
                    last_buttons.clear();
                    last_buttons.extend_from_slice(buttons);

                    // If config changed from GUI, request save and repaint
                    if config_changed {
                        gui_window.window.request_redraw();
                        last_config_change_time = Some(now);
                        config_changed = false;
                    }

                    // Debounced saving of config (500ms delay)
                    if let Some(change_time) = last_config_change_time {
                        if now.duration_since(change_time) >= std::time::Duration::from_millis(500) {
                            log::info!("Debounce time elapsed, saving configuration to disk...");
                            if let Err(e) = config.save() {
                                log::warn!("Failed to auto-save config: {}", e);
                            }
                            last_config_change_time = None;
                        }
                    }

                    // Convert to local overlay window space
                    let overlay_pos = overlay_window.window.inner_position().unwrap_or_default();
                    let local_x = global_x - overlay_pos.x as f32;
                    let local_y = global_y - overlay_pos.y as f32;

                    // We need to render if mouse moved, buttons changed, or there is an ongoing animation
                    let needs_redraw = mouse_moved || buttons_changed || is_animating || last_config_change_time.is_some();

                    if needs_redraw {
                        // Trigger physics update
                        overlay_window.renderer.update_physics((local_x, local_y), &last_buttons, &config);

                        // Render overlay frame directly to bypass OS event-throttling on focus loss
                        overlay_window.render(&device, &queue, &config);

                        // Query if we should keep animating in the next frames
                        is_animating = overlay_window.renderer.is_animating(&config);
                        current_interval = active_interval;
                    } else {
                        // Slow down to 60Hz idle check when inactive (no animations, no movement)
                        current_interval = idle_interval;
                    }
                }

                // Set next wake up time
                let next_frame_time = last_frame_time + current_interval;
                elwt.set_control_flow(ControlFlow::WaitUntil(next_frame_time));
            }

            Event::WindowEvent { window_id, event } => {
                if window_id == overlay_id {
                    log::trace!("Overlay WindowEvent: {:?}", event);
                }

                // Pass GUI events to egui-winit handler
                if window_id == gui_id && config_window_visible {
                    let response = gui_window.handle_event(&event);
                    if response.repaint {
                        gui_window.window.request_redraw();
                    }
                }

                match event {
                    WindowEvent::CloseRequested => {
                        if window_id == gui_id {
                            // If user closes Config panel, hide it rather than quitting
                            config_window_visible = false;
                            gui_window.window.set_visible(false);
                            log::info!("Settings window hidden. Restore from system tray.");
                        } else if window_id == overlay_id {
                            // If overlay closes, quit app
                            log::info!("Overlay window closed. Exiting...");
                            elwt.exit();
                        }
                    }

                    WindowEvent::Resized(new_size) => {
                        if window_id == gui_id {
                            gui_window.resize(&device, new_size);
                        } else if window_id == overlay_id {
                            overlay_window.resize(&device, new_size);
                        }
                    }

                    WindowEvent::ScaleFactorChanged { .. } => {
                        // In winit 0.29, ScaleFactorChanged has custom sub-properties. 
                        // To trigger resizing, we can use the window's current inner size.
                        if window_id == gui_id {
                            let size = gui_window.window.inner_size();
                            gui_window.resize(&device, size);
                        } else if window_id == overlay_id {
                            let size = overlay_window.window.inner_size();
                            overlay_window.resize(&device, size);
                        }
                    }

                    WindowEvent::RedrawRequested
                        if window_id == gui_id && config_window_visible =>
                    {
                        gui_window.render(&device, &queue, &mut config, &mut config_changed);
                    }

                    _ => {}
                }
            }

            _ => {}
        }
    });

    if let Err(e) = run_result {
        log::error!("Event loop error: {:?}", e);
    }
}
