mod preview;

use preview::PreviewData;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

struct DaemonState {
    #[allow(dead_code)]
    _active_preview: Option<PreviewData>,
    preview_count: u64,
}

fn register_startup() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let s = exe.to_str().ok_or("Invalid exe path")?;
        let status = std::process::Command::new("reg")
            .args(&["add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                     "/v", "urGlance", "/t", "REG_SZ", "/d", s, "/f"])
            .status().map_err(|e| e.to_string())?;
        if status.success() { Ok(()) } else { Err("reg add failed".into()) }
    }
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
        let dir = PathBuf::from(&home).join(".config").join("autostart");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let s = exe.to_str().ok_or("Invalid exe path")?;
        std::fs::write(dir.join("urGlance.desktop"),
            format!("[Desktop Entry]\nType=Application\nName=urGlance\nComment=File preview daemon\nExec={}\nTerminal=false\nX-GNOME-Autostart-enabled=true\n", s)
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    { Err("Unsupported platform".into()) }
}

fn unregister_startup() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let status = std::process::Command::new("reg")
            .args(&["delete", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run", "/v", "urGlance", "/f"])
            .status().map_err(|e| e.to_string())?;
        if status.success() { Ok(()) } else { Err("reg delete failed".into()) }
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let f = PathBuf::from(&home).join(".config").join("autostart").join("urGlance.desktop");
            if f.exists() { std::fs::remove_file(f).map_err(|e| e.to_string())?; }
        }
        Ok(())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    { Err("Unsupported platform".into()) }
}

fn is_startup_enabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        matches!(std::process::Command::new("reg")
            .args(&["query", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run", "/v", "urGlance"])
            .output(), Ok(o) if o.status.success())
    }
    #[cfg(target_os = "linux")]
    {
        std::env::var("HOME").ok()
            .map(|h| PathBuf::from(h).join(".config").join("autostart").join("urGlance.desktop").exists())
            .unwrap_or(false)
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    { false }
}

fn url_decode(s: &str) -> String {
    let mut d = String::new();
    let mut c = s.chars();
    while let Some(ch) = c.next() {
        if ch == '%' {
            let h1 = c.next().unwrap_or('0');
            let h2 = c.next().unwrap_or('0');
            if let Ok(b) = u8::from_str_radix(&format!("{}{}", h1, h2), 16) {
                d.push(b as char);
            }
        } else if ch == '+' { d.push(' '); }
        else { d.push(ch); }
    }
    d
}

async fn start_http_server(state: Arc<Mutex<DaemonState>>) {
    let addr = "127.0.0.1:8080";
    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("FATAL: Failed to bind HTTP server on {}: {}", addr, e);
            return;
        }
    };

    println!("\n  ╔══════════════════════════════════════════════════╗");
    println!("  ║           urGlance Preview Service             ║");
    println!("  ╠══════════════════════════════════════════════════╣");
    println!("  ║  Overlay:   Select files in your file manager  ║");
    println!("  ║  Dashboard: http://127.0.0.1:8080              ║");
    println!("  ║  CLI:       urglance-preview <file>            ║");
    println!("  ║  Stop:      Ctrl+C                             ║");
    println!("  ╚══════════════════════════════════════════════════╝\n");

    loop {
        let (mut socket, _) = match listener.accept().await {
            Ok(s) => s,
            Err(_) => continue,
        };

        let st = Arc::clone(&state);

        tokio::spawn(async move {
            let mut buf = [0u8; 8192];
            let n = match socket.read(&mut buf).await {
                Ok(n) if n > 0 => n,
                _ => return,
            };

            let req = String::from_utf8_lossy(&buf[..n]);
            let line = req.lines().next().unwrap_or("");
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 { return; }
            let _method = parts[0];
            let full = parts[1];

            let (path, query) = match full.find('?') {
                Some(i) => (&full[..i], &full[i+1..]),
                None => (full, ""),
            };

            macro_rules! send_text {
                ($status:expr, $ct:expr, $body:expr) => {
                    let r = format!(
                        "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        $status, $ct, $body.len(), $body
                    );
                    let _ = socket.write_all(r.as_bytes()).await;
                };
            }

            match (parts[0], path) {
                ("GET", "/") => {
                    send_text!("HTTP/1.1 200 OK", "text/html; charset=utf-8", include_str!("index.html"));
                }
                ("GET", "/api/status") => {
                    let (startup_enabled, preview_count) = {
                        let s = st.lock().unwrap();
                        (is_startup_enabled(), s.preview_count)
                    };
                    let j = serde_json::json!({
                        "startup_enabled": startup_enabled,
                        "preview_count": preview_count,
                        "version": env!("CARGO_PKG_VERSION"),
                        "current_path": std::env::current_dir().unwrap_or_default().to_string_lossy()
                    });
                    send_text!("HTTP/1.1 200 OK", "application/json", &j.to_string());
                }
                ("GET", "/api/files") => {
                    let mut p = String::new();
                    for pair in query.split('&') {
                        let mut kv = pair.split('=');
                        if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                            if k == "path" { p = url_decode(v); }
                        }
                    }
                    let target = if p.is_empty() || p == "." {
                        std::env::current_dir().unwrap_or_default()
                    } else {
                        PathBuf::from(&p)
                    };
                    let abs = std::fs::canonicalize(&target).unwrap_or(target.clone());

                    let res = match std::fs::read_dir(&abs) {
                        Ok(entries) => {
                            let mut files = Vec::new();
                            for e in entries.flatten() {
                                let m = e.metadata();
                                files.push(serde_json::json!({
                                    "name": e.file_name().to_string_lossy(),
                                    "path": e.path().to_string_lossy(),
                                    "is_dir": m.as_ref().map(|x| x.is_dir()).unwrap_or(false),
                                    "size": m.as_ref().map(|x| x.len()).unwrap_or(0),
                                }));
                            }
                            files.sort_by(|a, b| {
                                let ad = a["is_dir"].as_bool().unwrap_or(false);
                                let bd = b["is_dir"].as_bool().unwrap_or(false);
                                match (ad, bd) {
                                    (true, false) => std::cmp::Ordering::Less,
                                    (false, true) => std::cmp::Ordering::Greater,
                                    _ => a["name"].as_str().unwrap_or("").to_lowercase()
                                        .cmp(&b["name"].as_str().unwrap_or("").to_lowercase()),
                                }
                            });
                            serde_json::json!({"current_path": abs.to_string_lossy(), "entries": files})
                        }
                        Err(e) => serde_json::json!({"error": format!("Cannot read folder: {}", e)}),
                    };
                    let status = if res.get("error").is_some() { "HTTP/1.1 400 Bad Request" } else { "HTTP/1.1 200 OK" };
                    send_text!(status, "application/json", &res.to_string());
                }
                ("GET", "/api/preview") => {
                    let mut p = String::new();
                    for pair in query.split('&') {
                        let mut kv = pair.split('=');
                        if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                            if k == "path" { p = url_decode(v); }
                        }
                    }

                    let preview = preview::extract_file_preview(&p);
                    st.lock().unwrap().preview_count += 1;

                    let j = serde_json::json!({
                        "file_type": preview.file_type,
                        "content_snippet": preview.content_snippet,
                        "metadata_summary": preview.metadata_summary,
                        "has_image": preview.preview_image_b64.is_some(),
                        "preview_image_b64": preview.preview_image_b64,
                        "success": preview.success,
                    });
                    send_text!("HTTP/1.1 200 OK", "application/json", &j.to_string());
                }
                ("POST", "/api/startup") => {
                    let mut enable = false;
                    for pair in query.split('&') {
                        let mut kv = pair.split('=');
                        if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                            if k == "enable" { enable = v == "true"; }
                        }
                    }
                    let res = if enable { register_startup() } else { unregister_startup() };
                    let j = match res {
                        Ok(_) => serde_json::json!({"success": true}),
                        Err(e) => serde_json::json!({"success": false, "error": e}),
                    };
                    send_text!("HTTP/1.1 200 OK", "application/json", &j.to_string());
                }
                _ => {
                    let _ = socket.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").await;
                }
            }
        });
    }
}

