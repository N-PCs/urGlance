use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Clone, Debug, Serialize)]
pub struct PreviewData {
    pub file_type: String,
    pub content_snippet: String,
    pub metadata_summary: String,
    pub preview_image_b64: Option<String>,
    pub success: bool,
}

const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(data: &[u8]) -> String {
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(BASE64_CHARS[((triple >> 18) & 0x3F) as usize] as char);
        out.push(BASE64_CHARS[((triple >> 12) & 0x3F) as usize] as char);
        out.push(if chunk.len() > 1 { BASE64_CHARS[((triple >> 6) & 0x3F) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { BASE64_CHARS[(triple & 0x3F) as usize] as char } else { '=' });
    }
    out
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["Bytes", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut i = 0;
    while size >= 1024.0 && i < 4 {
        size /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{} {}", bytes, UNITS[i])
    } else {
        format!("{:.2} {}", size, UNITS[i])
    }
}

fn format_time(meta: &std::fs::Metadata) -> String {
    if let Ok(mtime) = meta.modified() {
        let d = mtime.duration_since(UNIX_EPOCH).unwrap_or_default();
        let secs = d.as_secs() as i64;
        // Use libc localtime_r for reliable date formatting
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        let t = secs;
        unsafe {
            libc::localtime_r(&t, &mut tm);
        }
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            tm.tm_year + 1900,
            tm.tm_mon + 1,
            tm.tm_mday,
            tm.tm_hour,
            tm.tm_min,
            tm.tm_sec
        )
    } else {
        "Unknown Date".to_string()
    }
}

fn format_permissions(mode: &std::fs::Metadata) -> String {
    use std::os::unix::fs::PermissionsExt;
    let perm = mode.permissions().mode();
    let mut r = String::with_capacity(9);
    r.push(if perm & 0o400 != 0 { 'r' } else { '-' });
    r.push(if perm & 0o200 != 0 { 'w' } else { '-' });
    r.push(if perm & 0o100 != 0 { 'x' } else { '-' });
    r.push(if perm & 0o040 != 0 { 'r' } else { '-' });
    r.push(if perm & 0o020 != 0 { 'w' } else { '-' });
    r.push(if perm & 0o010 != 0 { 'x' } else { '-' });
    r.push(if perm & 0o004 != 0 { 'r' } else { '-' });
    r.push(if perm & 0o002 != 0 { 'w' } else { '-' });
    r.push(if perm & 0o001 != 0 { 'x' } else { '-' });
    r
}

fn get_metadata_string(_path: &Path, meta: &std::fs::Metadata) -> String {
    format!("Size: {} | Modified: {} | Permissions: {}",
        format_size(meta.len()), format_time(meta), format_permissions(meta))
}

fn read_text_preview(path: &Path) -> String {
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return "[Could not open file]".to_string(),
    };
    let max_chars = 4096usize;
    let max_lines = 30usize;
    let mut buf = String::with_capacity(max_chars);
    let mut line_count = 0usize;
    let mut read_buf = [0u8; 512];
    loop {
        let n = match file.read(&mut read_buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };
        let s = String::from_utf8_lossy(&read_buf[..n]);
        for ch in s.chars() {
            if buf.len() >= max_chars {
                buf.push_str("...\n[Preview truncated]");
                return buf;
            }
            if ch == '\n' {
                line_count += 1;
                if line_count > max_lines {
                    buf.push_str("\n... [Lines truncated]");
                    return buf;
                }
            }
            buf.push(ch);
        }
    }
    if buf.is_empty() { "[Empty file]".to_string() } else { buf }
}

fn generate_image_thumbnail(path: &Path) -> Option<String> {
    let img = image::open(path).ok()?;
    let thumb = if img.width() > 800 || img.height() > 800 {
        img.thumbnail(800, 800)
    } else {
        img
    };
    let mut buf = std::io::Cursor::new(Vec::new());
    thumb.write_to(&mut buf, image::ImageFormat::Jpeg).ok()?;
    let b64 = base64_encode(buf.get_ref());
    Some(format!("data:image/jpeg;base64,{}", b64))
}

fn detect_mime(path: &Path) -> &str {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "txt" | "md" => "text/plain",
        "py" => "text/x-python",
        "rs" => "text/x-rust",
        "js" => "text/javascript",
        "ts" => "text/typescript",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "json" => "application/json",
        "xml" => "application/xml",
        "toml" => "text/x-toml",
        "yaml" | "yml" => "text/x-yaml",
        "csv" => "text/csv",
        "cpp" | "cc" | "cxx" => "text/x-c++",
        "c" => "text/x-c",
        "h" | "hpp" => "text/x-c-header",
        "go" => "text/x-go",
        "sh" | "bash" => "text/x-shellscript",
        "pdf" => "application/pdf",
        "doc" | "docx" => "application/msword",
        "xls" | "xlsx" => "application/vnd.ms-excel",
        "mp3" | "flac" | "wav" | "ogg" => "audio/*",
        "mp4" | "avi" | "mkv" | "mov" => "video/*",
        _ => "application/octet-stream",
    }
}

pub fn extract_file_preview(file_path: &str) -> PreviewData {
    let path = PathBuf::from(file_path);

    if !path.exists() {
        return PreviewData {
            file_type: "Unknown / Non-existent".into(),
            content_snippet: "[File not found on disk]".into(),
            metadata_summary: "N/A".into(),
            preview_image_b64: None,
            success: false,
        };
    }

    if path.is_dir() {
        let count = fs::read_dir(&path).map(|e| e.count()).unwrap_or(0);
        return PreviewData {
            file_type: "Directory".into(),
            content_snippet: "[Directory]".into(),
            metadata_summary: format!("Items: {}", count),
            preview_image_b64: None,
            success: true,
        };
    }

    let meta = fs::metadata(&path).ok();
    let meta_str = meta.as_ref().map(|m| get_metadata_string(&path, m)).unwrap_or_default();
    let mime = detect_mime(&path);

    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();

    let category = if mime.starts_with("image/") { "Image" }
    else if mime.starts_with("audio/") { "Audio" }
    else if mime.starts_with("video/") { "Video" }
    else if mime.starts_with("text/") || ext == "toml" || ext == "csv" || ext == "yaml" || ext == "yml" || ext == "json" || ext == "xml" { "Text / Code" }
    else if ext == "pdf" { "Document" }
    else if ext == "doc" || ext == "docx" || ext == "xls" || ext == "xlsx" { "Office Document" }
    else { "Binary / Generic" };

    let display = format!("{} File ({})", category, if ext.is_empty() { "no extension" } else { &ext });

    let (snippet, image_b64) = match category {
        "Image" => {
            let b64 = generate_image_thumbnail(&path);
            (if b64.is_some() { "[Image Preview Available Below]".to_string() } else { "[Could not decode image]".to_string() }, b64)
        }
        "Text / Code" => (read_text_preview(&path), None),
        "Audio" | "Video" => {
            let sz = meta.as_ref().map(|m| format_size(m.len())).unwrap_or_default();
            (format!("[{} file - {}]", category, sz), None)
        }
        _ => {
            let txt = read_text_preview(&path);
            if txt.starts_with('[') {
                let sz = meta.as_ref().map(|m| format_size(m.len())).unwrap_or_default();
                (format!("[Binary file - {}]", sz), None)
            } else { (txt, None) }
        }
    };

    PreviewData {
        file_type: display,
        content_snippet: snippet,
        metadata_summary: meta_str,
        preview_image_b64: image_b64,
        success: true,
    }
}
