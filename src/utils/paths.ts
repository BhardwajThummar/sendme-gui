import { invoke } from '@tauri-apps/api/tauri';

export async function getDownloadsFolderPath(): Promise<string> {
    try {
        // Using Tauri's invoke to get the downloads directory from the backend
        return await invoke<string>('get_downloads_dir');
    } catch (error) {
        console.error('Error getting downloads directory:', error);
        // Fallback to a default path if the backend call fails
        return '';
    }
} 