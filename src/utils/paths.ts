import { invoke } from '@tauri-apps/api/core';
import { Store } from '@tauri-apps/plugin-store';

// Key for storing the download path
const DOWNLOAD_PATH_KEY = 'download_path';
const STORE_NAME = 'user-preferences.json';

// Create a store instance for saving user preferences
// In Tauri v2, we need to use Store.create() instead of new Store()
let storePromise = Store.load(STORE_NAME);

export async function getDownloadsFolderPath(): Promise<string> {
  try {
    // Get the store instance
    const store = await storePromise;

    // First check if we have a saved path in the store
    const savedPath = await store.get<string>(DOWNLOAD_PATH_KEY);
    if (savedPath) {
      return savedPath;
    }

    // If no saved path, get the default downloads directory from the backend
    const downloadsDir = await invoke<string>('get_downloads_dir');

    // Save the downloads directory as the default path for future use
    if (downloadsDir) {
      await saveDownloadPath(downloadsDir);
    }

    return downloadsDir;
  } catch (error) {
    console.error('Error getting downloads folder:', error);
    // Fallback to a default path if all else fails
    return '';
  }
}

export async function saveDownloadPath(path: string): Promise<void> {
  try {
    // Get the store instance
    const store = await storePromise;

    // Save the path to the store
    await store.set(DOWNLOAD_PATH_KEY, path);
    await store.save(); // Persist the changes to disk
  } catch (error) {
    console.error('Error saving download path:', error);
  }
}
