#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod sender_state;
mod sendme; // Replace with the name of the module that contains your functions

use sender_state::{SenderState, SharedSenderState};
use std::sync::Mutex;
use tauri::State;
use dirs;

#[tauri::command]
async fn send_file_command(
    file_path: String,
    verbose: bool,
    state: State<'_, SharedSenderState>,
) -> Result<String, String> {
    match sendme::send_file_minimal(file_path, verbose, state).await {
        Ok(ticket) => Ok(ticket),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn stop_sharing_command(
    state: State<'_, SharedSenderState>,
) -> Result<(), String> {
    match sendme::stop_sharing(state).await {
        Ok(()) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn receive_file_command(
    ticket: String,
    file_storage_path: String,
    verbose: bool,
) -> Result<String, String> {
    // Run the non-Send future in a blocking task.
    tauri::async_runtime::spawn_blocking(move || {
        futures::executor::block_on(sendme::receive_file_minimal(ticket, file_storage_path, verbose))
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_downloads_dir() -> Result<String, String> {
    match dirs::download_dir() {
        Some(path) => Ok(path.to_string_lossy().to_string()),
        None => Err("Could not determine downloads directory".to_string()),
    }
}

/// Deletes all directories in the current directory whose names start with ".sendme-"
fn cleanup_sendme_dirs() {
    if let Ok(current_dir) = std::env::current_dir() {
        match std::fs::read_dir(&current_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.starts_with(".sendme-") {
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
    
    tauri::Builder::default()
        // Manage our shared sender state.
        .manage(Mutex::new(SenderState::default()))
        .invoke_handler(tauri::generate_handler![send_file_command, stop_sharing_command, receive_file_command, get_downloads_dir])
        .on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
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
