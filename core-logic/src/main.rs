#[cxx::bridge(namespace = "urfileorganizer")]
mod ffi {
    struct PreviewData {
        file_type: String,
        content_snippet: String,
        metadata_summary: String,
        success: bool,
    }

    unsafe extern "C++" {
        include!("urfileorganizer/src/preview.h");
        fn extract_file_preview(file_path: &str) -> PreviewData;
    }
}

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

/// Thread-safe manager to coordinate file hover interactions,
/// enforce debouncing, and handle task cancellation.
pub struct HoverPreviewManager {
    active_task: Arc<Mutex<Option<(u64, JoinHandle<()>)>>>,
    next_task_id: Arc<Mutex<u64>>,
}

impl HoverPreviewManager {
    /// Creates a new `HoverPreviewManager`.
    pub fn new() -> Self {
        Self {
            active_task: Arc::new(Mutex::new(None)),
            next_task_id: Arc::new(Mutex::new(0)),
        }
    }

    /// Handles a file hover event. Aborts any existing hover task immediately,
    /// sets up a 250ms debounce window, and executes parsing on a blocking thread pool.
    /// Returns a oneshot Receiver that yields the PreviewData when completed.
    pub fn handle_hover(&self, file_path: PathBuf) -> oneshot::Receiver<Option<ffi::PreviewData>> {
        let mut active_task_lock = self.active_task.lock().unwrap();

        // 1. Instantly abort/cancel the previous task if it is running
        if let Some((id, ref handle)) = *active_task_lock {
            println!("  [Manager] Cancelling active task #{} (Superseded by hover on: {:?})", id, file_path);
            handle.abort();
        }

        // 2. Generate a new task ID
        let mut id_lock = self.next_task_id.lock().unwrap();
        let task_id = *id_lock;
        *id_lock += 1;

        let (tx, rx) = oneshot::channel();
        let active_task_clone = Arc::clone(&self.active_task);
        println!("  [Manager] Queueing task #{} for {:?}", task_id, file_path);

        // 3. Spawn the debounce and execution task
        let handle = tokio::spawn(async move {
            // Strict 250ms debounce window
            sleep(Duration::from_millis(250)).await;

            let path_str = match file_path.to_str() {
                Some(s) => s.to_string(),
                None => {
                    eprintln!("Invalid file path encoding: {:?}", file_path);
                    let _ = tx.send(None);
                    return;
                }
            };

            println!("  [Manager] Debounce passed for task #{}. Dispatching C++ parser...", task_id);
            
            // Offload the FFI call to the dedicated OS blocking thread pool
            let result = tokio::task::spawn_blocking(move || {
                ffi::extract_file_preview(&path_str)
            }).await;

            let preview_data = match result {
                Ok(data) => Some(data),
                Err(e) => {
                    if e.is_cancelled() {
                        println!("  [Manager] Task #{} was cancelled during blocking execution.", task_id);
                    } else {
                        eprintln!("  [Manager] Task #{} failed to complete: {:?}", task_id, e);
                    }
                    None
                }
            };

            // Send result back to the awaiting HTTP handler
            let _ = tx.send(preview_data);

            // Clean up manager slot if this task finished without being superseded
            let mut lock = active_task_clone.lock().unwrap();
            if let Some((current_id, _)) = *lock {
                if current_id == task_id {
                    *lock = None;
                }
            }
        });

        *active_task_lock = Some((task_id, handle));
        rx
    }

    /// Handles a hover-off event. Instantly aborts any active task.
    pub fn handle_hover_off(&self) {
        let mut active_task_lock = self.active_task.lock().unwrap();
        if let Some((id, ref handle)) = *active_task_lock {
            println!("  [Manager] Mouse hover-off detected. Aborting active task #{}.", id);
            handle.abort();
        }
        *active_task_lock = None;
    }
}

// Structs for JSON Serialization
#[derive(serde::Serialize)]
struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
}

#[derive(serde::Serialize)]
struct PreviewDataResponse {
    file_type: String,
    content_snippet: String,
    metadata_summary: String,
    success: bool,
}

