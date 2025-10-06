// Android compatibility layer
// This file provides alternative implementations for functionality that
// might not be available on Android

#[cfg(target_os = "android")]
pub mod android {
    use std::path::PathBuf;

    // Android-specific path helper
    pub fn get_android_app_data_dir() -> Result<PathBuf, String> {
        // On Android, we use the app's external files directory
        // This doesn't require storage permissions on Android 10+
        if let Some(data_dir) = std::env::var_os("ANDROID_DATA") {
            let data_path = PathBuf::from(data_dir);
            Ok(data_path)
        } else {
            // Fallback to a relative path within the app
            Ok(PathBuf::from("/data/data/com.sendme_gui_tauri_1.app/files"))
        }
    }

    pub fn get_android_external_files_dir() -> Result<PathBuf, String> {
        // Get the external files directory for the app
        // Path format: /storage/emulated/0/Android/data/<package>/files
        if let Ok(external_storage) = std::env::var("EXTERNAL_STORAGE") {
            let mut path = PathBuf::from(external_storage);
            path.push("Android/data/com.sendme_gui_tauri_1.app/files");
            Ok(path)
        } else {
            // Fallback
            Ok(PathBuf::from(
                "/sdcard/Android/data/com.sendme_gui_tauri_1.app/files",
            ))
        }
    }

    pub fn get_android_downloads_dir() -> Result<PathBuf, String> {
        // Get the Downloads directory in app-specific external storage
        let mut external_files = get_android_external_files_dir()?;
        external_files.push("Downloads");
        Ok(external_files)
    }

    // Provide Android implementations using futures/async runtime
    pub fn send_file_minimal(file_path: String, verbose: bool) -> Result<String, String> {
        // Use tokio runtime to run async function synchronously on Android
        let runtime = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;

        runtime.block_on(async {
            // Import necessary types
            use crate::sendme::{
                create_blob, start_send, AddrInfoOptions, CommonArgs, Format, RelayModeOption,
                SendArgs,
            };
            use std::path::PathBuf;

            let send_args = SendArgs {
                path: PathBuf::from(file_path),
                ticket_type: AddrInfoOptions::RelayAndAddresses,
                common: CommonArgs {
                    magic_ipv4_addr: None,
                    magic_ipv6_addr: None,
                    format: Format::Hex,
                    verbose: if verbose { 1 } else { 0 },
                    relay: RelayModeOption::Default,
                },
            };

            let (ticket, _router, _blobs_data_dir) =
                start_send(send_args).await.map_err(|e| e.to_string())?;

            let blob = ticket.to_string();
            match create_blob(blob).await {
                Ok(code) => Ok(code),
                Err(err) => Err(format!("Failed to create blob: {}", err)),
            }
        })
    }

    pub fn receive_file_minimal(
        ticket: String,
        file_storage_path: String,
        verbose: bool,
    ) -> Result<String, String> {
        // Use tokio runtime to run async function synchronously on Android
        let runtime = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;

        runtime.block_on(async {
            use crate::sendme::{
                get_blob, receive_file_minimal as receive_impl, CommonArgs, Format, ReceiveArgs,
                RelayModeOption,
            };
            use iroh_blobs::ticket::BlobTicket;
            use std::str::FromStr;

            let blob = match get_blob(ticket).await {
                Ok(blob) => blob,
                Err(_err) => return Err("Failed to get blob".to_string()),
            };

            let ticket = BlobTicket::from_str(&blob[0]).map_err(|e| e.to_string())?;
            let ticket_string = ticket.to_string();

            let _receive_args = ReceiveArgs {
                ticket: ticket,
                common: CommonArgs {
                    magic_ipv4_addr: None,
                    magic_ipv6_addr: None,
                    format: Format::Hex,
                    verbose: if verbose { 1 } else { 0 },
                    relay: RelayModeOption::Default,
                },
            };

            receive_impl(ticket_string, file_storage_path, verbose)
                .await
                .map_err(|e| e.to_string())
        })
    }

    pub fn stop_sharing() -> Result<(), String> {
        // Simplified implementation for Android
        // The router and cleanup will be handled by the runtime
        Ok(())
    }
}

#[cfg(not(target_os = "android"))]
pub mod desktop {
    // Use the original implementations for desktop platforms
    pub use crate::sendme::{receive_file_minimal, send_file_minimal, stop_sharing};
}
