import { logger } from './logger';

interface PickerResponse {
    success: boolean;
    uri?: string;
    uris?: string[];
    error?: string;
}

/**
 * Check if we're running on Android
 */
export function isAndroid(): boolean {
    return typeof navigator !== 'undefined' && /android/i.test(navigator.userAgent);
}

/**
 * Open Android directory picker
 * @returns The selected directory URI or null if cancelled
 */
export async function openAndroidDirectoryPicker(): Promise<string | null> {
    logger.info('AndroidPicker', '=== OPENING DIRECTORY PICKER ===');

    if (!window.DirectoryPickerPlugin) {
        logger.error('AndroidPicker', 'DirectoryPickerPlugin is not available on window object');
        throw new Error('DirectoryPickerPlugin is not available');
    }

    try {
        // Launch the directory picker
        logger.info('AndroidPicker', 'Calling DirectoryPickerPlugin.openDirectoryPicker()');
        const launchResponse = JSON.parse(window.DirectoryPickerPlugin.openDirectoryPicker()) as PickerResponse;

        logger.info('AndroidPicker', 'Launch response:', JSON.stringify(launchResponse));

        if (!launchResponse.success) {
            throw new Error(launchResponse.error || 'Failed to launch directory picker');
        }

        logger.info('AndroidPicker', 'Directory picker launched successfully, waiting for result...');

        // Poll for the result (Android will call back via onActivityResult)
        const uri = await pollForResult(() => {
            const result = JSON.parse(window.DirectoryPickerPlugin!.getLastDirectoryUri()) as PickerResponse;
            return result.success ? result.uri : undefined;
        }, 60000); // 60 second timeout

        if (!uri) {
            logger.warn('AndroidPicker', 'Directory picker cancelled or timed out');
            return null;
        }

        logger.info('AndroidPicker', '=== DIRECTORY SELECTED ===');
        logger.info('AndroidPicker', 'URI:', uri);
        return uri;
    } catch (error) {
        logger.error('AndroidPicker', 'Error opening directory picker', error);
        throw error;
    }
}

/**
 * Open Android file picker
 * @param multiple Allow multiple file selection
 * @returns Array of selected file URIs or null if cancelled
 */
export async function openAndroidFilePicker(multiple: boolean = true): Promise<string[] | null> {
    if (!window.DirectoryPickerPlugin) {
        throw new Error('DirectoryPickerPlugin is not available');
    }

    try {
        // Launch the file picker
        const launchResponse = JSON.parse(window.DirectoryPickerPlugin.openFilePicker(multiple)) as PickerResponse;

        if (!launchResponse.success) {
            throw new Error(launchResponse.error || 'Failed to launch file picker');
        }

        logger.debug('AndroidPicker', 'File picker launched, waiting for result...');

        // Poll for the result
        const uris = await pollForResult(() => {
            const result = JSON.parse(window.DirectoryPickerPlugin!.getLastFileUris()) as PickerResponse;
            return result.success ? result.uris : undefined;
        }, 60000);

        if (!uris || uris.length === 0) {
            logger.info('AndroidPicker', 'File picker cancelled');
            return null;
        }

        logger.info('AndroidPicker', `Files selected: ${uris.length}`);
        return uris;
    } catch (error) {
        logger.error('AndroidPicker', 'Error opening file picker', error);
        throw error;
    }
}

/**
 * Poll for a result with timeout
 */
async function pollForResult<T>(
    checkFn: () => T | undefined,
    timeoutMs: number,
    intervalMs: number = 200
): Promise<T | null> {
    const startTime = Date.now();

    return new Promise((resolve) => {
        const intervalId = setInterval(() => {
            const result = checkFn();

            if (result !== undefined) {
                clearInterval(intervalId);
                resolve(result);
                return;
            }

            if (Date.now() - startTime > timeoutMs) {
                clearInterval(intervalId);
                resolve(null);
            }
        }, intervalMs);
    });
}
