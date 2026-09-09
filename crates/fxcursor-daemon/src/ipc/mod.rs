use std::io::{BufRead, BufReader, Write};
use std::sync::Arc;
use fxcursor_protocol::{get_builtin_presets, AppConfig};
use serde::{Deserialize, Serialize};
use crate::state::StateManager;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
#[allow(clippy::large_enum_variant)]
pub enum IpcRequest {
    GetConfig,
    UpdateConfig(AppConfig),
    ToggleEnabled,
    ApplyPreset(String),
    Ping,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
#[allow(clippy::large_enum_variant)]
pub enum IpcResponse {
    Config(AppConfig),
    Toggled(bool),
    Success,
    Pong,
    Error(String),
}

pub struct IpcServer {
    state: Arc<StateManager>,
}

impl IpcServer {
    pub fn new(state: Arc<StateManager>) -> Self {
        Self { state }
    }

    pub fn start(&self) {
        let state = self.state.clone();

        std::thread::Builder::new()
            .name("ipc-server-thread".into())
            .spawn(move || {
                log::info!("[ipc] FXCursor IPC server starting...");
                #[cfg(windows)]
                Self::run_windows_named_pipe_loop(state);

                #[cfg(not(windows))]
                Self::run_unix_socket_loop(state);
            })
            .expect("Failed to spawn IPC thread");
    }

    pub fn handle_message(msg: &str, state: &Arc<StateManager>) -> String {
        let req: Result<IpcRequest, _> = serde_json::from_str(msg);
        let resp = match req {
            Ok(IpcRequest::GetConfig) => {
                let cfg = state.config.read().clone();
                IpcResponse::Config(cfg)
            }
            Ok(IpcRequest::UpdateConfig(new_cfg)) => {
                state.save_config(new_cfg.clone());
                IpcResponse::Config(new_cfg)
            }
            Ok(IpcRequest::ToggleEnabled) => {
                let mut cfg = state.config.write();
                cfg.enabled = !cfg.enabled;
                let enabled = cfg.enabled;
                state.save_config(cfg.clone());
                IpcResponse::Toggled(enabled)
            }
            Ok(IpcRequest::ApplyPreset(preset_id)) => {
                let presets = get_builtin_presets();
                if let Some(found) = presets.into_iter().find(|p| p.id == preset_id) {
                    state.save_config(found.config.clone());
                    IpcResponse::Config(found.config)
                } else {
                    IpcResponse::Error(format!("Preset '{}' not found", preset_id))
                }
            }
            Ok(IpcRequest::Ping) => IpcResponse::Pong,
            Err(e) => IpcResponse::Error(format!("Invalid IPC JSON payload: {}", e)),
        };

        serde_json::to_string(&resp).unwrap_or_else(|_| r#"{"type":"Error","payload":"Serialization error"}"#.into())
    }

    #[cfg(windows)]
    fn run_windows_named_pipe_loop(state: Arc<StateManager>) {
        use std::os::windows::io::FromRawHandle;
        use windows::core::w;
        use windows::Win32::Foundation::{
            CloseHandle, GetLastError, ERROR_PIPE_CONNECTED, INVALID_HANDLE_VALUE, HANDLE,
        };
        use windows::Win32::Storage::FileSystem::PIPE_ACCESS_DUPLEX;
        use windows::Win32::System::Pipes::{
            ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_MESSAGE,
            PIPE_TYPE_MESSAGE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
        };

        loop {
            let handle = unsafe {
                CreateNamedPipeW(
                    w!(r"\\.\pipe\fxcursor-v4"),
                    PIPE_ACCESS_DUPLEX,
                    PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
                    PIPE_UNLIMITED_INSTANCES,
                    65536,
                    65536,
                    5000,
                    None,
                )
            };

            if handle == INVALID_HANDLE_VALUE {
                log::error!("[ipc] Failed to create named pipe. Sleeping 1s.");
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }

            let connected = unsafe { ConnectNamedPipe(handle, None).is_ok() || GetLastError() == ERROR_PIPE_CONNECTED };
            if connected {
                let state_clone = state.clone();
                let raw_handle = handle.0 as usize;
                std::thread::spawn(move || {
                    let handle = HANDLE(raw_handle as _);
                    let mut file = unsafe { std::fs::File::from_raw_handle(raw_handle as _) };
                    let file_read = match file.try_clone() {
                        Ok(f) => f,
                        Err(_) => return,
                    };
                    let mut reader = BufReader::new(file_read);
                    let mut line = String::new();

                    while let Ok(n) = reader.read_line(&mut line) {
                        if n == 0 { break; }
                        let resp = Self::handle_message(line.trim(), &state_clone);
                        let _ = writeln!(file, "{}", resp);
                        let _ = file.flush();
                        line.clear();
                    }

                    // `file` owns the handle (`from_raw_handle`) and closes it on drop, so only
                    // disconnect here; closing it again would free a handle that may already
                    // have been reused by another thread.
                    unsafe {
                        let _ = DisconnectNamedPipe(handle);
                    }
                    drop(reader);
                    drop(file);
                });
            } else {
                unsafe {
                    let _ = CloseHandle(handle);
                }
            }
        }
    }

    #[cfg(not(windows))]
    fn run_unix_socket_loop(state: Arc<StateManager>) {
        use std::os::unix::net::UnixListener;

        let socket_path = std::env::temp_dir().join("fxcursor-v4.sock");
        if socket_path.exists() {
            let _ = std::fs::remove_file(&socket_path);
        }

        let listener = match UnixListener::bind(&socket_path) {
            Ok(l) => l,
            Err(e) => {
                log::error!("[ipc] Failed to bind Unix socket at {:?}: {}", socket_path, e);
                return;
            }
        };

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let state_clone = state.clone();
                    std::thread::spawn(move || {
                        let stream_clone = match stream.try_clone() {
                            Ok(s) => s,
                            Err(_) => return,
                        };
                        let mut reader = BufReader::new(stream_clone);
                        let mut line = String::new();

                        while let Ok(n) = reader.read_line(&mut line) {
                            if n == 0 { break; }
                            let resp = Self::handle_message(line.trim(), &state_clone);
                            let _ = writeln!(stream, "{}", resp);
                            let _ = stream.flush();
                            line.clear();
                        }
                    });
                }
                Err(e) => {
                    log::warn!("[ipc] Unix socket incoming error: {}", e);
                }
            }
        }
    }
}
