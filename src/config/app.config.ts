/**
 * Application Configuration
 * Centralized configuration for the entire application
 * All hardcoded values should be defined here
 */

import { LogLevel } from '@/types/logger.types';

// Environment detection
const isDevelopment = import.meta.env.DEV;
const isProduction = import.meta.env.PROD;

/**
 * Storage Configuration
 */
export const STORAGE_CONFIG = {
  // Store file name for user preferences
  STORE_NAME: import.meta.env.VITE_STORE_NAME || 'user-preferences.json',

  // Key for storing download path
  DOWNLOAD_PATH_KEY: import.meta.env.VITE_DOWNLOAD_PATH_KEY || 'download_path',
} as const;

/**
 * Platform-specific Configuration
 */
export const PLATFORM_CONFIG = {
  // Default download path for Android
  ANDROID_DOWNLOAD_PATH: import.meta.env.VITE_ANDROID_DOWNLOAD_PATH || '/sdcard/Download',

  // Android user agent pattern
  ANDROID_USER_AGENT_PATTERN: /android/i,
} as const;

/**
 * Logging Configuration
 */
export const LOGGING_CONFIG = {
  // Default log level based on environment
  DEFAULT_LOG_LEVEL: isDevelopment ? LogLevel.DEBUG : LogLevel.INFO,

  // Enable/disable timestamps in logs
  ENABLE_TIMESTAMP: import.meta.env.VITE_ENABLE_LOG_TIMESTAMP !== 'false',

  // Enable/disable log prefixes (level and context)
  ENABLE_PREFIX: import.meta.env.VITE_ENABLE_LOG_PREFIX !== 'false',

  // Override log level from environment
  LOG_LEVEL_OVERRIDE: import.meta.env.VITE_LOG_LEVEL as keyof typeof LogLevel | undefined,
} as const;

/**
 * File Transfer Configuration
 */
export const FILE_TRANSFER_CONFIG = {
  // Code length for send/receive
  CODE_LENGTH: 6,

  // Maximum file size display in bytes
  MAX_DISPLAY_SIZE: 1024 * 1024 * 1024 * 10, // 10GB

  // Progress animation settings
  PROGRESS_INTERVAL_MS: 500,
  PROGRESS_MAX_SIMULATED: 90,
} as const;

/**
 * UI Configuration
 */
export const UI_CONFIG = {
  // Clipboard notification duration
  CLIPBOARD_NOTIFICATION_DURATION: 2000, // 2 seconds

  // Input focus delay
  INPUT_FOCUS_DELAY: 100,

  // Maximum files to display in list
  MAX_FILES_DISPLAY: 1000,
} as const;

/**
 * API Configuration
 */
export const API_CONFIG = {
  // Base URL for API calls (if needed)
  BASE_URL: import.meta.env.VITE_API_BASE_URL || '',

  // Timeout for API calls
  TIMEOUT: parseInt(import.meta.env.VITE_API_TIMEOUT || '30000', 10),

  // Verbose mode for Tauri commands
  VERBOSE_MODE: import.meta.env.VITE_VERBOSE_MODE === 'true',
} as const;

/**
 * Environment Information
 */
export const ENV = {
  isDevelopment,
  isProduction,
  mode: import.meta.env.MODE,
} as const;

/**
 * Get log level from configuration
 */
export function getLogLevel(): LogLevel {
  if (LOGGING_CONFIG.LOG_LEVEL_OVERRIDE) {
    const levelName = LOGGING_CONFIG.LOG_LEVEL_OVERRIDE.toUpperCase();
    if (levelName in LogLevel && typeof LogLevel[levelName as keyof typeof LogLevel] === 'number') {
      return LogLevel[levelName as keyof typeof LogLevel] as LogLevel;
    }
  }
  return LOGGING_CONFIG.DEFAULT_LOG_LEVEL;
}

// Export all configs as a single object for convenience
export const AppConfig = {
  STORAGE: STORAGE_CONFIG,
  PLATFORM: PLATFORM_CONFIG,
  LOGGING: LOGGING_CONFIG,
  FILE_TRANSFER: FILE_TRANSFER_CONFIG,
  UI: UI_CONFIG,
  API: API_CONFIG,
  ENV,
} as const;

export default AppConfig;
