// src/components/FileShare.tsx
import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import './FileShare.css';

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

  return (
    <div className="file-share-container">
      <h2>Share Files</h2>
      
      {shareCode ? (
        <div className="share-code-container">
          <div className="share-code-header">
            <h3>Your Share Code</h3>
            <div className="pulse-animation"></div>
          </div>
          <div className="share-code">
            {shareCode.split('').map((char, index) => (
              <span key={index} className="share-code-char">{char}</span>
            ))}
          </div>
          <p className="share-code-instructions">
            Share this code with the recipient to let them download your files
          </p>
          <button className="copy-code-button" onClick={copyCodeToClipboard}>
            Copy Code
          </button>
          <button className="reset-button" onClick={handleReset}>
            Share Different Files
          </button>
        </div>
      ) : (
        <>
          <div className="file-selection-area">
            <div className="file-browse-container">
              <button 
                className="file-browse-button"
                onClick={handleFileSelect}
                disabled={status === 'processing'}
              >
                {status === 'processing' ? (
                  <>
                    <div className="spinner"></div>
                    Processing...
                  </>
                ) : (
                  <>
                    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                      <polyline points="17 8 12 3 7 8"></polyline>
                      <line x1="12" y1="3" x2="12" y2="15"></line>
                    </svg>
                    Select Files to Share
                  </>
                )}
              </button>
              <button 
                className="file-browse-button"
                onClick={handleDirSelect}
                disabled={status === 'processing'}
              >
                {status === 'processing' ? (
                  <>
                    <div className="spinner"></div>
                    Processing...
                  </>
                ) : (
                  <>
                    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                      <polyline points="17 8 12 3 7 8"></polyline>
                      <line x1="12" y1="3" x2="12" y2="15"></line>
                    </svg>
                    Select Directories to Share
                  </>
                )}
              </button>
              <p className="file-browse-hint">
                Use the button above to select files from your computer
              </p>
            </div>

            {selectedFiles.length > 0 && (
              <div className="selected-files-container">
                <h3>Selected Files</h3>
                <ul className="file-list">
                  {selectedFiles.map((file, index) => (
                    <li key={index} className="file-item">
                      <div className="file-icon">
                        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                          <polyline points="14 2 14 8 20 8"></polyline>
                          <line x1="16" y1="13" x2="8" y2="13"></line>
                          <line x1="16" y1="17" x2="8" y2="17"></line>
                          <polyline points="10 9 9 9 8 9"></polyline>
                        </svg>
                      </div>
                      <div className="file-details">
                        <div className="file-name">{file.name}</div>
                        <div className="file-size">{file.size}</div>
                      </div>
                      <button 
                        className="remove-file-button"
                        onClick={(e) => {
                          e.stopPropagation();
                          removeFile(index);
                        }}
                      >
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                          <circle cx="12" cy="12" r="10"></circle>
                          <line x1="15" y1="9" x2="9" y2="15"></line>
                          <line x1="9" y1="9" x2="15" y2="15"></line>
                        </svg>
                      </button>
                    </li>
                  ))}
                </ul>
                
                <div className="action-buttons">
                  <button 
                    className="add-more-button"
                    onClick={handleFileSelect}
                    disabled={status === 'processing'}
                  >
                    {status === 'processing' ? 'Processing...' : 'Add More Files'}
                  </button>
                  
                  <button 
                    className="share-button"
                    onClick={handleShare}
                    disabled={status === 'processing' || selectedFiles.length === 0}
                  >
                    {status === 'processing' ? (
                      <>
                        <div className="spinner"></div>
                        Preparing...
                      </>
                    ) : (
                      'Generate Share Code'
                    )}
                  </button>
                </div>
              </div>
            )}
          </div>
          
          {statusMessage && (
            <div className={`status-message ${status === 'error' ? 'error' : ''}`}>
              {statusMessage}
            </div>
          )}
        </>
      )}
    </div>
  );
};

export default FileShare;