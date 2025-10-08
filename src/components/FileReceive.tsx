import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { getDownloadsFolderPath, saveDownloadPath } from '../utils/paths';
import { listen } from '@tauri-apps/api/event';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { Input } from '@/components/ui/input';
import { logger } from '@/utils/logger';
import { PLATFORM_CONFIG, FILE_TRANSFER_CONFIG, UI_CONFIG, API_CONFIG } from '@/config/app.config';
import {
  Download,
  FolderOpen,
  CheckCircle,
  RefreshCw,
  Loader2,
  FileDown,
  X,
} from 'lucide-react';

// Define types for download events
interface DownloadCompletedEvent {
  success: boolean;
  message: string;
  elapsed_time_ms: number;
  download_path: string;
  filename: string;
  total_bytes: number;
  files_count: number;
}

const FileReceive: React.FC = () => {
  const [code, setCode] = useState<string[]>(Array(FILE_TRANSFER_CONFIG.CODE_LENGTH).fill(''));
  const [downloadPath, setDownloadPath] = useState<string>('');
  const [status, setStatus] = useState<
    'idle' | 'processing' | 'success' | 'error'
  >('idle');
  const [statusMessage, setStatusMessage] = useState<string>('');
  const [progress, setProgress] = useState<number>(0);
  const [fileName, setFileName] = useState<string>('');
  const [downloadComplete, setDownloadComplete] = useState<boolean>(false);
  const [downloadStats, setDownloadStats] = useState<{
    totalFiles: number;
    totalSize: string;
    elapsedTime: string;
    speed: string;
  } | null>(null);
  const [isAndroid, setIsAndroid] = useState(false);

  // Create refs for each input field
  const inputRefs = useRef<(HTMLInputElement | null)[]>(Array(FILE_TRANSFER_CONFIG.CODE_LENGTH).fill(null));

  // Initialize download path and set up event listeners
  useEffect(() => {
    const isAndroidPlatform = typeof navigator !== 'undefined' && PLATFORM_CONFIG.ANDROID_USER_AGENT_PATTERN.test(navigator.userAgent);

    setIsAndroid(isAndroidPlatform);

    // On Android, immediately set the default path to system Downloads
    if (isAndroidPlatform) {
      setDownloadPath(PLATFORM_CONFIG.ANDROID_DOWNLOAD_PATH);
      logger.info('FileReceive', `Android platform detected, using path: ${PLATFORM_CONFIG.ANDROID_DOWNLOAD_PATH}`);
    }

    const initPath = async () => {
      if (isAndroidPlatform) {
        return;
      }

      try {
        let path = await getDownloadsFolderPath();

        if (!path) {
          path = await invoke<string>('get_downloads_dir');
        }

        setDownloadPath(path || '');
        logger.info('FileReceive', `Download path initialized: ${path}`);
      } catch (error) {
        logger.error('FileReceive', 'Failed to initialize download path', error);
        setDownloadPath('');
      }
    };

    initPath();

    // Focus first input when component mounts
    setTimeout(() => {
      if (inputRefs.current[0]) {
        inputRefs.current[0]?.focus();
      }
    }, UI_CONFIG.INPUT_FOCUS_DELAY);

    // Set up event listeners for download events
    const setupListeners = async () => {
      const unlisteners: (() => void)[] = [];

      // Listen for download start
      unlisteners.push(
        await listen('download_started', () => {
          logger.info('FileReceive', 'Download started');
          setStatus('processing');
          setStatusMessage('Starting download...');

          // Set up a simple progress animation since we can't get actual progress
          let progressValue = 0;
          const interval = setInterval(() => {
            progressValue += 5;

            if (progressValue > FILE_TRANSFER_CONFIG.PROGRESS_MAX_SIMULATED) {
              clearInterval(interval);
              return;
            }

            setProgress(progressValue);
          }, FILE_TRANSFER_CONFIG.PROGRESS_INTERVAL_MS);

          return () => clearInterval(interval);
        })
      );

      // Listen for status updates
      unlisteners.push(
        await listen<string>('download_status', (event) => {
          logger.debug('FileReceive', `Status update: ${event.payload}`);
          setStatusMessage(event.payload);
        })
      );

      // Listen for download completion
      unlisteners.push(
        await listen<DownloadCompletedEvent>('download_completed', (event) => {
          const { filename, total_bytes, files_count, elapsed_time_ms } =
            event.payload;

          logger.info('FileReceive', `Download completed: ${filename} (${formatBytes(total_bytes)})`);

          setFileName(filename);
          setProgress(100);
          setStatus('success');
          setStatusMessage('Download complete!');
          setDownloadComplete(true);

          setDownloadStats({
            totalFiles: files_count,
            totalSize: formatBytes(total_bytes),
            elapsedTime: formatTime(elapsed_time_ms),
            speed:
              formatBytes(Math.floor(total_bytes / (elapsed_time_ms / 1000))) +
              '/s',
          });
        })
      );

      // Listen for download errors
      unlisteners.push(
        await listen<string>('download_error', (event) => {
          logger.error('FileReceive', 'Download failed', event.payload);
          setStatus('error');
          setStatusMessage(`Error: ${event.payload}`);
          setProgress(0);
        })
      );

      // Return cleanup function
      return () => {
        unlisteners.forEach((unlisten) => unlisten());
      };
    };

    // Set up listeners and store cleanup function
    const cleanupPromise = setupListeners();

    // Return cleanup function
    return () => {
      cleanupPromise.then((cleanup) => cleanup());
    };
  }, []);

  // Handle input change for code fields
  const handleCodeChange = (index: number, value: string) => {
    // Only allow alphanumeric characters
    if (!/^[a-zA-Z0-9]*$/.test(value)) {
      return;
    }

    // Update the code array
    const newCode = [...code];
    newCode[index] = value.toUpperCase();
    setCode(newCode);

    // Auto-focus next input if current one is filled
    if (value && index < FILE_TRANSFER_CONFIG.CODE_LENGTH - 1) {
      if (inputRefs.current[index + 1]) {
        inputRefs.current[index + 1]?.focus();
      }
    }
  };

  // Handle key down for backspace and arrow navigation
  const handleKeyDown = (index: number, e: React.KeyboardEvent) => {
    if (e.key === 'Backspace' && !code[index] && index > 0) {
      if (inputRefs.current[index - 1]) {
        inputRefs.current[index - 1]?.focus();
      }
    } else if (e.key === 'ArrowLeft' && index > 0) {
      if (inputRefs.current[index - 1]) {
        inputRefs.current[index - 1]?.focus();
      }
    } else if (e.key === 'ArrowRight' && index < FILE_TRANSFER_CONFIG.CODE_LENGTH - 1) {
      if (inputRefs.current[index + 1]) {
        inputRefs.current[index + 1]?.focus();
      }
    }
  };

  // Handle paste for the entire code
  const handlePaste = (e: React.ClipboardEvent) => {
    e.preventDefault();
    const pastedData = e.clipboardData.getData('text').trim();

    // If pasted data matches the expected format
    const pattern = new RegExp(`^[a-zA-Z0-9]{${FILE_TRANSFER_CONFIG.CODE_LENGTH}}$`);
    if (pattern.test(pastedData)) {
      const chars = pastedData.split('').map((char) => char.toUpperCase());
      setCode(chars);

      // Focus the last input after paste
      const lastIndex = FILE_TRANSFER_CONFIG.CODE_LENGTH - 1;
      if (inputRefs.current[lastIndex]) {
        inputRefs.current[lastIndex]?.focus();
      }
    }
  };

  const handleBrowse = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Download Location',
      });

      if (selected && !Array.isArray(selected)) {
        setDownloadPath(selected);
        await saveDownloadPath(selected);
        logger.info('FileReceive', `Download path updated to: ${selected}`);
      }
    } catch (error) {
      logger.error('FileReceive', 'Error selecting directory', error);
    }
  };

  const handleReceive = async () => {
    const fullCode = code.join('');

    if (fullCode.length !== FILE_TRANSFER_CONFIG.CODE_LENGTH) {
      setStatus('error');
      setStatusMessage(`Please enter a complete ${FILE_TRANSFER_CONFIG.CODE_LENGTH}-character code`);
      return;
    }

    if (!downloadPath) {
      setStatus('error');
      setStatusMessage('Please select a download location');
      return;
    }

    setStatus('processing');
    setStatusMessage('Connecting to sender...');
    setProgress(0);

    try {
      await saveDownloadPath(downloadPath);

      logger.info('FileReceive', `Initiating download with code: ${fullCode}`);

      await invoke<string>('receive_file_command', {
        ticket: fullCode,
        fileStoragePath: downloadPath,
        verbose: API_CONFIG.VERBOSE_MODE,
      });
    } catch (error) {
      logger.error('FileReceive', 'Error receiving file', error);
      setStatus('error');
      setStatusMessage(`Error: ${error}`);
      setProgress(0);
    }
  };

  // Reset the form
  const handleReset = () => {
    setCode(Array(FILE_TRANSFER_CONFIG.CODE_LENGTH).fill(''));
    setStatus('idle');
    setStatusMessage('');
    setProgress(0);
    setDownloadComplete(false);
    setDownloadStats(null);

    // Focus the first input again
    setTimeout(() => {
      if (inputRefs.current[0]) {
        inputRefs.current[0]?.focus();
      }
    }, UI_CONFIG.INPUT_FOCUS_DELAY);
  };

  // Helper function to format bytes
  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';

    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));

    return parseFloat((bytes / Math.pow(1024, i)).toFixed(2)) + ' ' + sizes[i];
  };

  // Helper function to format time
  const formatTime = (ms: number): string => {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);

    if (hours > 0) {
      return `${hours}h ${minutes % 60}m ${seconds % 60}s`;
    } else if (minutes > 0) {
      return `${minutes}m ${seconds % 60}s`;
    } else {
      return `${seconds}s`;
    }
  };

  return (
    <div className="w-full">
      {downloadComplete ? (
        <div className="space-y-6">
          <div className="flex flex-col items-center justify-center">
            <div className="w-16 h-16 rounded-full bg-muted flex items-center justify-center mb-4">
              <CheckCircle className="h-8 w-8 text-primary" />
            </div>
            <h2 className="text-xl font-bold text-center">
              Download Complete!
            </h2>

            {downloadStats && (
              <div className="bg-muted p-4 rounded-lg text-xs space-y-3 border border-border w-full mt-4">
                <div className="flex items-center gap-2 pb-2 border-b border-border">
                  <FileDown className="h-4 w-4 text-foreground shrink-0" />
                  <div className="min-w-0">
                    <div className="font-medium text-sm truncate">
                      {fileName}
                    </div>
                    {downloadStats.totalFiles > 1 && (
                      <div className="text-xs text-muted-foreground">
                        {downloadStats.totalFiles} files
                      </div>
                    )}
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-3">
                  <div>
                    <div className="text-muted-foreground mb-1">Total Size</div>
                    <div className="font-medium">{downloadStats.totalSize}</div>
                  </div>
                  <div>
                    <div className="text-muted-foreground mb-1">
                      Download Time
                    </div>
                    <div className="font-medium">
                      {downloadStats.elapsedTime}
                    </div>
                  </div>
                  <div>
                    <div className="text-muted-foreground mb-1">
                      Average Speed
                    </div>
                    <div className="font-medium">{downloadStats.speed}</div>
                  </div>
                </div>
              </div>
            )}

            <div className="space-y-1 w-full px-2 mt-2">
              <p className="text-xs text-muted-foreground">
                Your file has been saved to:
              </p>
              <div className="bg-muted p-2 rounded-md text-xs font-mono break-all border border-border">
                {downloadPath}
              </div>
            </div>
          </div>

          <div className="px-2">
            <Button
              onClick={handleReset}
              className="flex items-center gap-2 w-full"
            >
              <RefreshCw className="h-4 w-4" />
              Receive Another File
            </Button>
          </div>
        </div>
      ) : (
        <div className="space-y-4">
          <div className="space-y-3">
            <h3 className="text-base font-medium text-center">
              Enter 6-character Code
            </h3>
            <div className="flex justify-center gap-2">
              {code.map((char, index) => (
                <Input
                  key={index}
                  ref={(el) => (inputRefs.current[index] = el)}
                  type="text"
                  maxLength={1}
                  value={char}
                  onChange={(e) => handleCodeChange(index, e.target.value)}
                  onKeyDown={(e) => handleKeyDown(index, e)}
                  onPaste={index === 0 ? handlePaste : undefined}
                  className="w-12 h-14 text-center text-xl font-mono border-border focus:border-primary focus:ring-primary/30 p-0"
                  disabled={status === 'processing'}
                />
              ))}
            </div>
            <p className="text-xs text-center text-muted-foreground">
              Code is case-insensitive and can be pasted
            </p>
          </div>

          <div className="space-y-2">
            <label className="text-xs font-medium block">Save files to</label>
            <div className="flex items-center gap-2">
              <Input
                type="text"
                value={downloadPath}
                onChange={(e) => setDownloadPath(e.target.value)}
                placeholder="Download location"
                disabled={status === 'processing'}
                className="flex-1 border-border text-xs h-9"
              />
              <Button
                variant="outline"
                onClick={handleBrowse}
                disabled={status === 'processing'}
                className="flex items-center justify-center border-border hover:bg-muted hover:text-primary h-9 w-9 p-0"
                size="icon"
                title="Browse for folder"
              >
                <FolderOpen className="h-4 w-4" />
              </Button>
            </div>
            {downloadPath && (
              <p className="text-xs text-muted-foreground">
                Files will be saved to: {downloadPath}
              </p>
            )}
            {!downloadPath && (
              <p className="text-xs text-yellow-600">
                Loading download path...
              </p>
            )}
          </div>

          <div className="pt-2">
            <Button
              className="w-full flex items-center gap-2"
              onClick={handleReceive}
              disabled={
                code.join('').length !== FILE_TRANSFER_CONFIG.CODE_LENGTH ||
                !downloadPath ||
                status === 'processing'
              }
              variant="default"
            >
              {status === 'processing' ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  <span>Receiving...</span>
                </>
              ) : (
                <>
                  <Download className="h-4 w-4" />
                  <span>Receive Files</span>
                </>
              )}
            </Button>

            {(status === 'error' || status === 'success') && (
              <Button
                variant="outline"
                onClick={handleReset}
                className="flex items-center gap-2 w-full mt-2"
                size="sm"
              >
                <RefreshCw className="h-4 w-4" />
                Reset
              </Button>
            )}
          </div>

          {status === 'processing' && (
            <div className="space-y-2 mt-4 border border-border rounded-lg p-3 bg-muted">
              <div className="flex justify-between items-center text-xs">
                <span className="font-medium truncate max-w-[70%]">
                  {fileName || 'Downloading file...'}
                </span>
                <span className="font-medium">{Math.round(progress)}%</span>
              </div>
              <Progress value={progress} className="h-2" />
              <p className="text-xs text-center text-muted-foreground">
                {statusMessage || 'Please wait...'}
              </p>
            </div>
          )}

          {status === 'error' && statusMessage && (
            <div className="p-3 rounded-md bg-muted border border-destructive mt-4">
              <div className="flex items-center gap-2 text-destructive">
                <X className="h-4 w-4 shrink-0" />
                <p className="text-xs">{statusMessage}</p>
              </div>
            </div>
          )}

          {status !== 'error' && status !== 'processing' && statusMessage && (
            <div className="p-3 rounded-md bg-muted border border-primary mt-4">
              <div className="flex items-center gap-2 text-primary">
                <p className="text-xs">{statusMessage}</p>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default FileReceive;
