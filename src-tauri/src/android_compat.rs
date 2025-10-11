// Android compatibility layer
// This file provides alternative implementations for functionality that
// might not be available on Android

#[cfg(target_os = "android")]
pub mod android {
    use std::{path::PathBuf, sync::Mutex};

    use lazy_static::lazy_static;

    use crate::{
        config::config,
        logger::{debug, error, info, warn},
    };

    // Global state to keep router alive on Android
    lazy_static! {
        static ref ANDROID_ROUTER: Mutex<Option<iroh::protocol::Router>> = Mutex::new(None);
        static ref ANDROID_BLOBS_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);
    }

    // Android-specific path helper
    pub fn get_android_app_data_dir() -> Result<PathBuf, String> {
        let cfg = config();
        // On Android, we use the app's external files directory
        // This doesn't require storage permissions on Android 10+
        if let Some(data_dir) = std::env::var_os("ANDROID_DATA") {
            let data_path = PathBuf::from(data_dir);
            Ok(data_path)
        } else {
            // Fallback to configured path
            Ok(PathBuf::from(&cfg.platform.android_app_data_path))
        }
    }

    pub fn get_android_external_files_dir() -> Result<PathBuf, String> {
        let cfg = config();
        // Get the external files directory for the app
        // Path format: /storage/emulated/0/Android/data/<package>/files
        if let Ok(external_storage) = std::env::var("EXTERNAL_STORAGE") {
            let mut path = PathBuf::from(external_storage);
            path.push("Android/data");
            path.push(&cfg.platform.android_package_name);
            path.push("files");
            Ok(path)
        } else {
            // Fallback
            let mut path = PathBuf::from(&cfg.platform.sdcard_fallback_path);
            path.push("Android/data");
            path.push(&cfg.platform.android_package_name);
            path.push("files");
            Ok(path)
        }
    }

    pub fn get_android_downloads_dir() -> Result<PathBuf, String> {
        let cfg = config();
        // Use the system Downloads directory instead of app-specific
        // This is accessible without special permissions on Android 10+
        if let Ok(external_storage) = std::env::var("EXTERNAL_STORAGE") {
            let mut path = PathBuf::from(external_storage);
            path.push("Download"); // Note: Android uses "Download" not "Downloads"
            Ok(path)
        } else {
            // Fallback to configured path
            Ok(PathBuf::from(&cfg.platform.android_download_path))
        }
    }

    // Provide Android implementations using async/await
    pub async fn send_file_minimal(
        file_path: String,
        verbose: bool,
        _window: tauri::Window,
    ) -> Result<String, String> {
        let cfg = config();
        // Import necessary types
        use std::path::PathBuf;

        use crate::sendme::{
            create_blob, start_send, AddrInfoOptions, CommonArgs, Format, RelayModeOption, SendArgs,
        };

        info!("Android: Starting send_file_minimal for: {}", file_path);

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

        debug!("Android: Calling start_send");
        let (ticket, router, _blobs_data_dir) = start_send(send_args, None).await.map_err(|e| {
            let err_msg = format!("start_send failed: {}", e);
            error!("Android: {}", err_msg);
            err_msg
        })?;

        info!("Android: Ticket generated successfully");
        let blob = ticket.to_string();
        debug!("Android: Blob/ticket string length: {}", blob.len());

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
        debug!("Android: Router and data dir stored in global state");

        // Now make the HTTP request to create the blob code with timeout
        info!("Android: Creating blob code via API");
        use std::time::Duration;
        let result = tokio::time::timeout(
            Duration::from_secs(cfg.network.http_timeout_secs),
            create_blob(blob),
        )
        .await;

        match result {
            Ok(Ok(code)) => {
                info!("Android: Blob created successfully with code: {}", code);
                Ok(code)
            }
            Ok(Err(err)) => {
                let err_msg = format!("Failed to create blob: {}", err);
                error!("Android: {}", err_msg);
                Err(err_msg)
            }
            Err(_) => {
                let err_msg = format!(
                    "Timeout: create_blob took longer than {} seconds",
                    cfg.network.http_timeout_secs
                );
                error!("Android: {}", err_msg);
                Err(err_msg)
            }
        }
    }

    pub async fn receive_file_minimal(
        ticket: String,
        file_storage_path: String,
        verbose: bool,
    ) -> Result<String, String> {
        let cfg = config();
        use std::{str::FromStr, time::Duration};

        use iroh_blobs::ticket::BlobTicket;

        use crate::sendme::{get_blob, receive, CommonArgs, Format, ReceiveArgs, RelayModeOption};

        info!("Android: Starting receive_file_minimal");
        debug!("Android: Ticket code: {}", ticket);
        debug!("Android: Storage path: {}", file_storage_path);

        // Get blob from API with timeout
        info!("Android: Fetching blob from API");
        let blob_result = tokio::time::timeout(
            Duration::from_secs(cfg.network.http_timeout_secs),
            get_blob(ticket),
        )
        .await;

        let blob = match blob_result {
            Ok(Ok(blob)) => {
                info!("Android: Successfully fetched blob from API");
                blob
            }
            Ok(Err(err)) => {
                let err_msg = format!("Failed to get blob from API: {}", err);
                error!("Android: {}", err_msg);
                return Err(err_msg);
            }
            Err(_) => {
                let err_msg = format!(
                    "Timeout: get_blob took longer than {} seconds",
                    cfg.network.http_timeout_secs
                );
                error!("Android: {}", err_msg);
                return Err(err_msg);
            }
        };

        if blob.is_empty() {
            let err_msg = "Received empty blob array from API".to_string();
            error!("Android: {}", err_msg);
            return Err(err_msg);
        }

        debug!("Android: Parsing blob ticket");
        let parsed_ticket = BlobTicket::from_str(&blob[0]).map_err(|e| {
            let err_msg = format!("Failed to parse blob ticket: {}", e);
            error!("Android: {}", err_msg);
            err_msg
        })?;
        debug!("Android: Ticket parsed successfully");

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

        info!("Android: Starting file download");
        // Call the actual receive function directly (not the wrapper)
        let result = receive(receive_args, file_storage_path).await;

        match &result {
            Ok(_) => {
                info!("Android: File received successfully");
                Ok("success".to_string())
            }
            Err(e) => {
                let err_msg = format!("Failed to receive file: {}", e);
                error!("Android: {}", err_msg);
                Err(err_msg)
            }
        }
    }

    pub async fn stop_sharing_async() -> Result<(), String> {
        let cfg = config();
        use std::time::Duration;

        info!("Android: Stopping sharing");

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
            debug!("Android: Shutting down router");
            tokio::time::timeout(
                Duration::from_secs(cfg.network.router_shutdown_timeout_secs),
                router.shutdown(),
            )
            .await
            .map_err(|_| "Timeout shutting down router".to_string())?
            .map_err(|e| format!("Failed to shutdown router: {}", e))?;
            info!("Android: Router shutdown complete");
        }

        if let Some(dir) = blobs_dir {
            debug!("Android: Cleaning up blobs directory");
            tokio::fs::remove_dir_all(&dir)
                .await
                .map_err(|e| format!("Failed to cleanup directory: {}", e))?;
            info!("Android: Cleanup complete");
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

    pub async fn send_files_minimal(
        file_paths: Vec<String>,
        verbose: bool,
        _window: tauri::Window,
    ) -> Result<String, String> {
        // For now, Android will just send the first file
        // TODO: Implement proper multi-file support for Android
        if file_paths.is_empty() {
            return Err("No files provided".to_string());
        }

        if file_paths.len() == 1 {
            return send_file_minimal(file_paths[0].clone(), verbose, _window).await;
        }

        // For multiple files, we could implement similar directory bundling logic
        // For now, just send the first file with a warning
        warn!(
            "Android: Multiple files requested ({}), but only first file will be sent",
            file_paths.len()
        );
        send_file_minimal(file_paths[0].clone(), verbose, _window).await
    }
}

#[cfg(not(target_os = "android"))]
pub mod desktop {
    // Desktop platforms use the implementations from sendme module directly
}
