// iOS compatibility layer
// This file provides iOS-specific implementations for functionality that
// differs from desktop platforms

#[cfg(target_os = "ios")]
pub mod ios {
    use std::path::PathBuf;

    use crate::logger::{debug, info};

    /// Get the iOS app's Documents directory
    /// This is the recommended location for user-visible files on iOS
    pub fn get_ios_documents_dir() -> Result<PathBuf, String> {
        // On iOS, we use the app's Documents directory
        // This is accessible via the Files app if UIFileSharingEnabled is true
        if let Some(home) = dirs::home_dir() {
            let documents = home.join("Documents");
            info!("iOS Documents directory: {}", documents.display());
            Ok(documents)
        } else {
            Err("Could not determine iOS home directory".to_string())
        }
    }

    /// Get the iOS app's temporary directory
    /// Used for intermediate files during transfer
    pub fn get_ios_temp_dir() -> Result<PathBuf, String> {
        // Use the standard temp directory on iOS
        let temp = std::env::temp_dir();
        let sendme_temp = temp.join("sendme-temp");
        debug!("iOS temp directory: {}", sendme_temp.display());
        Ok(sendme_temp)
    }

    /// Get the iOS app's cache directory
    /// Used for blob storage during transfers
    pub fn get_ios_cache_dir() -> Result<PathBuf, String> {
        if let Some(cache) = dirs::cache_dir() {
            let sendme_cache = cache.join("sendme");
            debug!("iOS cache directory: {}", sendme_cache.display());
            Ok(sendme_cache)
        } else {
            // Fallback to temp directory
            get_ios_temp_dir()
        }
    }

    /// Get the iOS Downloads directory equivalent
    /// On iOS, we use the Documents directory since there's no standard Downloads
    pub fn get_ios_downloads_dir() -> Result<PathBuf, String> {
        let docs = get_ios_documents_dir()?;
        let downloads = docs.join("Downloads");

        // Create the directory if it doesn't exist
        if !downloads.exists() {
            std::fs::create_dir_all(&downloads)
                .map_err(|e| format!("Failed to create Downloads directory: {}", e))?;
        }

        info!("iOS Downloads directory: {}", downloads.display());
        Ok(downloads)
    }

    /// iOS doesn't support background services like Android
    /// But we can request background task time from the system
    pub fn request_background_time() {
        info!("iOS: Requesting background task time");
        // Background task handling is done at the Swift/UIKit level
        // The Rust code will continue running during the granted time
    }

    /// Signal that background work is complete
    pub fn end_background_time() {
        info!("iOS: Background task complete");
    }
}

#[cfg(not(target_os = "ios"))]
pub mod ios {
    // Stub module for non-iOS platforms
    use std::path::PathBuf;

    pub fn get_ios_documents_dir() -> Result<PathBuf, String> {
        Err("Not running on iOS".to_string())
    }

    pub fn get_ios_temp_dir() -> Result<PathBuf, String> {
        Err("Not running on iOS".to_string())
    }

    pub fn get_ios_cache_dir() -> Result<PathBuf, String> {
        Err("Not running on iOS".to_string())
    }

    pub fn get_ios_downloads_dir() -> Result<PathBuf, String> {
        Err("Not running on iOS".to_string())
    }

    pub fn request_background_time() {}
    pub fn end_background_time() {}
}
