# SendMe GUI - AI Coding Instructions

## Architecture Overview

**Tauri v2 + React** peer-to-peer file transfer app using the [Iroh protocol](https://iroh.computer/) for direct device-to-device transfers without cloud intermediaries.

### Key Layers

- **Frontend**: React + TypeScript + Tailwind CSS in [src/](src/)
- **Backend**: Rust in [src-tauri/src/](src-tauri/src/) - handles Iroh networking, file I/O, and platform-specific behavior
- **Bridge**: Tauri commands (`#[tauri::command]`) connect frontend ↔ backend via `invoke()` calls

### Core Modules (Rust)

| Module                                                       | Purpose                                                                                                   |
| ------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------- |
| [lib.rs](src-tauri/src/lib.rs)                               | Tauri commands: `send_file_command`, `send_files_command`, `receive_file_command`, `stop_sharing_command` |
| [sendme.rs](src-tauri/src/sendme.rs)                         | Iroh networking: `send_file_minimal`, `receive_file_with_progress`, blob tickets                          |
| [android_compat.rs](src-tauri/src/android_compat.rs)         | Android-specific implementations (guarded by `#[cfg(target_os = "android")]`)                             |
| [ios_compat.rs](src-tauri/src/ios_compat.rs)                 | iOS-specific implementations (guarded by `#[cfg(target_os = "ios")]`)                                     |
| [background_manager.rs](src-tauri/src/background_manager.rs) | Keeps transfers alive when app is backgrounded                                                            |
| [config.rs](src-tauri/src/config.rs)                         | Centralized configuration - **all hardcoded values go here**                                              |

### Frontend Structure

- [src/components/FileSend.tsx](src/components/FileSend.tsx) / [FileReceive.tsx](src/components/FileReceive.tsx) - Main UI components
- [src/context/TransferContext.tsx](src/context/TransferContext.tsx) - Global transfer state via React Context
- [src/config/app.config.ts](src/config/app.config.ts) - Frontend configuration constants
- [src/utils/logger.ts](src/utils/logger.ts) - Structured logging with levels

## Development Commands

```bash
yarn install                 # Install dependencies
yarn tauri dev               # Run desktop app (hot reload)
yarn android:dev             # Run Android app
yarn tauri ios dev           # Run iOS app (requires Xcode + signing)
yarn build                   # Build for production
yarn version:patch|minor|major  # Bump version (updates package.json, Cargo.toml, tauri.conf.json)
```

## Critical Patterns

### Platform Conditionals (Rust)

All platform-specific code uses `#[cfg(target_os = "...")]` guards. Use three-way splits for mobile:

```rust
#[cfg(target_os = "android")]
{ /* Android implementation */ }

#[cfg(target_os = "ios")]
{ /* iOS implementation */ }

#[cfg(not(any(target_os = "android", target_os = "ios")))]
{ /* Desktop implementation */ }
```

### Frontend ↔ Backend Communication

- **Commands**: Frontend calls `invoke("command_name", { args })` → Rust `#[tauri::command]` functions
- **Events**: Rust emits events via `window.emit("event_name", payload)` → Frontend listens with `listen("event_name", callback)`
- Key events: `send_transfer_progress`, `download_completed`, `import_progress`, `start-background-service`

### Configuration

- **Rust**: Use `config::config()` to access `AppConfig` (never hardcode paths/values)
- **TypeScript**: Import from `@/config/app.config.ts` (uses `VITE_*` env vars)

### Logging

- **Rust**: Use `logger::{info!, debug!, warn!, error!}` macros (not `println!`)
- **TypeScript**: Use `logger.info(context, message)` from `@/utils/logger`

### Background Transfers

Always wrap transfer operations with background mode:

```rust
background_manager::enable_background_mode_with_window(&window);
// ... transfer logic ...
background_manager::disable_background_mode_with_window(&window);
```

## Version Management

Versions are synchronized across 3 files via [scripts/version.sh](scripts/version.sh):

- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

Never edit versions manually—use `yarn version:*` commands.

## Platform-Specific Notes

### iOS

- **Entitlements**: Network permissions in `src-tauri/gen/apple/sendme-gui_iOS/sendme-gui_iOS.entitlements`
- **Info.plist**: Background modes, local network usage in `src-tauri/gen/apple/sendme-gui_iOS/Info.plist`
- **Paths**: Use `ios_compat::ios::get_ios_documents_dir()` for user files, `get_ios_downloads_dir()` for downloads
- **File sharing**: Enabled via `UIFileSharingEnabled` - files in Documents are visible in iOS Files app
- **Build**: Requires Apple Developer account for device builds; simulator works without signing

### Android

- **Paths**: Use `android_compat::android::get_android_external_files_dir()` for app storage
- **Background**: Foreground service started via events to frontend (`start-background-service`)
- **Permissions**: External storage handled via scoped storage (Android 10+)

## Common Gotchas

1. **Path imports**: Use `@/` alias for absolute imports (e.g., `@/components/ui/button`)
2. **Iroh blobs**: File transfers use `BlobTicket` for addressing—see `iroh_blobs` crate
3. **Shared state**: `SenderState` is wrapped in `Mutex<>` for thread safety via Tauri's `.manage()`
4. **Temp directories**: Cleaned up on app close via `cleanup_sendme_dirs()` in lib.rs
5. **iOS builds**: Use `CC=clang` when building Rust for iOS targets if you have custom CC set
6. **Disk space**: The `src-tauri/target` folder can grow to 10GB+; use `cargo clean` to free space
