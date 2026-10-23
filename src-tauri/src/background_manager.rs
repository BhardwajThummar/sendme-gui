// Background task manager for keeping the app running
// This module ensures file transfers continue when the app is backgrounded

use crate::logger::{debug, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Enable background execution mode
/// This should be called when starting file transfers
pub fn enable_background_mode() {
    info!("Enabling background execution mode");

    #[cfg(target_os = "android")]
    {
        debug!("Background mode enabled for Android");
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

/// A share started or stopped. On Android the foreground service runs while
/// sharing or receiving, so the transfer keeps full speed in the background.
pub fn sharing(window: &tauri::Window, on: bool) {
    info!("Sharing {}", if on { "started" } else { "stopped" });

    #[cfg(target_os = "android")]
    crate::android_compat::android::set_sharing(window, on);

    #[cfg(target_os = "ios")]
    ios_transfer_active(window, on);

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let _ = window;
}

/// A download started or finished (see `sharing`).
pub fn receiving(window: &tauri::Window, on: bool) {
    info!("Receiving {}", if on { "started" } else { "finished" });

    #[cfg(target_os = "android")]
    if on {
        crate::android_compat::android::receiving_started(window);
    } else {
        crate::android_compat::android::receiving_finished(window);
    }

    #[cfg(target_os = "ios")]
    ios_transfer_active(window, on);

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let _ = window;
}

/// iOS: keep the screen on and hold background time while a transfer runs.
/// (Sharing and receiving can overlap, so only the last one to stop releases it.)
#[cfg(target_os = "ios")]
fn ios_transfer_active(window: &tauri::Window, on: bool) {
    use std::sync::atomic::AtomicUsize;
    use tauri::Manager;
    use tauri_plugin_ios_folder::IosFolderExt;
    static ACTIVE: AtomicUsize = AtomicUsize::new(0);

    let now = if on {
        ACTIVE.fetch_add(1, Ordering::SeqCst) + 1
    } else {
        ACTIVE
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| {
                Some(n.saturating_sub(1))
            })
            .map_or(0, |prev| prev.saturating_sub(1))
    };
    // Only the first start and the last stop change anything.
    if (on && now != 1) || (!on && now != 0) {
        return;
    }
    let app = window.app_handle().clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = app.ios_folder().transfer_active(on).await {
            warn!("iOS transfer keep-awake failed: {}", e);
        }
    });
}

/// Disable background execution mode
/// This should be called when file transfers are complete
pub fn disable_background_mode() {
    info!("Disabling background execution mode");

    #[cfg(target_os = "android")]
    {
        debug!("Background mode disabled for Android");
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

/// Check if background mode is currently enabled
pub fn is_background_mode_enabled() -> bool {
    #[cfg(target_os = "android")]
    {
        crate::android_compat::android::is_background_service_active()
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
