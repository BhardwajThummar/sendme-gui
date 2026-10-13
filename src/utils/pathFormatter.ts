/**
 * Formats file paths for better display in the UI
 */

/**
 * Formats a path to be more readable
 * Extracts meaningful parts and hides unnecessary Android internal paths
 */
export function formatPathForDisplay(path: string): string {
    if (!path) return '';

    // A folder picked on Android is stored as "saf:" + JSON with a content URI
    if (path.startsWith('saf:')) {
        return safLocationLabel(path);
    }

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

/**
 * Readable name for an Android picked-folder location, e.g.
 * content://com.android.externalstorage.documents/tree/primary%3ADocuments%2Fin
 * becomes "Documents/in".
 */
export function safLocationLabel(path: string): string {
    try {
        const uri: string = JSON.parse(path.slice('saf:'.length)).uri ?? '';
        // The tree id is the segment after /tree/; "/document/..." that may follow is not part of it
        const tree = decodeURIComponent((uri.split('/tree/')[1] ?? uri).split('/')[0]);
        // "primary:Documents/in" or "1A2B-3C4D:Pictures" -> drop the volume prefix
        return tree.includes(':') ? tree.slice(tree.indexOf(':') + 1) || 'Internal storage' : tree;
    } catch {
        return 'Selected folder';
    }
}
