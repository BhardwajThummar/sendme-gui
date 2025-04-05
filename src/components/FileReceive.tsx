// src/components/FileReceive.tsx
import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import { getDownloadsFolderPath } from '../utils/paths';
import { listen } from '@tauri-apps/api/event';
import './FileReceive.css';

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
        multiple: true,
        title: 'Select Download Location',
      });
      
      if (selected && !Array.isArray(selected)) {
        setDownloadPath(selected);
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

  return (
    <div className="file-receive-container">
      <h2>Receive Files</h2>
      
      {downloadComplete ? (
        <div className="download-complete">
          <div className="download-complete-icon">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
              <polyline points="22 4 12 14.01 9 11.01"></polyline>
            </svg>
          </div>
          <h3>Download Complete!</h3>
          {downloadStats && (
            <div className="download-stats">
              <p>
                Downloaded: <strong>{fileName}</strong>
                {downloadStats.totalFiles > 1 && ` (${downloadStats.totalFiles} files)`}
              </p>
              <p>Total Size: <strong>{downloadStats.totalSize}</strong></p>
              <p>Time: <strong>{downloadStats.elapsedTime}</strong> • Average Speed: <strong>{downloadStats.speed}</strong></p>
            </div>
          )}
          <p>Your file has been saved to:</p>
          <div className="download-path">{downloadPath}</div>
          <button className="receive-again-button" onClick={handleReset}>
            Receive Another File
          </button>
        </div>
      ) : (
        <>
          <div className="code-input-container">
            <label>Enter your 6-character receive code</label>
            <div className="code-inputs">
              {code.map((char, index) => (
                <input
                  key={index}
                  ref={(el) => inputRefs.current[index] = el}
                  type="text"
                  maxLength={1}
                  value={char}
                  onChange={(e) => handleCodeChange(index, e.target.value)}
                  onKeyDown={(e) => handleKeyDown(index, e)}
                  onPaste={index === 0 ? handlePaste : undefined}
                  className="code-input"
                  disabled={status === 'processing'}
                />
              ))}
            </div>
          </div>
          
          <div className="download-path-container">
            <label>Save files to</label>
            <div className="download-path-input">
              <input
                type="text"
                value={downloadPath}
                onChange={(e) => setDownloadPath(e.target.value)}
                placeholder="Download location"
                disabled={status === 'processing'}
              />
              <button 
                className="browse-button"
                onClick={handleBrowse}
                disabled={status === 'processing'}
              >
                Browse
              </button>
            </div>
          </div>
          
          {status === 'processing' && (
            <div className="download-progress">
              <div className="file-info">
                <div className="file-name">{fileName || 'Downloading file...'}</div>
                <div className="progress-percentage">{Math.round(progress)}%</div>
              </div>
              <div className="progress-bar-container">
                <div 
                  className="progress-bar"
                  style={{ width: `${progress}%` }}
                ></div>
              </div>
            </div>
          )}
          
          <div className="action-buttons">
            <button 
              className="receive-button"
              onClick={handleReceive}
              disabled={code.join('').length !== 6 || !downloadPath || status === 'processing'}
            >
              {status === 'processing' ? (
                <>
                  <div className="spinner"></div>
                  Receiving...
                </>
              ) : (
                'Receive Files'
              )}
            </button>
            
            {(status === 'error' || status === 'success') && (
              <button 
                className="reset-button"
                onClick={handleReset}
              >
                Reset
              </button>
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

export default FileReceive;