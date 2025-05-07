// src/components/FileShare.tsx
import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { useToast } from '@/components/ui/toast';
import { Upload, File, X, Copy, FolderOpen, Plus, CheckCircle, Loader2 } from 'lucide-react';

interface FileInfo {
  name: string;
  size: string;
  path: string;
}

const FileShare: React.FC = () => {
  const [selectedFiles, setSelectedFiles] = useState<FileInfo[]>([]);
  const [status, setStatus] = useState<'idle' | 'processing' | 'success' | 'error'>('idle');
  const [statusMessage, setStatusMessage] = useState<string>('');
  const [shareCode, setShareCode] = useState<string>('');

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
      console.log("Opening file dialog...");
      const selected = await open({
        multiple: true,
        // directory: true,
        title: 'Select Files to Share'
      });

      console.log("Files selected:", selected);

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
              size: formatFileSize(size)
            };
          } catch (error) {
            console.error(`Error getting size for file ${path}:`, error);
            return {
              name: fileName,
              path: path,
              size: 'Size unavailable'
            };
          }
        });

        try {
          // Wait for all file size promises to resolve
          const newFiles = await Promise.all(filePromises);
          console.log("Processed files with actual sizes:", newFiles);
          setSelectedFiles(prev => [...prev, ...newFiles]);
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
      console.log("Opening file dialog...");
      const selected = await open({
        multiple: true,
        directory: true,
        title: 'Select Directories to Share'
      });

      console.log("Directories selected:", selected);

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
              size: formatFileSize(size)
            };
          } catch (error) {
            console.error(`Error getting size for file ${path}:`, error);
            return {
              name: fileName,
              path: path,
              size: 'Size unavailable'
            };
          }
        });

        try {
          // Wait for all file size promises to resolve
          const newFiles = await Promise.all(filePromises);
          console.log("Processed files with actual sizes:", newFiles);
          setSelectedFiles(prev => [...prev, ...newFiles]);
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
    setSelectedFiles(files => files.filter((_, i) => i !== index));
    if (selectedFiles.length <= 1) {
      setShareCode('');
    }
  };

  // Handle the share button click
  const handleShare = async () => {
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
        verbose: false
      });

      // Set the share code received from backend
      setShareCode(result);
      setStatus('success');
      setStatusMessage('Your files are ready to share!');
    } catch (error) {
      console.error('Error sharing files:', error);
      setStatus('error');
      setStatusMessage(`Error: ${error}`);
    }
  };

  // Copy share code to clipboard
  const copyCodeToClipboard = () => {
    navigator.clipboard.writeText(shareCode);
    setStatusMessage('Code copied to clipboard!');

    // Reset the message after 2 seconds
    setTimeout(() => {
      if (status === 'success') {
        setStatusMessage('Your files are ready to share!');
      }
    }, 2000);
  };

  // Reset the share process
  const handleReset = () => {
    setSelectedFiles([]);
    setStatus('idle');
    setStatusMessage('');
    setShareCode('');
  };

  const { addToast } = useToast();

  return (
    <div className="w-full max-w-md mx-auto">
      {shareCode ? (
        <Card className="border-primary/20">
          <CardHeader className="text-center pb-2 space-y-2 pt-6">
            <div className="mx-auto w-14 h-14 sm:w-16 sm:h-16 rounded-full bg-primary/10 flex items-center justify-center">
              <CheckCircle className="h-7 w-7 sm:h-8 sm:w-8 text-primary" />
            </div>
            <CardTitle className="text-xl sm:text-2xl">Your Share Code</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4 text-center px-4">
            <div className="flex flex-wrap justify-center items-center gap-2 text-2xl sm:text-3xl font-mono bg-muted/50 p-4 rounded-lg my-2 border border-primary/20">
              {shareCode.split('').map((char, index) => (
                <span
                  key={index}
                  className="inline-flex items-center justify-center w-10 h-12 sm:w-12 sm:h-14 rounded-md bg-background border border-primary/20 shadow-sm"
                >
                  {char}
                </span>
              ))}
            </div>

            <p className="text-muted-foreground text-sm">
              Share this code with the recipient to let them download your files
            </p>
          </CardContent>
          <CardFooter className="flex flex-col gap-3 justify-center pb-6">
            <Button
              onClick={copyCodeToClipboard}
              className="flex items-center justify-center gap-2 w-full"
            >
              <Copy className="h-4 w-4" />
              Copy Code
            </Button>

            <Button
              variant="outline"
              onClick={handleReset}
              className="w-full"
            >
              Share Different Files
            </Button>
          </CardFooter>
        </Card>
      ) : (
        <div className="space-y-4">
          <Card className="border-primary/20">
            <CardHeader className="pb-2 pt-5">
              <CardTitle className="text-lg sm:text-xl text-center">Select Files to Share</CardTitle>
            </CardHeader>
            <CardContent className="px-4">
              <div className="grid grid-cols-1 gap-3">
                <Button
                  variant="outline"
                  className="h-auto py-5 flex flex-col items-center gap-3 border-primary/20 hover:border-primary/50 hover:bg-primary/5"
                  onClick={handleFileSelect}
                  disabled={status === 'processing'}
                >
                  {status === 'processing' ? (
                    <div className="flex items-center gap-2">
                      <Loader2 className="h-5 w-5 animate-spin text-primary" />
                      <span>Processing...</span>
                    </div>
                  ) : (
                    <>
                      <Upload className="h-8 w-8 text-primary" />
                      <div className="text-center">
                        <div className="font-medium text-sm sm:text-base">Select Files</div>
                        <div className="text-xs text-muted-foreground">Choose individual files to share</div>
                      </div>
                    </>
                  )}
                </Button>

                <Button
                  variant="outline"
                  className="h-auto py-5 flex flex-col items-center gap-3 border-primary/20 hover:border-primary/50 hover:bg-primary/5"
                  onClick={handleDirSelect}
                  disabled={status === 'processing'}
                >
                  {status === 'processing' ? (
                    <div className="flex items-center gap-2">
                      <Loader2 className="h-5 w-5 animate-spin text-primary" />
                      <span>Processing...</span>
                    </div>
                  ) : (
                    <>
                      <FolderOpen className="h-8 w-8 text-primary" />
                      <div className="text-center">
                        <div className="font-medium text-sm sm:text-base">Select Folders</div>
                        <div className="text-xs text-muted-foreground">Choose entire folders to share</div>
                      </div>
                    </>
                  )}
                </Button>
              </div>
            </CardContent>
          </Card>

          {selectedFiles.length > 0 && (
            <Card className="border-primary/20">
              <CardHeader className="pb-2 pt-5">
                <CardTitle className="text-lg sm:text-xl">Selected Files ({selectedFiles.length})</CardTitle>
              </CardHeader>
              <CardContent className="px-4">
                <div className="border rounded-lg divide-y max-h-[250px] overflow-y-auto">
                  {selectedFiles.map((file, index) => (
                    <div key={index} className="flex items-center justify-between p-2 hover:bg-muted/50">
                      <div className="flex items-center gap-2 flex-1 min-w-0">
                        <File className="h-4 w-4 text-primary shrink-0" />
                        <div className="min-w-0">
                          <div className="font-medium text-sm truncate">{file.name}</div>
                          <div className="text-xs text-muted-foreground">{file.size}</div>
                        </div>
                      </div>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-7 w-7 text-muted-foreground hover:text-destructive shrink-0"
                        onClick={(e) => {
                          e.stopPropagation();
                          removeFile(index);
                        }}
                      >
                        <X className="h-4 w-4" />
                      </Button>
                    </div>
                  ))}
                </div>
              </CardContent>
              <CardFooter className="flex flex-col gap-3 pt-2 pb-5">
                <Button
                  variant="outline"
                  className="flex items-center gap-2 w-full"
                  onClick={handleFileSelect}
                  disabled={status === 'processing'}
                  size="sm"
                >
                  <Plus className="h-4 w-4" />
                  {status === 'processing' ? 'Processing...' : 'Add More Files'}
                </Button>

                <Button
                  className="flex items-center gap-2 w-full"
                  onClick={handleShare}
                  disabled={status === 'processing' || selectedFiles.length === 0}
                >
                  {status === 'processing' ? (
                    <>
                      <Loader2 className="h-4 w-4 animate-spin" />
                      <span>Preparing...</span>
                    </>
                  ) : (
                    'Generate Share Code'
                  )}
                </Button>
              </CardFooter>
            </Card>
          )}

          {status === 'processing' && (
            <Card className="border-primary/20">
              <CardContent className="py-4 px-4">
                <div className="space-y-3">
                  <div className="flex justify-between items-center text-sm">
                    <span className="text-muted-foreground">Preparing files...</span>
                    <span className="font-medium">30%</span>
                  </div>
                  <Progress value={30} className="h-2" />
                  <p className="text-xs sm:text-sm text-center text-muted-foreground">{statusMessage}</p>
                </div>
              </CardContent>
            </Card>
          )}

          {status === 'error' && statusMessage && (
            <Card className="border-destructive/20 bg-destructive/5">
              <CardContent className="py-4 px-4">
                <div className="flex items-center gap-2 text-destructive">
                  <X className="h-4 w-4 shrink-0" />
                  <p className="text-sm">{statusMessage}</p>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      )}
    </div>
  );
};

export default FileShare;