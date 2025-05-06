// Android compatibility layer
// This file provides alternative implementations for functionality that
// might not be available on Android

#[cfg(target_os = "android")]
pub mod android {
    // Provide alternative implementations for Android
    pub fn send_file_minimal(_file_path: String, _verbose: bool) -> Result<String, String> {
        // Simplified implementation for Android
        Err("File sending not implemented for Android yet".to_string())
    }

    pub fn receive_file_minimal(_ticket: String, _file_storage_path: String, _verbose: bool) -> Result<String, String> {
        // Simplified implementation for Android
        Err("File receiving not implemented for Android yet".to_string())
    }

    pub fn stop_sharing() -> Result<(), String> {
        // Simplified implementation for Android
        Ok(())
    }
}

#[cfg(not(target_os = "android"))]
pub mod desktop {
    // Use the original implementations for desktop platforms
    pub use crate::sendme::{send_file_minimal, receive_file_minimal, stop_sharing};
}
