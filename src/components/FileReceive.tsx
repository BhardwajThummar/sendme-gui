// src/components/FileReceive.tsx
import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { getDownloadsFolderPath, saveDownloadPath } from '../utils/paths';
import { listen } from '@tauri-apps/api/event';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { Input } from '@/components/ui/input';
import { useToast } from '@/components/ui/toast';
import { Download, FolderOpen, CheckCircle, RefreshCw, Loader2, FileDown, X } from 'lucide-react';

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
  const [code, setCode] = useState<string[]>(Array(6).fill(''));
  const [downloadPath, setDownloadPath] = useState<string>('');
  const [status, setStatus] = useState<'idle' | 'processing' | 'success' | 'error'>('idle');
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

  // Create refs for each input field
  const inputRefs = useRef<(HTMLInputElement | null)[]>(Array(6).fill(null));

  // Initialize download path and set up event listeners
  useEffect(() => {
    const initPath = async () => {
      try {
        const path = await getDownloadsFolderPath();
        setDownloadPath(path);
      } catch (error) {
        console.error('Error getting downloads folder:', error);
        setDownloadPath('');
      }
    };

    initPath();

    // Focus first input when component mounts
    setTimeout(() => {
      if (inputRefs.current[0]) {
        inputRefs.current[0]?.focus();
      }
    }, 100);

    // Set up event listeners for download events
    const setupListeners = async () => {
      const unlisteners: (() => void)[] = [];

      // Listen for download start
      unlisteners.push(
        await listen('download_started', () => {
          console.log('Download started');
          setStatus('processing');
          setStatusMessage('Starting download...');

          // Set up a simple progress animation since we can't get actual progress
          let progressValue = 0;
          const interval = setInterval(() => {
            progressValue += 5;

            // Cap at 90% (save the last 10% for actual completion)
            if (progressValue > 90) {
              clearInterval(interval);
              return;
            }

            setProgress(progressValue);
          }, 500);

          // Clear interval on component unmount
          return () => clearInterval(interval);
        })
      );

      // Listen for status updates
      unlisteners.push(
        await listen<string>('download_status', (event) => {
          console.log('Download status:', event.payload);
          setStatusMessage(event.payload);
        })
      );

      // Listen for download completion
      unlisteners.push(
        await listen<DownloadCompletedEvent>('download_completed', (event) => {
          console.log('Download completed:', event.payload);
          const { filename, total_bytes, files_count, elapsed_time_ms } = event.payload;

          // Update state with completion information
          setFileName(filename);
          setProgress(100);
          setStatus('success');
          setStatusMessage('Download complete!');
          setDownloadComplete(true);

          // Format stats for display
          setDownloadStats({
            totalFiles: files_count,
            totalSize: formatBytes(total_bytes),
            elapsedTime: formatTime(elapsed_time_ms),
            speed: formatBytes(Math.floor(total_bytes / (elapsed_time_ms / 1000))) + '/s'
          });
        })
      );

      // Listen for download errors
      unlisteners.push(
        await listen<string>('download_error', (event) => {
          console.log('Download error:', event.payload);
          setStatus('error');
          setStatusMessage(`Error: ${event.payload}`);
          setProgress(0);
        })
      );

      // Return cleanup function
      return () => {
        unlisteners.forEach(unlisten => unlisten());
      };
    };

    // Set up listeners and store cleanup function
    const cleanupPromise = setupListeners();

    // Return cleanup function
    return () => {
      cleanupPromise.then(cleanup => cleanup());
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
    if (value && index < 5) {
      if (inputRefs.current[index + 1]) {
        inputRefs.current[index + 1]?.focus();
      }
    }
  };

  // Handle key down for backspace and arrow navigation
  const handleKeyDown = (index: number, e: React.KeyboardEvent) => {
    if (e.key === 'Backspace' && !code[index] && index > 0) {
      // If current field is empty and backspace is pressed, move to previous field
      if (inputRefs.current[index - 1]) {
        inputRefs.current[index - 1]?.focus();
      }
    } else if (e.key === 'ArrowLeft' && index > 0) {
      // Move to previous field on left arrow
      if (inputRefs.current[index - 1]) {
        inputRefs.current[index - 1]?.focus();
      }
    } else if (e.key === 'ArrowRight' && index < 5) {
      // Move to next field on right arrow
      if (inputRefs.current[index + 1]) {
        inputRefs.current[index + 1]?.focus();
      }
    }
  };

  // Handle paste for the entire code
  const handlePaste = (e: React.ClipboardEvent) => {
    e.preventDefault();
    const pastedData = e.clipboardData.getData('text').trim();

    // If pasted data matches the expected format (6 chars)
    if (/^[a-zA-Z0-9]{6}$/.test(pastedData)) {
      const chars = pastedData.split('').map(char => char.toUpperCase());
      setCode(chars);

      // Focus the last input after paste
      if (inputRefs.current[5]) {
        inputRefs.current[5]?.focus();
      }
    }
  };

  // Handle the browse button for download path
  const handleBrowse = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false, // Changed to false since we only need one directory
        title: 'Select Download Location',
      });

      if (selected && !Array.isArray(selected)) {
        // Update the UI with the selected path
        setDownloadPath(selected);

        // Save the selected path for future use
        await saveDownloadPath(selected);
      }
    } catch (error) {
      console.error('Error selecting directory:', error);
    }
  };

  // Handle the receive button
  const handleReceive = async () => {
    const fullCode = code.join('');

    if (fullCode.length !== 6) {
      setStatus('error');
      setStatusMessage('Please enter a complete 6-character code');
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
      // Save the current download path for future use
      await saveDownloadPath(downloadPath);

      // Invoke the Tauri command to receive file
      const result = await invoke<string>('receive_file_command', {
        ticket: fullCode,
        fileStoragePath: downloadPath,
        verbose: false
      });

      // The results and progress will be handled by the event listeners
      console.log('Command result:', result);
    } catch (error) {
      console.error('Error receiving file:', error);
      setStatus('error');
      setStatusMessage(`Error: ${error}`);
      setProgress(0);
    }
  };

  // Reset the form
  const handleReset = () => {
    setCode(Array(6).fill(''));
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
    }, 100);
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

  const { addToast } = useToast();

  return (
    <div className="w-full max-w-md mx-auto">
      {downloadComplete ? (
        <Card className="border-secondary/20">
          <CardHeader className="text-center pb-2 space-y-2 pt-6">
            <div className="mx-auto w-14 h-14 sm:w-16 sm:h-16 rounded-full bg-green-100 flex items-center justify-center">
              <CheckCircle className="h-7 w-7 sm:h-8 sm:w-8 text-green-600" />
            </div>
            <CardTitle className="text-xl sm:text-2xl">Download Complete!</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4 px-4">
            {downloadStats && (
              <div className="bg-muted/50 p-4 rounded-lg text-sm space-y-3 border border-secondary/20">
                <div className="flex items-center gap-2 pb-2 border-b">
                  <FileDown className="h-4 w-4 text-secondary shrink-0" />
                  <div className="min-w-0">
                    <div className="font-medium text-sm truncate">{fileName}</div>
                    {downloadStats.totalFiles > 1 && (
                      <div className="text-xs text-muted-foreground">
                        {downloadStats.totalFiles} files
                      </div>
                    )}
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-3 text-xs">
                  <div>
                    <div className="text-muted-foreground mb-1">Total Size</div>
                    <div className="font-medium">{downloadStats.totalSize}</div>
                  </div>
                  <div>
                    <div className="text-muted-foreground mb-1">Download Time</div>
                    <div className="font-medium">{downloadStats.elapsedTime}</div>
                  </div>
                  <div>
                    <div className="text-muted-foreground mb-1">Average Speed</div>
                    <div className="font-medium">{downloadStats.speed}</div>
                  </div>
                </div>
              </div>
            )}

            <div className="space-y-1">
              <p className="text-xs text-muted-foreground">Your file has been saved to:</p>
              <div className="bg-muted/50 p-2 rounded-md text-xs font-mono break-all border border-secondary/20">
                {downloadPath}
              </div>
            </div>
          </CardContent>
          <CardFooter className="justify-center pb-6">
            <Button
              onClick={handleReset}
              className="flex items-center gap-2 w-full"
            >
              <RefreshCw className="h-4 w-4" />
              Receive Another File
            </Button>
          </CardFooter>
        </Card>
      ) : (
        <div className="space-y-4">
          <Card className="border-secondary/20">
            <CardHeader className="pb-2 pt-5">
              <CardTitle className="text-lg sm:text-xl text-center">Enter Receive Code</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4 px-4">
              <div className="space-y-2">
                <label className="text-xs sm:text-sm font-medium block text-center">
                  Enter the 6-character code from sender
                </label>
                <div className="flex justify-center gap-1 sm:gap-2">
                  {code.map((char, index) => (
                    <Input
                      key={index}
                      ref={(el) => inputRefs.current[index] = el}
                      type="text"
                      maxLength={1}
                      value={char}
                      onChange={(e) => handleCodeChange(index, e.target.value)}
                      onKeyDown={(e) => handleKeyDown(index, e)}
                      onPaste={index === 0 ? handlePaste : undefined}
                      className="w-10 h-12 sm:w-12 sm:h-14 text-center text-lg sm:text-xl font-mono border-secondary/20 focus:border-secondary focus:ring-secondary/30 p-0"
                      disabled={status === 'processing'}
                    />
                  ))}
                </div>
                <p className="text-xs text-center text-muted-foreground">
                  Code is case-insensitive and can be pasted
                </p>
              </div>

              <div className="space-y-2">
                <label className="text-xs sm:text-sm font-medium block">Save files to</label>
                <div className="flex gap-2">
                  <Input
                    type="text"
                    value={downloadPath}
                    onChange={(e) => setDownloadPath(e.target.value)}
                    placeholder="Download location"
                    disabled={status === 'processing'}
                    className="flex-1 border-secondary/20 text-xs sm:text-sm"
                  />
                  <Button
                    variant="outline"
                    onClick={handleBrowse}
                    disabled={status === 'processing'}
                    className="flex items-center border-secondary/20 hover:bg-secondary/5 hover:text-secondary px-2 sm:px-3"
                    size="sm"
                  >
                    <FolderOpen className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            </CardContent>
            <CardFooter className="flex flex-col gap-3 pb-5">
              <Button
                className="w-full flex items-center gap-2"
                onClick={handleReceive}
                disabled={code.join('').length !== 6 || !downloadPath || status === 'processing'}
                variant="secondary"
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
                  className="flex items-center gap-2 w-full"
                  size="sm"
                >
                  <RefreshCw className="h-4 w-4" />
                  Reset
                </Button>
              )}
            </CardFooter>
          </Card>

          {status === 'processing' && (
            <Card className="border-secondary/20">
              <CardContent className="py-4 px-4">
                <div className="space-y-3">
                  <div className="flex justify-between items-center text-xs sm:text-sm">
                    <span className="font-medium truncate max-w-[70%]">{fileName || 'Downloading file...'}</span>
                    <span className="font-medium">{Math.round(progress)}%</span>
                  </div>
                  <Progress value={progress} className="h-2" />
                  <p className="text-xs text-center text-muted-foreground">{statusMessage || 'Please wait...'}</p>
                </div>
              </CardContent>
            </Card>
          )}

          {status === 'error' && statusMessage && (
            <Card className="border-destructive/20 bg-destructive/5">
              <CardContent className="py-3 px-4">
                <div className="flex items-center gap-2 text-destructive">
                  <X className="h-4 w-4 shrink-0" />
                  <p className="text-xs sm:text-sm">{statusMessage}</p>
                </div>
              </CardContent>
            </Card>
          )}

          {status !== 'error' && status !== 'processing' && statusMessage && (
            <Card className="border-secondary/20 bg-secondary/5">
              <CardContent className="py-3 px-4">
                <div className="flex items-center gap-2 text-secondary">
                  <p className="text-xs sm:text-sm">{statusMessage}</p>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      )}
    </div>
  );
};

export default FileReceive;