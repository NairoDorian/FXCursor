#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod gpu;
pub mod input;
pub mod ipc;
pub mod overlay;
pub mod panic_log;
pub mod portable;
pub mod state;
pub mod tray;

use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use fxcursor_render::OverlayRenderer;
use crate::gpu::GpuContext;
use crate::input::InputTracker;
use crate::ipc::IpcServer;
use crate::overlay::create_overlay_window;
use crate::state::StateManager;
use crate::tray::TrayManager;

struct App {
    state: Arc<StateManager>,
    tray: Option<TrayManager>,
    window: Option<Arc<Window>>,
    gpu: Option<GpuContext>,
    renderer: Option<OverlayRenderer>,
    input_tracker: InputTracker,
    last_frame_time: Instant,
    clean_frames_remaining: u32,
    prev_left_button: bool,
    prev_right_button: bool,
    prev_middle_button: bool,
}

impl App {
    fn new() -> Self {
        portable::init();
        panic_log::init();
        let state = Arc::new(StateManager::new());
        let ipc = IpcServer::new(state.clone());
        ipc.start();

        Self {
            state,
            tray: None,
            window: None,
            gpu: None,
            renderer: None,
            input_tracker: InputTracker::new(),
            last_frame_time: Instant::now(),
            clean_frames_remaining: 3,
            prev_left_button: false,
            prev_right_button: false,
            prev_middle_button: false,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = create_overlay_window(event_loop);
            self.tray = Some(TrayManager::new());

            let gpu = pollster::block_on(GpuContext::new(window.clone()));
            #[cfg_attr(not(windows), allow(unused_mut))]
            let mut renderer = OverlayRenderer::new(&gpu.device, gpu.config.format);

            #[cfg(windows)]
            {
                let ((orig_x, orig_y), _size) = overlay::windows::win32::get_virtual_screen_geometry();
                renderer.virtual_origin = (orig_x as f32, orig_y as f32);
            }

            self.window = Some(window);
            self.gpu = Some(gpu);
            self.renderer = Some(renderer);
            self.last_frame_time = Instant::now();
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {
        if let (Some(window), Some(gpu), Some(renderer)) = (
            &self.window,
            &mut self.gpu,
            &mut self.renderer,
        ) {
            let now = Instant::now();
            let dt = now.duration_since(self.last_frame_time).as_secs_f32();
            self.last_frame_time = now;

            let config = self.state.config.read().clone();

            // 1. Poll input & clicks
            let (pos, (left, right, middle)) = self.input_tracker.poll();

            if left && !self.prev_left_button {
                renderer.spawn_click(0, pos.0, pos.1, &config);
                self.clean_frames_remaining = 3;
            }
            if right && !self.prev_right_button {
                renderer.spawn_click(1, pos.0, pos.1, &config);
                self.clean_frames_remaining = 3;
            }
            if middle && !self.prev_middle_button {
                renderer.spawn_click(2, pos.0, pos.1, &config);
                self.clean_frames_remaining = 3;
            }
            self.prev_left_button = left;
            self.prev_right_button = right;
            self.prev_middle_button = middle;

            // 2. Physics update
            renderer.update_mouse(pos.0, pos.1, dt, &config);

            let is_active = renderer.is_animating(&config);
            if is_active {
                self.clean_frames_remaining = 3;
            }

            // 3. Render if active or flushing backbuffers
            if is_active || self.clean_frames_remaining > 0 {
                match gpu.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(frame) | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                        let size = window.inner_size();
                        renderer.render(
                            &gpu.device,
                            &gpu.queue,
                            &view,
                            size.width,
                            size.height,
                            &config,
                        );
                        gpu.queue.present(frame);
                    }
                    wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                        gpu.surface.configure(&gpu.device, &gpu.config);
                    }
                    _ => {}
                }

                if !is_active && self.clean_frames_remaining > 0 {
                    self.clean_frames_remaining -= 1;
                }
            }

            // Check tray menu clicks
            if let Ok(event) = muda::MenuEvent::receiver().try_recv() {
                if let Some(tray) = &self.tray {
                    if event.id == tray.quit_item_id {
                        std::process::exit(0);
                    } else if event.id == tray.toggle_item_id {
                        let mut cfg = self.state.config.write();
                        cfg.enabled = !cfg.enabled;
                        self.state.save_config(cfg.clone());
                    } else if event.id == tray.open_item_id {
                        log::info!("[tray] Open Settings requested");
                        // Trigger opening fxcursor UI
                    }
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.resize(new_size.width, new_size.height);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Paced at display refresh rate or throttled sleep
        if let Some(renderer) = &self.renderer {
            let config = self.state.config.read();
            if renderer.is_animating(&config) || self.clean_frames_remaining > 0 {
                event_loop.set_control_flow(ControlFlow::Poll);
            } else {
                event_loop.set_control_flow(ControlFlow::wait_duration(std::time::Duration::from_millis(15)));
            }
        }
    }
}

fn main() {
    env_logger::init();
    log::info!("Starting FXCursor V4 Background Micro-Daemon...");

    let event_loop = EventLoop::new().expect("Failed to initialize event loop");
    let mut app = App::new();

    let _ = event_loop.run_app(&mut app);
}
