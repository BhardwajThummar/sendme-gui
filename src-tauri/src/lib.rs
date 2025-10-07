#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// Import modules
mod android_compat;
mod events;
mod sender_state;
mod sendme; // Replace with the name of the module that contains your functions // New module for event types // Compatibility layer for Android

use sender_state::{SenderState, SharedSenderState};
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use tauri::{Emitter, State, Window};

#[cfg(not(target_os = "android"))]
use std::time::Instant;

#[cfg(not(target_os = "android"))]
use dirs;

use dotenv::dotenv;
use std::env; // Import the env module
use tauri_plugin_store;

// Define a constant for the sendme temporary directory prefix
const DIR_PREFIX: &str = ".sendme-";

lazy_static::lazy_static! {
    static ref HOME_DIR: std::path::PathBuf = get_home_dir();
    static ref CWD: std::path::PathBuf = get_temp_dir();
}

// Platform-specific path resolution
fn get_home_dir() -> std::path::PathBuf {
    #[cfg(target_os = "android")]
    {
        // On Android, use app-specific directory
        android_compat::android::get_android_external_files_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("/sdcard"))
    }
    #[cfg(not(target_os = "android"))]
    {
        dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."))
    }
}

fn get_temp_dir() -> std::path::PathBuf {
    #[cfg(target_os = "android")]
    {
        // On Android, use app-specific external files directory
        let mut path =
            android_compat::android::get_android_external_files_dir().unwrap_or_else(|_| {
                std::path::PathBuf::from("/sdcard/Android/data/com.sendme_gui_tauri_1.app/files")
            });
        path.push(format!("{}temp", DIR_PREFIX));
        path
    }
    #[cfg(not(target_os = "android"))]
    {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        home.join("Documents").join(format!("{}temp", DIR_PREFIX))
    }
}

#[tauri::command]
async fn send_file_command(
    file_path: String,
    verbose: bool,
    _state: State<'_, SharedSenderState>,
) -> Result<String, String> {
    #[cfg(target_os = "android")]
    {
        android_compat::android::send_file_minimal(file_path, verbose).await
    }

    #[cfg(not(target_os = "android"))]
    {
        match sendme::send_file_minimal(file_path, verbose, _state.clone()).await {
            Ok(ticket) => Ok(ticket),
            Err(_e) => {
                // stop sharing if it fails
                let _e = sendme::stop_sharing(_state).await;
                // Err(e.to_string())
                // Emit error event if sending failed
                Err(format!("Failed to send file"))
            }
        }
    }
}

