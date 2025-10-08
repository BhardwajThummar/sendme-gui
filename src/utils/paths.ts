import { invoke } from '@tauri-apps/api/core';
import { Store } from '@tauri-apps/plugin-store';
import { logger } from './logger';
import { STORAGE_CONFIG } from '@/config/app.config';

let storePromise = Store.load(STORAGE_CONFIG.STORE_NAME);

export async function getDownloadsFolderPath(): Promise<string> {
  try {
    const store = await storePromise;

    const savedPath = await store.get<string>(STORAGE_CONFIG.DOWNLOAD_PATH_KEY);
    if (savedPath) {
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