fn find_script() -> PathBuf {
    let candidates = [
        "scripts/urglance-overlay.py",
        "../scripts/urglance-overlay.py",
        "/usr/local/bin/urglance-overlay",
        "/usr/lib/urglance/urglance-overlay.py",
    ];
    // Check alongside the binary
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let nearby = parent.join("scripts/urglance-overlay.py");
            if nearby.exists() {
                return nearby;
            }
        }
    }
    // Check known paths
    for c in &candidates {
        let p = PathBuf::from(c);
        if p.exists() {
            return p;
        }
    }
    PathBuf::from("scripts/urglance-overlay.py")
}

fn launch_overlay() {
    let script_path = find_script();

    if !script_path.exists() {
        println!("  [Tip] Install overlay: cp scripts/urglance-overlay.py /usr/local/bin/urglance-overlay && pip3 install PyGObject");
        return;
    }

    match std::process::Command::new("python3")
        .arg(script_path.to_str().unwrap_or(""))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(_) => println!("  Preview overlay launched"),
        Err(e) => println!("  [Info] Could not launch overlay: {}", e),
    }
}

fn main() {
    let state = Arc::new(Mutex::new(DaemonState {
        _active_preview: None,
        preview_count: 0,
    }));

    println!("urGlance v{} - File Preview Service", env!("CARGO_PKG_VERSION"));

    // Launch the floating preview overlay (GTK popup)
    launch_overlay();

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    rt.block_on(async {
        start_http_server(state).await;
    });
}
