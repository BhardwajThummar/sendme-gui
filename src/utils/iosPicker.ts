import { invoke } from '@tauri-apps/api/core';
import { logger } from './logger';

/**
 * Pick a folder to send on iOS. The folder is copied into the app cache by the
 * `pick_ios_folder` command, since sendme needs real paths.
 * @returns Path of the copied folder, or null if cancelled
 */
export async function pickIosFolderToSend(): Promise<string | null> {
    try {
        return await invoke<string | null>('pick_ios_folder');
    } catch (error) {
        logger.error('IosPicker', 'Error picking folder to send', error);
        throw error;
    }
}

/**
 * Pick where received files are saved on iOS. The returned `path` ("ios:...")
 * holds a bookmark that keeps access to the folder across restarts; it is
 * stored and passed to the receive commands as is.
 * @returns The chosen location, or null if cancelled
 */
export async function pickIosSaveDir(): Promise<{ path: string; name: string } | null> {
    try {
        return await invoke<{ path: string; name: string } | null>('pick_ios_save_dir');
    } catch (error) {
        logger.error('IosPicker', 'Error picking save folder', error);
        throw error;
    }
}
