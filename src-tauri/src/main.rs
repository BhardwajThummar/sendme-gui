#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod sender_state;
mod sendme; // Replace with the name of the module that contains your functions
mod events; // New module for event types

use sender_state::{SenderState, SharedSenderState};
use std::sync::Mutex;
use tauri::{State, Window};
use dirs;
use std::fs;
use std::path::Path;
use std::time::Instant;

use std::env; // Import the env module
use dotenv::dotenv;

// Define a constant for the sendme temporary directory prefix
const DIR_PREFIX: &str = ".sendme-";

lazy_static::lazy_static! {
    static ref HOME_DIR: std::path::PathBuf = dirs::home_dir().unwrap();
    static ref CWD: std::path::PathBuf = HOME_DIR.join("Documents").join(format!("{}temp", DIR_PREFIX));
}

#[tauri::command]
async fn send_file_command(
    file_path: String,
    verbose: bool,
    state: State<'_, SharedSenderState>,
) -> Result<String, String> {
    match sendme::send_file_minimal(file_path, verbose, state.clone()).await {
        Ok(ticket) => Ok(ticket),
        Err(_e) =>{
            // stop sharing if it fails
            let _ = sendme::stop_sharing(state).await;
            // Err(e.to_string())
            // Emit error event if sending failed
            Err(format!("Failed to send file"))
        },
    }
}

#[tauri::command]
async fn stop_sharing_command(
    state: State<'_, SharedSenderState>,
) -> Result<(), String> {
    match sendme::stop_sharing(state.clone()).await {
        Ok(()) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

// Original function with the signature that Tauri expects
#[tauri::command]
async fn receive_file_command(
    window: Window,
    ticket: String,
    file_storage_path: String,
    verbose: bool,
) -> Result<String, String> {
    // Tauri automatically provides the window parameter to the command
    
    // Call our internal implementation that has the window parameter
    receive_file_with_stats(window, ticket, file_storage_path, verbose).await
}

// New internal function that handles the actual download with statistics
async fn receive_file_with_stats(
    window: Window,
    ticket: String,
    file_storage_path: String,
    verbose: bool,
) -> Result<String, String> {
    // Record start time for statistics
    let start_time = Instant::now();
    
    // Emit event that download is starting
    let _ = window.emit("download_started", ());
    
    // Create a window clone for use in the blocking task
    let window_clone = window.clone();
    
    // Run the non-Send future in a blocking task.
    let result = tauri::async_runtime::spawn_blocking(move || {
        // Run the original function inside the blocking task
        futures::executor::block_on(async {
            // First update status
            let _ = window_clone.emit("download_status", "Connecting to sender...");
            
            // Call the original function to do the actual download
            let result = sendme::receive_file_minimal(ticket, file_storage_path.clone(), verbose).await;
            
            // If download was successful, get file information and emit completion event
            if result.is_ok() {
                let elapsed_ms = start_time.elapsed().as_millis() as u64;
                
                // Get information about the downloaded file(s)
                let file_info = get_downloaded_file_info(&file_storage_path);
                
                // Emit completion event with statistics
                let _ = window_clone.emit("download_completed", events::DownloadCompletedEvent {
                    success: true,
                    message: "Download completed successfully".to_string(),
                    elapsed_time_ms: elapsed_ms,
                    download_path: file_storage_path,
                    filename: file_info.filename,
                    total_bytes: file_info.total_size,
                    files_count: file_info.file_count,
                });
            }
            
            result
        })
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| {
        // Emit error event if download failed
        let _ = window.emit("download_error", e.to_string());
        e.to_string()
    });
    
    result
}

// Helper struct for file information
struct DownloadedFileInfo {
    filename: String,
    total_size: u64,
    file_count: u32,
}

// Function to get information about downloaded file(s)
fn get_downloaded_file_info(path: &str) -> DownloadedFileInfo {
    let path_obj = Path::new(path);
    let mut total_size = 0;
    let mut file_count = 0;
    let mut filename = String::from("Downloaded File");
    
    // Check if path exists
    if path_obj.exists() {
        if path_obj.is_file() {
            // Single file download
            if let Ok(metadata) = fs::metadata(path_obj) {
                total_size = metadata.len();
                file_count = 1;
            }
            if let Some(name) = path_obj.file_name() {
                if let Some(name_str) = name.to_str() {
                    filename = name_str.to_string();
                }
            }
        } else if path_obj.is_dir() {
            // Directory with multiple files
            if let Ok(entries) = fs::read_dir(path_obj) {
                // Process each file in the directory
                let mut entries_vec = Vec::new();
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            total_size += metadata.len();
                            file_count += 1;
                        }
                    }
                    entries_vec.push(entry);
                }
                
                // Try to get main directory name or first file name
                if let Some(name) = path_obj.file_name() {
                    if let Some(name_str) = name.to_str() {
                        filename = name_str.to_string();
                    }
                } else if !entries_vec.is_empty() {
                    if let Some(name) = entries_vec[0].file_name().to_str() {
                        filename = name.to_string();
                    }
                }
            }
        }
    }
    
    DownloadedFileInfo {
        filename,
        total_size,
        file_count,
    }
}

#[tauri::command]
fn get_downloads_dir() -> Result<String, String> {
    match dirs::download_dir() {
        Some(path) => Ok(path.to_string_lossy().to_string()),
        None => Err("Could not determine downloads directory".to_string()),
    }
}

#[tauri::command]
fn get_file_size(path: String) -> Result<u64, String> {
    let path = Path::new(&path);
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.len()),
        Err(e) => Err(format!("Failed to get file size: {}", e))
    }
}

/// Deletes all directories in the current directory whose names start with DIR_PREFIX.
fn cleanup_sendme_dirs() {
    if let Ok(current_dir) = CWD.canonicalize() {
        match std::fs::read_dir(&current_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.starts_with(DIR_PREFIX) {
                            let path = entry.path();
                            if path.is_dir() {
                                match std::fs::remove_dir_all(&path) {
                                    Ok(_) => println!("Removed directory: {}", path.display()),
                                    Err(e) => eprintln!("Failed to remove {}: {}", path.display(), e),
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Failed to read current directory: {}", e),
        }
    } else {
        eprintln!("Could not determine the current directory for cleanup.");
    }
}

fn main() {
    // Set a panic hook to attempt cleanup on a crash.
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("Application panicked: {:?}", panic_info);
        cleanup_sendme_dirs();
    }));

    //Read the environment variables from a file
    dotenv().ok(); // Read the .env file
    
    tauri::Builder::default()
        // Manage our shared sender state.
        .manage(Mutex::new(SenderState::default()))
        .invoke_handler(tauri::generate_handler![
            send_file_command, 
            stop_sharing_command, 
            receive_file_command, 
            get_downloads_dir,
            get_file_size
        ])
        .on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested {..} = event.event() {
                cleanup_sendme_dirs();
                // api.prevent_close();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                cleanup_sendme_dirs();
                api.prevent_exit();
            }
            tauri::RunEvent::Exit => cleanup_sendme_dirs(),
            _ => {}
        })
}