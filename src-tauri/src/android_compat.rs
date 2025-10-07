// Android compatibility layer
// This file provides alternative implementations for functionality that
// might not be available on Android

#[cfg(target_os = "android")]
pub mod android {
    use lazy_static::lazy_static;
    use std::path::PathBuf;
    use std::sync::Mutex;

    // Global state to keep router alive on Android
    lazy_static! {
        static ref ANDROID_ROUTER: Mutex<Option<iroh::protocol::Router>> = Mutex::new(None);
        static ref ANDROID_BLOBS_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);
    }

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
        // Use the system Downloads directory instead of app-specific
        // This is accessible without special permissions on Android 10+
        if let Ok(external_storage) = std::env::var("EXTERNAL_STORAGE") {
            let mut path = PathBuf::from(external_storage);
            path.push("Download"); // Note: Android uses "Download" not "Downloads"
            Ok(path)
        } else {
            // Fallback to standard path
            Ok(PathBuf::from("/sdcard/Download"))
        }
    }

    // Provide Android implementations using async/await
    pub async fn send_file_minimal(file_path: String, verbose: bool) -> Result<String, String> {
        // Import necessary types
        use crate::sendme::{
            create_blob, start_send, AddrInfoOptions, CommonArgs, Format, RelayModeOption, SendArgs,
        };
        use std::path::PathBuf;

        println!("[Android] Starting send_file_minimal for: {}", file_path);

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

        println!("[Android] Calling start_send...");
        let (ticket, router, _blobs_data_dir) =
            start_send(send_args).await.map_err(|e| {
                let err_msg = format!("start_send failed: {}", e);
                println!("[Android] Error: {}", err_msg);
                err_msg
            })?;

        println!("[Android] start_send completed, ticket generated");
        let blob = ticket.to_string();
        println!("[Android] Blob/ticket string length: {}", blob.len());

        // Store router and data dir in global state to keep them alive BEFORE making HTTP request
        {
            let mut router_guard = ANDROID_ROUTER
                .lock()
                .map_err(|e| format!("Failed to lock router: {}", e))?;
            let mut dir_guard = ANDROID_BLOBS_DIR
                .lock()
                .map_err(|e| format!("Failed to lock dir: {}", e))?;
            *router_guard = Some(router);
            *dir_guard = Some(_blobs_data_dir);
        }
        println!("[Android] Router and data dir stored in global state");

        // Check if BASE_URL is set
        match std::env::var("BASE_URL") {
            Ok(url) => println!("[Android] BASE_URL is set to: {}", url),
            Err(_) => println!("[Android] WARNING: BASE_URL not set!"),
        }

        // Now make the HTTP request to create the blob code with timeout
        println!("[Android] Making HTTP request to create_blob...");
        use std::time::Duration;
        let result = tokio::time::timeout(
            Duration::from_secs(30),
            create_blob(blob)
        ).await;

        match result {
            Ok(Ok(code)) => {
                println!("[Android] create_blob succeeded, code: {}", code);
                Ok(code)
            },
            Ok(Err(err)) => {
                let err_msg = format!("Failed to create blob: {}", err);
                println!("[Android] Error: {}", err_msg);
                Err(err_msg)
            },
            Err(_) => {
                let err_msg = "Timeout: create_blob took longer than 30 seconds".to_string();
                println!("[Android] {}", err_msg);
                Err(err_msg)
            },
        }
    }

    pub async fn receive_file_minimal(
        ticket: String,
        file_storage_path: String,
        verbose: bool,
    ) -> Result<String, String> {
        use crate::sendme::{
            get_blob, receive, CommonArgs, Format, ReceiveArgs, RelayModeOption,
        };
        use iroh_blobs::ticket::BlobTicket;
        use std::str::FromStr;
        use std::time::Duration;

        println!("[Android] Starting receive_file_minimal");
        println!("[Android] Ticket code: {}", ticket);
        println!("[Android] Storage path: {}", file_storage_path);

        // Get blob from API with timeout
        println!("[Android] Fetching blob from API...");
        let blob_result = tokio::time::timeout(
            Duration::from_secs(30),
            get_blob(ticket)
        ).await;

        let blob = match blob_result {
            Ok(Ok(blob)) => {
                println!("[Android] Successfully fetched blob from API");
                blob
            },
            Ok(Err(err)) => {
                let err_msg = format!("Failed to get blob from API: {}", err);
                println!("[Android] Error: {}", err_msg);
                return Err(err_msg);
            },
            Err(_) => {
                let err_msg = "Timeout: get_blob took longer than 30 seconds".to_string();
                println!("[Android] {}", err_msg);
                return Err(err_msg);
            }
        };

        if blob.is_empty() {
            let err_msg = "Received empty blob array from API".to_string();
            println!("[Android] Error: {}", err_msg);
            return Err(err_msg);
        }

        println!("[Android] Parsing blob ticket...");
        let parsed_ticket = BlobTicket::from_str(&blob[0]).map_err(|e| {
            let err_msg = format!("Failed to parse blob ticket: {}", e);
            println!("[Android] Error: {}", err_msg);
            err_msg
        })?;
        println!("[Android] Ticket parsed successfully");

        // Construct ReceiveArgs with the parsed ticket
        let receive_args = ReceiveArgs {
            ticket: parsed_ticket,
            common: CommonArgs {
                magic_ipv4_addr: None,
                magic_ipv6_addr: None,
                format: Format::Hex,
                verbose: if verbose { 1 } else { 0 },
                relay: RelayModeOption::Default,
            },
        };

        println!("[Android] Starting file download...");
        // Call the actual receive function directly (not the wrapper)
        let result = receive(receive_args, file_storage_path).await;

        match &result {
            Ok(_) => {
                println!("[Android] File received successfully");
                Ok("success".to_string())
            },
            Err(e) => {
                let err_msg = format!("Failed to receive file: {}", e);
                println!("[Android] Error: {}", err_msg);
                Err(err_msg)
            }
        }
    }

    pub async fn stop_sharing_async() -> Result<(), String> {
        use std::time::Duration;

        let (router, blobs_dir) = {
            let mut router_guard = ANDROID_ROUTER
                .lock()
                .map_err(|e| format!("Failed to lock router: {}", e))?;
            let mut dir_guard = ANDROID_BLOBS_DIR
                .lock()
                .map_err(|e| format!("Failed to lock dir: {}", e))?;
            (router_guard.take(), dir_guard.take())
        };

        if let Some(router) = router {
            tokio::time::timeout(Duration::from_secs(5), router.shutdown())
                .await
                .map_err(|_| "Timeout shutting down router".to_string())?
                .map_err(|e| format!("Failed to shutdown router: {}", e))?;
        }

        if let Some(dir) = blobs_dir {
            tokio::fs::remove_dir_all(dir)
                .await
                .map_err(|e| format!("Failed to cleanup directory: {}", e))?;
        }

        Ok(())
    }

    pub fn stop_sharing() -> Result<(), String> {
        // For synchronous interface, use Runtime::block_on
        use tokio::runtime::Runtime;

        Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?
            .block_on(stop_sharing_async())
    }
}

#[cfg(not(target_os = "android"))]
pub mod desktop {
    // Use the original implementations for desktop platforms
    pub use crate::sendme::{receive_file_minimal, send_file_minimal, stop_sharing};
}
