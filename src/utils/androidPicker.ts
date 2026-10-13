import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { logger } from './logger';

/**
 * Check if we're running on Android
 */
export function isAndroid(): boolean {
    return typeof navigator !== 'undefined' && /android/i.test(navigator.userAgent);
}

/**
 * Open the Android file picker via the Tauri dialog plugin.
 * Selected files come back as `content://` URIs; they are copied to real
 * paths by the `resolve_content_uri` command (see fileSelection.ts).
 * @param multiple Allow multiple file selection
 * @returns Array of selected file URIs or null if cancelled
 */
export async function openAndroidFilePicker(multiple: boolean = true): Promise<string[] | null> {
    try {
        const selected = await open({ multiple, title: 'Select Files to Send' });

        if (!selected) {
            logger.info('AndroidPicker', 'File picker cancelled');
            return null;
        }

        const uris = Array.isArray(selected) ? selected : [selected];
        logger.info('AndroidPicker', `Files selected: ${uris.length}`);
        return uris;
    } catch (error) {
        logger.error('AndroidPicker', 'Error opening file picker', error);
        throw error;
    }
}

/**
 * Open the Android folder picker. The chosen folder is copied into the app
 * cache by the `pick_android_folder` command, since sendme needs real paths.
 * @returns Path of the copied folder, or null if cancelled
 */
export async function openAndroidFolderPicker(): Promise<string | null> {
    try {
        const path = await invoke<string | null>('pick_android_folder');
        if (!path) {
            logger.info('AndroidPicker', 'Folder picker cancelled');
        }
        return path;
    } catch (error) {
        logger.error('AndroidPicker', 'Error opening folder picker', error);
        throw error;
    }
}

/**
 * Open the Android folder picker to choose where received files are saved.
 * The returned `path` ("saf:..." ) is stored and passed to the receive
 * commands as is; `name` is the folder's display name.
 * @returns The chosen location, or null if cancelled
 */
export async function pickAndroidSaveDir(): Promise<{ path: string; name: string } | null> {
    try {
        return await invoke<{ path: string; name: string } | null>('pick_android_save_dir');
    } catch (error) {
        logger.error('AndroidPicker', 'Error picking save folder', error);
        throw error;
    }
}
