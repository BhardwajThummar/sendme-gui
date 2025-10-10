/**
 * Version Utility
 *
 * Provides version information for the application.
 * The version is automatically synced from package.json during build.
 */

import packageJson from '../../package.json';

/**
 * Application version information
 */
export const VERSION = {
  /**
   * Full version string (e.g., "1.2.3")
   */
  full: packageJson.version,

  /**
   * Formatted version with 'v' prefix (e.g., "v1.2.3")
   */
  display: `v${packageJson.version}`,

  /**
   * Application name
   */
  name: packageJson.name,

  /**
   * Parsed version components
   */
  parts: (() => {
    const [major, minor, patch] = packageJson.version.split('.').map(Number);
    return { major, minor, patch };
  })(),

  /**
   * Check if this is a development/pre-release version
   */
  isDev: packageJson.version.startsWith('0.'),

  /**
   * Get full version string with build info
   */
  getFullInfo: () => {
    return {
      version: packageJson.version,
      name: packageJson.name,
      isDevelopment: packageJson.version.startsWith('0.'),
      buildDate: new Date().toISOString(),
    };
  },
};

/**
 * Get version string for display in UI
 */
export const getVersionString = (): string => {
  return VERSION.display;
};

/**
 * Get detailed version information
 */
export const getVersionInfo = () => {
  return VERSION.getFullInfo();
};

export default VERSION;
