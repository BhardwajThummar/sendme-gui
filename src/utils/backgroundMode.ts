import { invoke } from '@tauri-apps/api/core';
import React from 'react';

/**
 * Background mode utility functions for keeping the app running during file transfers
 */

/**
 * Check if background mode is currently enabled
 * @returns {Promise<boolean>} True if background mode is enabled
 */
export async function isBackgroundModeEnabled(): Promise<boolean> {
  try {
    return await invoke<boolean>('is_background_mode_enabled');
  } catch (error) {
    console.error('Failed to check background mode status:', error);
    return false;
  }
}

/**
 * Enable background mode to keep the app running
 * This should be called before starting file transfers
 */
export async function enableBackgroundMode(): Promise<void> {
  try {
    await invoke('enable_background_mode');
    console.log('Background mode enabled');
  } catch (error) {
    console.error('Failed to enable background mode:', error);
  }
}

/**
 * Disable background mode
 * This should be called after file transfers are complete
 */
export async function disableBackgroundMode(): Promise<void> {
  try {
    await invoke('disable_background_mode');
    console.log('Background mode disabled');
  } catch (error) {
    console.error('Failed to disable background mode:', error);
  }
}

/**
 * React hook for managing background mode
 * @returns Object with background mode status and control functions
 */
export function useBackgroundMode() {
  const [isEnabled, setIsEnabled] = React.useState(false);

  React.useEffect(() => {
    // Check initial status
    isBackgroundModeEnabled().then(setIsEnabled);
  }, []);

  const enable = React.useCallback(async () => {
    await enableBackgroundMode();
    setIsEnabled(true);
  }, []);

  const disable = React.useCallback(async () => {
    await disableBackgroundMode();
    setIsEnabled(false);
  }, []);

  return { isEnabled, enable, disable };
}
