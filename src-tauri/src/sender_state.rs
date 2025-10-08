// src-tauri/src/sender_state.rs
use std::{path::PathBuf, sync::Mutex};

// Adjust the type of the router as needed (this example uses the same type as your send function).
// You may need to import iroh::protocol::Router from your code.
pub struct SenderState {
    pub router: Option<iroh::protocol::Router>,
    pub blobs_data_dir: Option<PathBuf>,
}

impl Default for SenderState {
    fn default() -> Self {
        Self {
            router: None,
            blobs_data_dir: None,
        }
    }
}

// For easier sharing in Tauri, we’ll wrap it in a Mutex.
pub type SharedSenderState = Mutex<SenderState>;