// Autostart startup controls
fn register_startup() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let current_exe = std::env::current_exe()
            .map_err(|e| format!("Failed to get current exe path: {}", e))?;
        let exe_str = current_exe.to_str()
            .ok_or_else(|| "Invalid exe path encoding".to_string())?;
        
        let status = std::process::Command::new("reg")
            .args(&[
                "add",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "urFileOrganizer",
                "/t",
                "REG_SZ",
                "/d",
                exe_str,
                "/f"
            ])
            .status()
            .map_err(|e| format!("Failed to execute reg command: {}", e))?;
            
        if status.success() {
            Ok(())
        } else {
            Err("reg command returned non-zero status".to_string())
        }
    }

    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME")
            .map_err(|_| "HOME environment variable not set".to_string())?;
        let autostart_dir = std::path::PathBuf::from(home)
            .join(".config")
            .join("autostart");
            
        std::fs::create_dir_all(&autostart_dir)
            .map_err(|e| format!("Failed to create autostart directory: {}", e))?;
            
        let desktop_file = autostart_dir.join("urfileorganizer.desktop");
        let current_exe = std::env::current_exe()
            .map_err(|e| format!("Failed to get current exe path: {}", e))?;
        let exe_str = current_exe.to_str()
            .ok_or_else(|| "Invalid exe path encoding".to_string())?;
            
        let content = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=urFileOrganizer\n\
             Comment=Core backend file preview daemon\n\
             Exec={}\n\
             Terminal=false\n\
             X-GNOME-Autostart-enabled=true\n",
            exe_str
        );
        
        std::fs::write(&desktop_file, content)
            .map_err(|e| format!("Failed to write autostart desktop file: {}", e))?;
            
        Ok(())
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err("Autostart registration not supported on this platform".to_string())
    }
}

fn unregister_startup() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let status = std::process::Command::new("reg")
            .args(&[
                "delete",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "urFileOrganizer",
                "/f"
            ])
            .status()
            .map_err(|e| format!("Failed to execute reg delete: {}", e))?;
            
        if status.success() {
            Ok(())
        } else {
            Err("reg delete command returned non-zero status".to_string())
        }
    }

    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME")
            .map_err(|_| "HOME environment variable not set".to_string())?;
        let desktop_file = std::path::PathBuf::from(home)
            .join(".config")
            .join("autostart")
            .join("urfileorganizer.desktop");
            
        if desktop_file.exists() {
            std::fs::remove_file(desktop_file)
                .map_err(|e| format!("Failed to remove desktop file: {}", e))?;
        }
        Ok(())
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err("Autostart unregistration not supported on this platform".to_string())
    }
}

fn is_startup_enabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("reg")
            .args(&[
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "urFileOrganizer"
            ])
            .output();
        match output {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let desktop_file = std::path::PathBuf::from(home)
                .join(".config")
                .join("autostart")
                .join("urfileorganizer.desktop");
            desktop_file.exists()
        } else {
            false
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        false
    }
}

// Simple URL percent decoding utility
fn url_decode(s: &str) -> String {
    let mut decoded = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let h1 = chars.next().unwrap_or('0');
            let h2 = chars.next().unwrap_or('0');
            if let Ok(byte) = u8::from_str_radix(&format!("{}{}", h1, h2), 16) {
                decoded.push(byte as char);
            }
        } else if c == '+' {
            decoded.push(' ');
        } else {
            decoded.push(c);
        }
    }
    decoded
}

