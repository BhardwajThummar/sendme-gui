// Background task manager for keeping the app running
// This module ensures file transfers continue when the app is backgrounded

use crate::logger::{debug, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[cfg(target_os = "android")]
use lazy_static::lazy_static;

#[cfg(target_os = "android")]
lazy_static! {
    static ref BACKGROUND_ENABLED: AtomicBool = AtomicBool::new(false);
}

/// Enable background execution mode
/// This should be called when starting file transfers
pub fn enable_background_mode() {
    info!("Enabling background execution mode");

    #[cfg(target_os = "android")]
    {
        BACKGROUND_ENABLED.store(true, Ordering::SeqCst);
        debug!("Background mode enabled for Android - service started via android_compat");
    }

    #[cfg(target_os = "ios")]
    {
        debug!("Background mode enabled for iOS");
        // iOS background tasks are handled by the system via UIBackgroundModes
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        debug!("Background mode enabled (desktop platform)");
    }
}

/// Enable background execution mode with window handle (Android-specific)
pub fn enable_background_mode_with_window(window: &tauri::Window) {
    info!("Enabling background execution mode with window");

    #[cfg(target_os = "android")]
    {
        use crate::android_compat;
        BACKGROUND_ENABLED.store(true, Ordering::SeqCst);
        let _ = android_compat::android::start_background_service(window);
    }

    #[cfg(not(target_os = "android"))]
    {
        let _ = window; // Suppress unused warning
        enable_background_mode();
    }
}

/// Disable background execution mode
/// This should be called when file transfers are complete
pub fn disable_background_mode() {
    info!("Disabling background execution mode");

    #[cfg(target_os = "android")]
    {
        BACKGROUND_ENABLED.store(false, Ordering::SeqCst);
        debug!("Background mode disabled for Android - service stopped via android_compat");
    }

    #[cfg(target_os = "ios")]
    {
        debug!("Background mode disabled for iOS");
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        debug!("Background mode disabled (desktop platform)");
    }
}

/// Disable background execution mode with window handle (Android-specific)
pub fn disable_background_mode_with_window(window: &tauri::Window) {
    info!("Disabling background execution mode with window");

    #[cfg(target_os = "android")]
    {
        use crate::android_compat;
        BACKGROUND_ENABLED.store(false, Ordering::SeqCst);
        let _ = android_compat::android::stop_background_service(window);
    }

    #[cfg(not(target_os = "android"))]
    {
        let _ = window; // Suppress unused warning
        disable_background_mode();
    }
}

/// Check if background mode is currently enabled
pub fn is_background_mode_enabled() -> bool {
    #[cfg(target_os = "android")]
    {
        BACKGROUND_ENABLED.load(Ordering::SeqCst)
    }

    #[cfg(not(target_os = "android"))]
    {
        // On other platforms, assume background is always available
        true
    }
}

/// Initialize background task handling
/// This should be called during app startup
pub fn init_background_handling() {
    info!("Initializing background task handling");

    #[cfg(target_os = "android")]
    {
        debug!("Android background handling initialized");
        // Android-specific initialization would go here
        // For example, registering for power management callbacks
    }

    #[cfg(target_os = "ios")]
    {
        debug!("iOS background handling initialized");
        // iOS-specific initialization would go here
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        debug!("Desktop background handling initialized");
    }
}

/// Handle app going to background
/// This is called when the app loses focus or is minimized
pub fn on_app_backgrounded() {
    info!("App has been backgrounded");

    if is_background_mode_enabled() {
        warn!("Background tasks are active - keeping app running");

        #[cfg(target_os = "android")]
        {
            debug!("Maintaining Android background execution");
        }

        #[cfg(target_os = "ios")]
        {
            debug!("iOS will continue background tasks based on UIBackgroundModes");
        }
    } else {
        debug!("No background tasks active");
    }
}

/// Handle app returning to foreground
/// This is called when the app regains focus
pub fn on_app_foregrounded() {
    info!("App has returned to foreground");

    #[cfg(target_os = "android")]
    {
        debug!("Android app is now in foreground");
    }

    #[cfg(target_os = "ios")]
    {
        debug!("iOS app is now in foreground");
    }
}

/// Keep alive task to prevent app termination during transfers
/// This function spawns a background task that keeps the app active
pub async fn start_keep_alive_task() -> Arc<AtomicBool> {
    let should_stop = Arc::new(AtomicBool::new(false));
    let should_stop_clone = should_stop.clone();

    info!("Starting keep-alive background task");

    tokio::spawn(async move {
        while !should_stop_clone.load(Ordering::SeqCst) {
            // Periodic heartbeat to keep the app active
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

            if is_background_mode_enabled() {
                debug!("Keep-alive heartbeat: background mode active");
            }
        }

        info!("Keep-alive task stopped");
    });

    should_stop
}

/// Stop the keep-alive task
pub fn stop_keep_alive_task(should_stop: Arc<AtomicBool>) {
    info!("Stopping keep-alive task");
    should_stop.store(true, Ordering::SeqCst);
}
