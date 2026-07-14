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

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
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
    pub fn handle_hover(&self, file_path: PathBuf) {
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
                    return;
                }
            };

            println!("  [Manager] Debounce passed for task #{}. Dispatching C++ parser...", task_id);
            
            // Offload the FFI call to the dedicated OS blocking thread pool
            let result = tokio::task::spawn_blocking(move || {
                ffi::extract_file_preview(&path_str)
            }).await;

            match result {
                Ok(preview_data) => {
                    println!("\n================== PREVIEW SUCCESS (Task #{}) ==================", task_id);
                    println!("File Type:        {}", preview_data.file_type);
                    println!("Snippet:\n{}", preview_data.content_snippet.trim_end());
                    println!("Metadata Summary: {}", preview_data.metadata_summary);
                    println!("Success Status:   {}", preview_data.success);
                    println!("=================================================================\n");
                }
                Err(e) => {
                    if e.is_cancelled() {
                        println!("  [Manager] Task #{} was cancelled during blocking execution.", task_id);
                    } else {
                        eprintln!("  [Manager] Task #{} failed to complete: {:?}", task_id, e);
                    }
                }
            }

            // Clean up manager slot if this task finished without being superseded
            let mut lock = active_task_clone.lock().unwrap();
            if let Some((current_id, _)) = *lock {
                if current_id == task_id {
                    *lock = None;
                }
            }
        });

        *active_task_lock = Some((task_id, handle));
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

fn create_test_files() {
    // 1. Text file with 22 lines to demonstrate line truncation (max 15 lines)
    let mut f = File::create("test_text.txt").unwrap();
    for i in 1..=22 {
        writeln!(f, "This is line {} of our test text file.", i).unwrap();
    }

    // 2. Dummy PNG file (800x600) with proper signature and IHDR layout
    let mut f = File::create("test_image.png").unwrap();
    f.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]).unwrap(); // Signature
    f.write_all(&[0x00, 0x00, 0x00, 0x0D]).unwrap(); // IHDR length (13)
    f.write_all(b"IHDR").unwrap(); // IHDR chunk type
    f.write_all(&[0x00, 0x00, 0x03, 0x20]).unwrap(); // Width: 800
    f.write_all(&[0x00, 0x00, 0x02, 0x58]).unwrap(); // Height: 600
    f.write_all(&[0x08, 0x02, 0x00, 0x00, 0x00]).unwrap(); // Additional headers

    // 3. Dummy BMP file (1024x768) with standard DIB layout
    let mut f = File::create("test_image.bmp").unwrap();
    f.write_all(b"BM").unwrap(); // Signature
    f.write_all(&[0x00; 12]).unwrap(); // Dummy size & offset
    f.write_all(&[40, 0, 0, 0]).unwrap(); // DIB Header size (40)
    f.write_all(&[0x00, 0x04, 0x00, 0x00]).unwrap(); // Width: 1024 (little-endian)
    f.write_all(&[0x00, 0x03, 0x00, 0x00]).unwrap(); // Height: 768 (little-endian)
}

fn cleanup_test_files() {
    let _ = std::fs::remove_file("test_text.txt");
    let _ = std::fs::remove_file("test_image.png");
    let _ = std::fs::remove_file("test_image.bmp");
}

#[tokio::main]
async fn main() {
    println!("=== urFileOrganizer Core Backend Demo ===");
    
    create_test_files();
    println!("Test files initialized.\n");

    let manager = HoverPreviewManager::new();

    // Simulation 1: Rapid scrolling across multiple files at 50ms intervals.
    // Debounce is 250ms, so only the final file (Cargo.toml) should compile/render.
    println!("--- Simulating Rapid Scrolling (50ms intervals, 250ms debounce) ---");
    let files = vec![
        PathBuf::from("test_text.txt"),
        PathBuf::from("test_image.png"),
        PathBuf::from("test_image.bmp"),
        PathBuf::from("non_existent_file.json"),
        PathBuf::from("Cargo.toml"),
    ];

    for file in files {
        manager.handle_hover(file);
        sleep(Duration::from_millis(50)).await;
    }

    // Wait to allow the final file to process and complete
    sleep(Duration::from_millis(500)).await;

    // Simulation 2: Hover and mouse-off before debounce expires.
    // No parsing should be performed.
    println!("--- Simulating Hover followed by immediate Mouse-Off (no parsing should run) ---");
    manager.handle_hover(PathBuf::from("test_text.txt"));
    sleep(Duration::from_millis(100)).await;
    manager.handle_hover_off();
    sleep(Duration::from_millis(500)).await;

    // Simulation 3: Individual file types testing to showcase fast parsing functionality
    println!("--- Testing Text Snippet and Truncation (test_text.txt) ---");
    manager.handle_hover(PathBuf::from("test_text.txt"));
    sleep(Duration::from_millis(500)).await;

    println!("--- Testing PNG Dimensions Parsing (test_image.png) ---");
    manager.handle_hover(PathBuf::from("test_image.png"));
    sleep(Duration::from_millis(500)).await;

    println!("--- Testing BMP Dimensions Parsing (test_image.bmp) ---");
    manager.handle_hover(PathBuf::from("test_image.bmp"));
    sleep(Duration::from_millis(500)).await;

    println!("--- Testing Resilient Fallback (non_existent_file.json) ---");
    manager.handle_hover(PathBuf::from("non_existent_file.json"));
    sleep(Duration::from_millis(500)).await;

    cleanup_test_files();
    println!("Test files cleaned up. Demo complete.");
}
