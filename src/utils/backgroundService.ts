import { logger } from './logger';

/**
 * Interface for the Android Transfer Service Plugin
 */
interface TransferServicePlugin {
  startService(transferType: string, fileCount: number): string;
  stopService(): string;
  isServiceRunning(): boolean;
}

/**
 * Global window interface extension for Android plugins
 */
declare global {
  interface Window {
    TransferServicePlugin?: TransferServicePlugin;
  }
}

/**
 * Check if the app is running on Android
 */
export const isAndroid = (): boolean => {
  return typeof navigator !== 'undefined' && /android/i.test(navigator.userAgent);
};

/**
 * Check if the background service is available (Android only)
 */
export const isServiceAvailable = (): boolean => {
  return isAndroid() && typeof window.TransferServicePlugin !== 'undefined';
};

/**
 * Start the background file transfer service (Android only)
 * @param transferType Either "send" or "receive"
 * @param fileCount Number of files being transferred
 * @returns true if service started successfully
 */
export const startBackgroundService = (
  transferType: 'send' | 'receive',
  fileCount: number = 1
): boolean => {
  if (!isServiceAvailable()) {
    logger.warn('BackgroundService', 'Service not available on this platform');
    return false;
  }

  try {
    const result = window.TransferServicePlugin!.startService(transferType, fileCount);
    const response = JSON.parse(result);

    if (response.success) {
      logger.info('BackgroundService', `Service started: ${transferType}, ${fileCount} files`);
      return true;
    } else {
      logger.error('BackgroundService', `Failed to start service: ${response.error}`);
      return false;
    }
  } catch (error) {
    logger.error('BackgroundService', 'Error starting service', error);
    return false;
  }
};

/**
 * Stop the background file transfer service (Android only)
 * @returns true if service stopped successfully
 */
export const stopBackgroundService = (): boolean => {
  if (!isServiceAvailable()) {
    logger.warn('BackgroundService', 'Service not available on this platform');
    return false;
  }

  try {
    const result = window.TransferServicePlugin!.stopService();
    const response = JSON.parse(result);

    if (response.success) {
      logger.info('BackgroundService', 'Service stopped successfully');
      return true;
    } else {
      logger.error('BackgroundService', `Failed to stop service: ${response.error}`);
      return false;
    }
  } catch (error) {
    logger.error('BackgroundService', 'Error stopping service', error);
    return false;
  }
};

/**
 * Check if the background service is currently running (Android only)
 * @returns true if service is running
 */
export const isServiceRunning = (): boolean => {
  if (!isServiceAvailable()) {
    return false;
  }

  try {
    return window.TransferServicePlugin!.isServiceRunning();
  } catch (error) {
    logger.error('BackgroundService', 'Error checking service status', error);
    return false;
  }
};

/**
 * Request notification permission (Android 13+)
 * This should be called before starting the service on Android 13+
 */
export const requestNotificationPermission = async (): Promise<boolean> => {
  if (!isAndroid()) {
    return true; // Not needed on other platforms
  }

  // On Android 13+, we need to request POST_NOTIFICATIONS permission
  // This would typically be done through Tauri's permission API
  // For now, we'll assume permissions are granted
  logger.info('BackgroundService', 'Notification permission check (would request on Android 13+)');
  return true;
};
