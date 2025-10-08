/**
 * Centralized logging utility following industry standards
 * Provides structured logging with configurable log levels
 */

import { LogLevel } from '@/types/logger.types';
import { getLogLevel, LOGGING_CONFIG } from '@/config/app.config';

export { LogLevel };

interface LoggerConfig {
  level: LogLevel;
  enableTimestamp: boolean;
  enablePrefix: boolean;
}

class Logger {
  private config: LoggerConfig = {
    level: getLogLevel(),
    enableTimestamp: LOGGING_CONFIG.ENABLE_TIMESTAMP,
    enablePrefix: LOGGING_CONFIG.ENABLE_PREFIX,
  };

  private formatMessage(level: string, context: string, message: string): string {
    const parts: string[] = [];

    if (this.config.enableTimestamp) {
      parts.push(new Date().toISOString());
    }

    if (this.config.enablePrefix) {
      parts.push(`[${level}]`);
      if (context) {
        parts.push(`[${context}]`);
      }
    }

    parts.push(message);

    return parts.join(' ');
  }

  /**
   * Log error messages - always shown
   */
  error(context: string, message: string, error?: unknown): void {
    if (this.config.level >= LogLevel.ERROR) {
      console.error(this.formatMessage('ERROR', context, message), error || '');
    }
  }

  /**
   * Log warning messages
   */
  warn(context: string, message: string, data?: unknown): void {
    if (this.config.level >= LogLevel.WARN) {
      console.warn(this.formatMessage('WARN', context, message), data || '');
    }
  }

  /**
   * Log informational messages
   */
  info(context: string, message: string, data?: unknown): void {
    if (this.config.level >= LogLevel.INFO) {
      console.info(this.formatMessage('INFO', context, message), data || '');
    }
  }

  /**
   * Log debug messages - only in development
   */
  debug(context: string, message: string, data?: unknown): void {
    if (this.config.level >= LogLevel.DEBUG) {
      console.debug(this.formatMessage('DEBUG', context, message), data || '');
    }
  }

  /**
   * Update logger configuration
   */
  configure(config: Partial<LoggerConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Get current log level
   */
  getLevel(): LogLevel {
    return this.config.level;
  }
}

// Export singleton instance
export const logger = new Logger();
