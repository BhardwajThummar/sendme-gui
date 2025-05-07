// src/components/FileSend.tsx
import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import {
  Upload,
  File,
  X,
  Copy,
  FolderOpen,
  Plus,
  CheckCircle,
  Loader2,
} from 'lucide-react';

interface FileInfo {
  name: string;
  size: string;
  path: string;
}

const FileSend: React.FC = () => {
  const [selectedFiles, setSelectedFiles] = useState<FileInfo[]>([]);
  const [status, setStatus] = useState<
    'idle' | 'processing' | 'success' | 'error'
  >('idle');
  const [statusMessage, setStatusMessage] = useState<string>('');
  const [sendCode, setSendCode] = useState<string>('');

  // Format file size for display
  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return bytes + ' B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  // Handle file selection using Tauri's dialog API
  const handleFileSelect = async () => {
    try {
      console.log('Opening file dialog...');
      const selected = await open({
        multiple: true,
        // directory: true,
        title: 'Select Files to Send',
      });

      console.log('Files selected:', selected);

      if (selected && Array.isArray(selected) && selected.length > 0) {
        setStatus('processing');
        setStatusMessage('Getting file information...');

        // Get actual file sizes using backend command
        const filePromises = selected.map(async (path) => {
          // Extract filename from path
          const pathParts = path.split(/[/\\]/);
          const fileName = pathParts[pathParts.length - 1];

          try {
            // Get actual file size from Tauri backend
            const size = await invoke<number>('get_file_size', { path });
            return {
              name: fileName,
              path: path,
              size: formatFileSize(size),
            };
          } catch (error) {
            console.error(`Error getting size for file ${path}:`, error);
            return {
              name: fileName,
              path: path,
              size: 'Size unavailable',
            };
          }
        });

        try {
          // Wait for all file size promises to resolve
          const newFiles = await Promise.all(filePromises);
          console.log('Processed files with actual sizes:', newFiles);

          // Filter out duplicates based on file path
          setSelectedFiles((prev) => {
            const existingPaths = new Set(prev.map((file) => file.path));
            const uniqueNewFiles = newFiles.filter(
              (file) => !existingPaths.has(file.path)
            );
            return [...prev, ...uniqueNewFiles];
          });

          setStatus('idle');
          setStatusMessage('');
        } catch (error) {
          console.error('Error processing files:', error);
          setStatus('error');
          setStatusMessage('Error getting file information');
        }
      }
    } catch (error) {
      console.error('Error selecting files:', error);
      setStatus('error');
      setStatusMessage(`Error selecting files: ${error}`);
    }
  };
  
  // Handle directory selection using Tauri's dialog API
  const handleDirSelect = async () => {
    try {
      console.log('Opening file dialog...');
      const selected = await open({
        multiple: true,
        directory: true,
        title: 'Select Directories to Send',
      });

      console.log('Directories selected:', selected);

      if (selected && Array.isArray(selected) && selected.length > 0) {
        setStatus('processing');
        setStatusMessage('Getting file information...');

        // Get actual file sizes using backend command
        const filePromises = selected.map(async (path) => {
          // Extract filename from path
          const pathParts = path.split(/[/\\]/);
          const fileName = pathParts[pathParts.length - 1];

          try {
            // Get actual file size from Tauri backend
            const size = await invoke<number>('get_file_size', { path });
            return {
              name: fileName,
              path: path,
              size: formatFileSize(size),
            };
          } catch (error) {
            console.error(`Error getting size for file ${path}:`, error);
            return {
              name: fileName,
              path: path,
              size: 'Size unavailable',
            };
          }
        });

        try {
          // Wait for all file size promises to resolve
          const newFiles = await Promise.all(filePromises);
          console.log('Processed files with actual sizes:', newFiles);

          // Filter out duplicates based on file path
          setSelectedFiles((prev) => {
            const existingPaths = new Set(prev.map((file) => file.path));
            const uniqueNewFiles = newFiles.filter(
              (file) => !existingPaths.has(file.path)
            );
            return [...prev, ...uniqueNewFiles];
          });

          setStatus('idle');
          setStatusMessage('');
        } catch (error) {
          console.error('Error processing files:', error);
          setStatus('error');
          setStatusMessage('Error getting file information');
        }
      }
    } catch (error) {
      console.error('Error selecting files:', error);
      setStatus('error');
      setStatusMessage(`Error selecting files: ${error}`);
    }
  };

  // Remove a file from the selection
  const removeFile = (index: number) => {
    setSelectedFiles((files) => files.filter((_, i) => i !== index));
    if (selectedFiles.length <= 1) {
      setSendCode('');
    }
  };

  // Handle the send button click
  const handleSend = async () => {
    if (selectedFiles.length === 0) {
      setStatus('error');
      setStatusMessage('Please select at least one file');
      return;
    }

    setStatus('processing');
    setStatusMessage('Preparing your files...');

    try {
      // Use the first selected file path since the backend expects a single filePath
      const filePath = selectedFiles[0].path;

      // Call the Tauri command with the correct parameter name
      const result = await invoke<string>('send_file_command', {
        filePath: filePath,
        verbose: false,
      });

      // Set the send code received from backend
      setSendCode(result);
      setStatus('success');
      setStatusMessage('Your files are ready to send!');
    } catch (error) {
      console.error('Error sending files:', error);
      setStatus('error');
      setStatusMessage(`Error: ${error}`);
    }
  };

  // Copy send code to clipboard
  const copyCodeToClipboard = () => {
    navigator.clipboard.writeText(sendCode);
    setStatusMessage('Code copied to clipboard!');

    // Reset the message after 2 seconds
    setTimeout(() => {
      if (status === 'success') {
        setStatusMessage('Your files are ready to send!');
      }
    }, 2000);
  };

  // Reset the send process
  const handleReset = () => {
    setSelectedFiles([]);
    setStatus('idle');
    setStatusMessage('');
    setSendCode('');
  };

  return (
    <div className="w-full">
      {sendCode ? (
        <div className="space-y-6">
          <div className="flex flex-col items-center justify-center">
            <div className="w-16 h-16 rounded-full bg-muted flex items-center justify-center mb-4">
              <CheckCircle className="h-8 w-8 text-primary" />
            </div>
            <h2 className="text-xl font-bold text-center">Your Send Code</h2>

            <div className="flex flex-wrap justify-center items-center gap-2 text-2xl font-mono bg-muted p-4 rounded-lg my-4 w-full border border-border">
              {sendCode.split('').map((char, index) => (
                <span
                  key={index}
                  className="inline-flex items-center justify-center w-10 h-12 rounded-md bg-background border border-border shadow-sm"
                >
                  {char}
                </span>
              ))}
            </div>

            <p className="text-muted-foreground text-sm text-center px-4">
              Send this code to the recipient to let them download your files
            </p>
          </div>

          <div className="flex flex-col gap-3 px-2">
            <Button
              onClick={copyCodeToClipboard}
              className="flex items-center justify-center gap-2 w-full"
            >
              <Copy className="h-4 w-4" />
              Copy Code
            </Button>

            <Button variant="outline" onClick={handleReset} className="w-full">
              Send Different Files
            </Button>
          </div>
        </div>
      ) : (
        <div className="space-y-4">
          <div className="grid grid-cols-2 gap-3">
            <Button
              variant="outline"
              className="h-auto py-4 flex flex-col items-center gap-2 border-border hover:border-primary hover:bg-muted"
              onClick={handleFileSelect}
              disabled={status === 'processing'}
            >
              {status === 'processing' ? (
                <Loader2 className="h-5 w-5 animate-spin text-primary" />
              ) : (
                <Upload className="h-6 w-6 text-primary" />
              )}
              <div className="text-center">
                <div className="font-medium text-xs">Select Files</div>
              </div>
            </Button>

            <Button
              variant="outline"
              className="h-auto py-4 flex flex-col items-center gap-2 border-border hover:border-primary hover:bg-muted"
              onClick={handleDirSelect}
              disabled={status === 'processing'}
            >
              {status === 'processing' ? (
                <Loader2 className="h-5 w-5 animate-spin text-primary" />
              ) : (
                <FolderOpen className="h-6 w-6 text-primary" />
              )}
              <div className="text-center">
                <div className="font-medium text-xs">Select Folders</div>
              </div>
            </Button>
          </div>

          {selectedFiles.length > 0 && (
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <h3 className="text-sm font-medium">
                  Selected Files ({selectedFiles.length})
                </h3>
              </div>

              <div className="border border-border rounded-lg divide-y divide-border max-h-[200px] overflow-y-auto">
                {selectedFiles.map((file, index) => (
                  <div
                    key={index}
                    className="flex items-center justify-between p-2 hover:bg-muted"
                  >
                    <div className="flex items-center gap-2 flex-1 min-w-0">
                      <File className="h-4 w-4 text-foreground shrink-0" />
                      <div className="min-w-0">
                        <div className="font-medium text-xs truncate">
                          {file.name}
                        </div>
                        <div className="text-xs text-muted-foreground">
                          {file.size}
                        </div>
                      </div>
                    </div>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-6 w-6 text-muted-foreground hover:text-destructive shrink-0"
                      onClick={(e) => {
                        e.stopPropagation();
                        removeFile(index);
                      }}
                    >
                      <X className="h-3 w-3" />
                    </Button>
                  </div>
                ))}
              </div>

              <div className="flex flex-col gap-2">
                <Button
                  variant="outline"
                  className="flex items-center gap-2 w-full"
                  onClick={handleFileSelect}
                  disabled={status === 'processing'}
                  size="sm"
                >
                  <Plus className="h-3 w-3" />
                  {status === 'processing' ? 'Processing...' : 'Add More Files'}
                </Button>

                <Button
                  className="flex items-center gap-2 w-full"
                  onClick={handleSend}
                  disabled={
                    status === 'processing' || selectedFiles.length === 0
                  }
                >
                  {status === 'processing' ? (
                    <>
                      <Loader2 className="h-4 w-4 animate-spin" />
                      <span>Preparing...</span>
                    </>
                  ) : (
                    'Generate Send Code'
                  )}
                </Button>
              </div>
            </div>
          )}

          {status === 'processing' && (
            <div className="space-y-2 mt-4 p-3 border border-border rounded-md bg-muted">
              <div className="flex justify-between items-center text-xs">
                <span className="text-muted-foreground">
                  Preparing files...
                </span>
                <span className="font-medium">30%</span>
              </div>
              <Progress value={30} className="h-2" />
              <p className="text-xs text-center text-muted-foreground">
                {statusMessage}
              </p>
            </div>
          )}

          {status === 'error' && statusMessage && (
            <div className="p-3 rounded-md bg-muted border border-destructive">
              <div className="flex items-center gap-2 text-destructive">
                <X className="h-4 w-4 shrink-0" />
                <p className="text-xs">{statusMessage}</p>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default FileSend;
