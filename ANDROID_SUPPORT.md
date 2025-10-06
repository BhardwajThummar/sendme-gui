# Android File Transfer Support

This document outlines the changes made to enable proper Android support for file transfers in the Sendme Tauri application.

## Changes Summary

### 1. Android Permissions (AndroidManifest.xml)

Added the following permissions to support file transfers on Android:

- **Storage Permissions (Android 12 and below):**
  - `READ_EXTERNAL_STORAGE` (maxSdkVersion="32")
  - `WRITE_EXTERNAL_STORAGE` (maxSdkVersion="32")

- **Media Permissions (Android 13+):**
  - `READ_MEDIA_IMAGES`
  - `READ_MEDIA_VIDEO`
  - `READ_MEDIA_AUDIO`

- **Optional Permission for Downloads folder:**
  - `MANAGE_EXTERNAL_STORAGE` (Android 11+)

### 2. File Provider Configuration (file_paths.xml)

Updated the FileProvider paths to include Android-specific storage locations:

- **App-specific external storage** (no permissions needed on Android 10+):
  - `external-files-path`: App's external files directory
  - Downloads subdirectory for received files

- **App-specific internal storage:**
  - `files-path`: Internal files directory

- **Cache directory:**
  - `cache-path`: For temporary files

- **External storage** (requires permissions):
  - `external-path`: General external storage
  - Downloads folder access

### 3. Platform-Specific Path Resolution

#### In `android_compat.rs`:
Created Android-specific helper functions:
- `get_android_app_data_dir()`: Returns app's internal data directory
- `get_android_external_files_dir()`: Returns app-specific external storage
- `get_android_downloads_dir()`: Returns Downloads directory in app storage

#### In `sendme.rs` and `lib.rs`:
Replaced hardcoded desktop paths with platform-specific implementations:
- Desktop: `~/Documents/.sendme-temp`
- Android: `/storage/emulated/0/Android/data/com.sendme_gui_tauri_1.app/files/.sendme-temp`

### 4. Android File Transfer Implementation

Implemented full file transfer functionality for Android in `android_compat.rs`:

#### Send Files (`send_file_minimal`):
- Creates Tokio runtime for async operations
- Uses Iroh networking stack for peer-to-peer transfer
- Generates and stores blob tickets via API
- Returns transfer code for sharing

#### Receive Files (`receive_file_minimal`):
- Creates Tokio runtime for async operations
- Retrieves blob ticket from API using code
- Downloads files using Iroh protocol
- Stores files in app-specific Downloads directory

#### Stop Sharing (`stop_sharing`):
- Handles cleanup of transfer resources
- Compatible with Android runtime

### 5. Downloads Directory Handler

Updated `get_downloads_dir()` in `lib.rs`:
- **Android**: Returns app-specific Downloads directory and creates it if needed
- **Desktop**: Uses system Downloads folder via `dirs` crate

## File Storage Locations on Android

### Received Files:
```
/storage/emulated/0/Android/data/com.sendme_gui_tauri_1.app/files/Downloads/
```

### Temporary Transfer Data:
```
/storage/emulated/0/Android/data/com.sendme_gui_tauri_1.app/files/.sendme-temp/
```

## Benefits of This Implementation

1. **No Permission Prompts for Basic Operations**: Uses scoped storage (app-specific directories) which doesn't require runtime permissions on Android 10+

2. **Backward Compatibility**: Supports older Android versions with appropriate permission declarations

3. **Secure Storage**: Files are stored in app-specific directories, preventing unauthorized access

4. **Automatic Cleanup**: App-specific directories are automatically cleaned up when the app is uninstalled

## Testing Recommendations

1. **Test on Android API 29 (minimum SDK)**: Verify basic file operations
2. **Test on Android API 30-32**: Check scoped storage behavior
3. **Test on Android API 33+**: Verify granular media permissions
4. **Test file permissions**: Ensure FileProvider works correctly for sharing
5. **Test network transfers**: Verify Iroh networking works on mobile networks

## Known Considerations

1. **MANAGE_EXTERNAL_STORAGE Permission**: This is a sensitive permission and may require special approval on Google Play Store. Consider making it optional.

2. **Network Performance**: Mobile networks may have different characteristics than desktop networks. Consider adding network type detection and optimization.

3. **Battery Usage**: Long-running file transfers should handle battery optimization properly.

4. **Background Execution**: Android may kill background processes. Consider implementing foreground services for large transfers.

## Files Modified

1. `/src-tauri/gen/android/app/src/main/AndroidManifest.xml`
2. `/src-tauri/gen/android/app/src/main/res/xml/file_paths.xml`
3. `/src-tauri/src/android_compat.rs`
4. `/src-tauri/src/sendme.rs`
5. `/src-tauri/src/lib.rs`
