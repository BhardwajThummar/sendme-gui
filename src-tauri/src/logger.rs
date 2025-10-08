//! Logging Module
//!
//! Provides structured logging using the `tracing` framework.
//! Replaces all println!, eprintln! with proper structured logging.

use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Initialize the logger with appropriate log levels
pub fn init() {
    // Get log level from environment or use defaults
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            // Debug mode: show DEBUG and above
            EnvFilter::new("debug")
        } else {
            // Release mode: show INFO and above
            EnvFilter::new("info")
        }
    });

    // Configure the tracing subscriber
    let fmt_layer = fmt::layer()
        .with_target(true) // Include the target (module path)
        .with_thread_ids(false) // Don't show thread IDs for cleaner logs
        .with_thread_names(false)
        .with_file(cfg!(debug_assertions)) // Show file/line only in debug
        .with_line_number(cfg!(debug_assertions))
        .with_span_events(FmtSpan::CLOSE); // Log when spans close

    // Build and initialize the subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()
        .ok(); // Ignore error if already initialized
}

/// Re-export commonly used macros and types for convenience
pub use tracing::{debug, error, info, warn};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_init() {
        // Logger should initialize without panicking
        init();
    }
}
