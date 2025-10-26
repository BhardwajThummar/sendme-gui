/**
 * Formats file paths for better display in the UI
 */

/**
 * Formats a path to be more readable
 * Extracts meaningful parts and hides unnecessary Android internal paths
 */
export function formatPathForDisplay(path: string): string {
    if (!path) return '';

    try {
        // Simply show the last 2-3 parts of the path for readability
        const parts = path.split('/').filter(p => p);

        // If path is very short, just return it
        if (parts.length <= 2) {
            return path;
        }

        // Show last 2 parts for better readability
        const displayParts = parts.slice(-2);
        return displayParts.join('/');
    } catch (error) {
        // If anything fails, just return the original path
        console.error('Error formatting path:', error);
        return path;
    }
}

/**
 * Format file size for display
 */
export function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    if (!bytes || bytes < 0) return 'Unknown size';

    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));

    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}