#[tauri::command]
async fn stop_sharing_command(_state: State<'_, SharedSenderState>) -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        android_compat::android::stop_sharing()
    }

    #[cfg(not(target_os = "android"))]
    {
        match sendme::stop_sharing(_state.clone()).await {
            Ok(()) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
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
    #[cfg(target_os = "android")]
    {
        // Simplified implementation for Android
        // Emit event that download is starting
        let _ = window.emit("download_started", ());

        // Emit status update
        let _ = window.emit("download_status", "Connecting to sender...");

        // Prepare values to move into the blocking task
        let ticket_clone = ticket.clone();
        let storage_path_clone = file_storage_path.clone();

        // Call the Android-specific implementation on a blocking thread
        let join_result = tauri::async_runtime::spawn_blocking(move || {
            futures::executor::block_on(async move {
                android_compat::android::receive_file_minimal(
                    ticket_clone,
                    storage_path_clone,
                    verbose,
                )
                .await
            })
        })
        .await;

        let result = match join_result {
            Ok(res) => res,
            Err(join_err) => {
                let err_string = join_err.to_string();
                let _ = window.emit("download_error", err_string.clone());
                return Err(err_string);
            }
        };

        // Handle the result
        match result {
            Ok(path) => {
                // Emit completion event with basic statistics
                let _ = window.emit(
                    "download_completed",
                    events::DownloadCompletedEvent {
                        success: true,
                        message: "Download completed successfully".to_string(),
                        elapsed_time_ms: 0,
                        download_path: file_storage_path,
                        filename: "file".to_string(),
                        total_bytes: 0,
                        files_count: 1,
                    },
                );
                Ok(path)
            }
            Err(e) => {
                // Emit error event if download failed
                let _ = window.emit("download_error", e.to_string());
                Err(e)
            }
        }
    }

    #[cfg(not(target_os = "android"))]
    {
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
                let result =
                    sendme::receive_file_minimal(ticket, file_storage_path.clone(), verbose).await;

                // If download was successful, get file information and emit completion event
                if result.is_ok() {
                    let elapsed_ms = start_time.elapsed().as_millis() as u64;

                    // Get information about the downloaded file(s)
                    let file_info = get_downloaded_file_info(&file_storage_path);

                    // Emit completion event with statistics
                    let _ = window_clone.emit(
                        "download_completed",
                        events::DownloadCompletedEvent {
                            success: true,
                            message: "Download completed successfully".to_string(),
                            elapsed_time_ms: elapsed_ms,
                            download_path: file_storage_path,
                            filename: file_info.filename,
                            total_bytes: file_info.total_size,
                            files_count: file_info.file_count,
                        },
                    );
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
    #[cfg(target_os = "android")]
    {
        println!("[get_downloads_dir] Android: Getting downloads directory");
        // On Android, use app-specific Downloads directory
        match android_compat::android::get_android_downloads_dir() {
            Ok(path) => {
                println!("[get_downloads_dir] Android: Path obtained: {:?}", path);
                // Create the directory if it doesn't exist
                if let Err(e) = std::fs::create_dir_all(&path) {
                    eprintln!("[get_downloads_dir] Android: Failed to create downloads directory: {}", e);
                    return Err(format!("Failed to create downloads directory: {}", e));
                }
                println!("[get_downloads_dir] Android: Directory created successfully");
                let path_string = path.to_string_lossy().to_string();
                println!("[get_downloads_dir] Android: Returning path: {}", path_string);
                Ok(path_string)
            }
            Err(e) => {
                eprintln!("[get_downloads_dir] Android: Error: {}", e);
                Err(format!("Could not determine downloads directory: {}", e))
            }
        }
    }
    #[cfg(not(target_os = "android"))]
    {
        match dirs::download_dir() {
            Some(path) => Ok(path.to_string_lossy().to_string()),
            None => Err("Could not determine downloads directory".to_string()),
        }
    }
}

#[tauri::command]
fn get_file_size(path: String) -> Result<u64, String> {
    let path = Path::new(&path);
    match fs::metadata(path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                calculate_directory_size(path).map_err(|e| e.to_string())
            } else {
                Ok(metadata.len())
            }
        }
        Err(e) => Err(format!("Failed to get file size: {}", e)),
    }
}

fn calculate_directory_size(path: &Path) -> Result<u64, std::io::Error> {
    let mut total: u64 = 0;
    let mut stack = Vec::new();
    stack.push(path.to_path_buf());

    while let Some(current) = stack.pop() {
        let entries = match fs::read_dir(&current) {
            Ok(entries) => entries,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::PermissionDenied {
                    continue;
                }
                return Err(err);
            }
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            let metadata = match entry.metadata() {
                Ok(data) => data,
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::PermissionDenied {
                        continue;
                    }
                    return Err(err);
                }
            };

            if metadata.is_dir() {
                stack.push(entry_path);
            } else {
                total = total.saturating_add(metadata.len());
            }
        }
    }

    Ok(total)
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
                                    Err(e) => {
                                        eprintln!("Failed to remove {}: {}", path.display(), e)
                                    }
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Set a panic hook to attempt cleanup on a crash.
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("Application panicked: {:?}", panic_info);
        cleanup_sendme_dirs();
    }));

    //Read the environment variables from a file
    dotenv().ok(); // Read the .env file

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        // Manage our shared sender state.
        .manage(Mutex::new(SenderState::default()))
        // Initialize the store plugin
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            send_file_command,
            stop_sharing_command,
            receive_file_command,
            get_downloads_dir,
            get_file_size
        ])
        .on_window_event(|_app, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
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
