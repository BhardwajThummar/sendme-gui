//! Application Configuration Module
//!
//! Centralized configuration for the Tauri application.
//! All hardcoded values should be defined here and can be overridden via environment variables.

use std::sync::OnceLock;

/// Global configuration instance
static CONFIG: OnceLock<AppConfig> = OnceLock::new();

/// Application configuration structure
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct AppConfig {
    /// Platform configuration
    pub platform: PlatformConfig,
    /// API configuration
    pub api: ApiConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Network configuration
    pub network: NetworkConfig,
    /// Environment info
    pub env: EnvConfig,
}

/// Platform-specific configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PlatformConfig {
    /// Android default download path
    pub android_download_path: String,
    /// Android app data path
    pub android_app_data_path: String,
    /// Android package name
    pub android_package_name: String,
    /// Fallback sdcard path
    pub sdcard_fallback_path: String,
}

/// API configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ApiConfig {
    /// Base URL for API calls
    pub base_url: String,
    /// API timeout in seconds
    pub timeout_secs: u64,
    /// Production base URL fallback
    pub production_url: String,
}

/// Storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Temporary directory prefix
    pub temp_dir_prefix: String,
    /// Documents folder name
    pub documents_folder: String,
}

/// Network configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NetworkConfig {
    /// Router shutdown timeout in seconds
    pub router_shutdown_timeout_secs: u64,
    /// HTTP request timeout in seconds
    pub http_timeout_secs: u64,
}

/// Environment configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EnvConfig {
    /// Whether running in debug mode
    pub is_debug: bool,
    /// Whether running in production
    pub is_production: bool,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            android_download_path: env_or_default("ANDROID_DOWNLOAD_PATH", "/sdcard/Download"),
            android_app_data_path: env_or_default(
                "ANDROID_APP_DATA_PATH",
                "/data/data/com.sendme_gui_tauri_1.app/files",
            ),
            android_package_name: env_or_default(
                "ANDROID_PACKAGE_NAME",
                "com.sendme_gui_tauri_1.app",
            ),
            sdcard_fallback_path: env_or_default("SDCARD_FALLBACK_PATH", "/sdcard"),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: std::env::var("BASE_URL")
                .unwrap_or_else(|_| "https://send.hindalert.com".to_string()),
            timeout_secs: env_or_default_parsed("API_TIMEOUT_SECS", 30),
            production_url: "https://send.hindalert.com".to_string(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            temp_dir_prefix: env_or_default("TEMP_DIR_PREFIX", ".sendme-"),
            documents_folder: env_or_default("DOCUMENTS_FOLDER", "Documents"),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            router_shutdown_timeout_secs: env_or_default_parsed("ROUTER_SHUTDOWN_TIMEOUT_SECS", 5),
            http_timeout_secs: env_or_default_parsed("HTTP_TIMEOUT_SECS", 30),
        }
    }
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            is_debug: cfg!(debug_assertions),
            is_production: !cfg!(debug_assertions),
        }
    }
}

/// Get environment variable or return default value
fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Get environment variable and parse it, or return default value
fn env_or_default_parsed<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

impl AppConfig {
    /// Initialize the global configuration
    pub fn init() -> &'static AppConfig {
        CONFIG.get_or_init(|| {
            // Load .env file if present
            dotenv::dotenv().ok();
            AppConfig::default()
        })
    }

    /// Get the global configuration instance
    pub fn get() -> &'static AppConfig {
        CONFIG
            .get()
            .expect("Config not initialized. Call AppConfig::init() first")
    }
}

/// Convenience function to get the global config
pub fn config() -> &'static AppConfig {
    AppConfig::get()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_init() {
        let config = AppConfig::init();
        assert!(!config.platform.android_download_path.is_empty());
        assert!(!config.api.base_url.is_empty());
        assert!(!config.storage.temp_dir_prefix.is_empty());
    }

    #[test]
    fn test_env_fallback() {
        let config = AppConfig::default();
        assert_eq!(config.platform.sdcard_fallback_path, "/sdcard");
    }
}
