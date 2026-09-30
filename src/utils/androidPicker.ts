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
