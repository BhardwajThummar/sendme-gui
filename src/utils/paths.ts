import { invoke } from '@tauri-apps/api/core';
import { Store } from '@tauri-apps/plugin-store';

// Create a store instance for saving user preferences
const store = new Store('user-preferences.json');

// Key for storing the download path
const DOWNLOAD_PATH_KEY = 'download_path';

export async function getDownloadsFolderPath(): Promise<string> {
    try {
        // First check if we have a saved path in the store
        const savedPath = await store.get<string>(DOWNLOAD_PATH_KEY);
        if (savedPath) {
            return savedPath;
        }

        // If no saved path, get the default downloads directory from the backend
        return await invoke<string>('get_downloads_dir');
    } catch (error) {
        console.error('Error getting downloads folder:', error);
        // Fallback to a default path if all else fails
        return '';
    }
}

export async function saveDownloadPath(path: string): Promise<void> {
    try {
        // Save the path to the store
        await store.set(DOWNLOAD_PATH_KEY, path);
        await store.save(); // Persist the changes to disk
    } catch (error) {
        console.error('Error saving download path:', error);
    }
}