async fn start_server(manager: Arc<HoverPreviewManager>) {
    let addr = "127.0.0.1:8080";
    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind server to {}: {}", addr, e);
            return;
        }
    };
    
    println!("\n========================================================");
    println!("  urFileOrganizer Background Service Running.");
    println!("  Access Glassmorphic UI Dashboard at: http://{}", addr);
    println!("  Press Ctrl+C to stop the daemon.");
    println!("========================================================\n");

    loop {
        let (mut socket, _) = match listener.accept().await {
            Ok(s) => s,
            Err(_) => continue,
        };
        
        let manager_clone = Arc::clone(&manager);
        
        tokio::spawn(async move {
            let mut buffer = [0; 8192];
            let bytes_read = match socket.read(&mut buffer).await {
                Ok(n) if n > 0 => n,
                _ => return,
            };
            
            let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);
            let mut lines = request_str.lines();
            let request_line = match lines.next() {
                Some(line) => line,
                None => return,
            };
            
            let parts: Vec<&str> = request_line.split_whitespace().collect();
            if parts.len() < 2 { return; }
            let method = parts[0];
            let full_path = parts[1];
            
            let (path_only, query) = if let Some(idx) = full_path.find('?') {
                let (p, q) = full_path.split_at(idx);
                (p, &q[1..])
            } else {
                (full_path, "")
            };
            
            // Route handlers
            if method == "GET" && path_only == "/" {
                let html = include_str!("index.html");
                let response = format!(
                    "HTTP/1.1 200 OK\r\n\
                     Content-Type: text/html; charset=utf-8\r\n\
                     Content-Length: {}\r\n\
                     Connection: close\r\n\r\n\
                     {}",
                    html.len(),
                    html
                );
                let _ = socket.write_all(response.as_bytes()).await;
            } else if method == "GET" && path_only == "/api/status" {
                let status_json = serde_json::json!({
                    "startup_enabled": is_startup_enabled(),
                    "current_path": std::env::current_dir().unwrap_or_default().to_string_lossy().to_string()
                });
                let json_str = status_json.to_string();
                let response = format!(
                    "HTTP/1.1 200 OK\r\n\
                     Content-Type: application/json\r\n\
                     Content-Length: {}\r\n\
                     Connection: close\r\n\r\n\
                     {}",
                    json_str.len(),
                    json_str
                );
                let _ = socket.write_all(response.as_bytes()).await;
            } else if method == "GET" && path_only == "/api/files" {
                let mut param_path = String::new();
                for pair in query.split('&') {
                    let mut kv = pair.split('=');
                    if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                        if k == "path" {
                            param_path = url_decode(v);
                        }
                    }
                }
                
                let target_path = if param_path.is_empty() || param_path == "." {
                    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                } else {
                    PathBuf::from(&param_path)
                };
                
                let absolute_path = std::fs::canonicalize(&target_path)
                    .unwrap_or(target_path.clone());
                
                let json_res = match std::fs::read_dir(&absolute_path) {
                    Ok(entries) => {
                        let mut file_entries = Vec::new();
                        for entry in entries {
                            if let Ok(entry) = entry {
                                let meta = entry.metadata();
                                let name = entry.file_name().to_string_lossy().to_string();
                                let path_str = entry.path().to_string_lossy().to_string();
                                let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                                let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                                file_entries.push(FileEntry {
                                    name,
                                    path: path_str,
                                    is_dir,
                                    size,
                                });
                            }
                        }
                        file_entries.sort_by(|a, b| {
                            match (a.is_dir, b.is_dir) {
                                (true, false) => std::cmp::Ordering::Less,
                                (false, true) => std::cmp::Ordering::Greater,
                                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                            }
                        });
                        serde_json::json!({
                            "current_path": absolute_path.to_string_lossy().to_string(),
                            "entries": file_entries
                        })
                    }
                    Err(e) => {
                        serde_json::json!({
                            "error": format!("Could not read folder: {}", e)
                        })
                    }
                };
                
                let json_str = json_res.to_string();
                let status_line = if json_res.get("error").is_some() {
                    "HTTP/1.1 400 Bad Request"
                } else {
                    "HTTP/1.1 200 OK"
                };
                let response = format!(
                    "{}\r\n\
                     Content-Type: application/json\r\n\
                     Content-Length: {}\r\n\
                     Connection: close\r\n\r\n\
                     {}",
                    status_line,
                    json_str.len(),
                    json_str
                );
                let _ = socket.write_all(response.as_bytes()).await;
            } else if method == "GET" && path_only == "/api/preview" {
                let mut param_path = String::new();
                for pair in query.split('&') {
                    let mut kv = pair.split('=');
                    if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                        if k == "path" {
                            param_path = url_decode(v);
                        }
                    }
                }
                
                let target_path = PathBuf::from(param_path);
                let rx = manager_clone.handle_hover(target_path);
                
                let (status_line, json_str) = match rx.await {
                    Ok(Some(preview_data)) => {
                        let res = PreviewDataResponse {
                            file_type: preview_data.file_type.to_string(),
                            content_snippet: preview_data.content_snippet.to_string(),
                            metadata_summary: preview_data.metadata_summary.to_string(),
                            success: preview_data.success,
                        };
                        ("HTTP/1.1 200 OK", serde_json::to_string(&res).unwrap_or_default())
                    }
                    _ => {
                        // Aborted / Cancelled
                        let res = PreviewDataResponse {
                            file_type: "Cancelled".to_string(),
                            content_snippet: "[Preview extraction aborted - superseded by another request]".to_string(),
                            metadata_summary: "N/A".to_string(),
                            success: false,
                        };
                        ("HTTP/1.1 200 OK", serde_json::to_string(&res).unwrap_or_default())
                    }
                };
                
                let response = format!(
                    "{}\r\n\
                     Content-Type: application/json\r\n\
                     Content-Length: {}\r\n\
                     Connection: close\r\n\r\n\
                     {}",
                    status_line,
                    json_str.len(),
                    json_str
                );
                let _ = socket.write_all(response.as_bytes()).await;
            } else if method == "POST" && path_only == "/api/startup" {
                let mut enable = false;
                for pair in query.split('&') {
                    let mut kv = pair.split('=');
                    if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                        if k == "enable" {
                            enable = v == "true";
                        }
                    }
                }
                
                let res = if enable {
                    register_startup()
                } else {
                    unregister_startup()
                };
                
                let json_res = match res {
                    Ok(_) => serde_json::json!({ "success": true }),
                    Err(e) => serde_json::json!({ "success": false, "error": e })
                };
                
                let json_str = json_res.to_string();
                let response = format!(
                    "HTTP/1.1 200 OK\r\n\
                     Content-Type: application/json\r\n\
                     Content-Length: {}\r\n\
                     Connection: close\r\n\r\n\
                     {}",
                    json_str.len(),
                    json_str
                );
                let _ = socket.write_all(response.as_bytes()).await;
            } else {
                let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                let _ = socket.write_all(response.as_bytes()).await;
            }
        });
    }
}

#[tokio::main]
async fn main() {
    let manager = Arc::new(HoverPreviewManager::new());
    
    // Serve background web server
    start_server(manager).await;
}
