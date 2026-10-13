import { invoke } from '@tauri-apps/api/core';
import { Store } from '@tauri-apps/plugin-store';
import { logger } from './logger';
import { STORAGE_CONFIG } from '@/config/app.config';

let storePromise = Store.load(STORAGE_CONFIG.STORE_NAME);

// iOS gives the app a new data-container path when it is reinstalled or restored,
// so a saved path inside the old container points nowhere useful. Re-derive it.
const IOS_APP_CONTAINER = /\/Containers\/Data\/Application\/[0-9A-F-]{36}\//i;

export async function getDownloadsFolderPath(): Promise<string> {
  try {
    const store = await storePromise;

    const savedPath = await store.get<string>(STORAGE_CONFIG.DOWNLOAD_PATH_KEY);
    if (savedPath && !IOS_APP_CONTAINER.test(savedPath)) {
      return savedPath;
    }

    const downloadsDir = await invoke<string>('get_downloads_dir');

    if (downloadsDir) {
      await saveDownloadPath(downloadsDir);
    }

    return downloadsDir;
  } catch (error) {
    logger.error('Paths', 'Error getting downloads folder', error);
    return '';
  }
}

export async function saveDownloadPath(path: string): Promise<void> {
  try {
    const store = await storePromise;

    await store.set(STORAGE_CONFIG.DOWNLOAD_PATH_KEY, path);
    await store.save();
  } catch (error) {
    logger.error('Paths', 'Error saving download path', error);
  }
}

/** The saved download path, or undefined if the user never chose one. */
export async function getSavedDownloadPath(): Promise<string | undefined> {
  try {
    const store = await storePromise;
    return await store.get<string>(STORAGE_CONFIG.DOWNLOAD_PATH_KEY);
  } catch (error) {
    logger.error('Paths', 'Error reading saved download path', error);
    return undefined;
  }
}

/**
 * iOS pickers return `file://` URLs (percent-encoded); the backend wants plain
 * paths. Other platforms already return paths, which pass through unchanged.
 */
export function fileUrlToPath(value: string): string {
  if (!value.startsWith('file://')) return value;
  try {
    return decodeURIComponent(new URL(value).pathname);
  } catch {
    return value;
  }
}
