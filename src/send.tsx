// src/App.tsx (or another component)
import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { getDownloadsFolderPath } from './utils/paths';

const SendFile: React.FC = () => {
  // const [verbose, setVerbose] = useState<boolean>(false);
  const [status, setStatus] = useState<string>('');
  const [fileStoragePath, setFileStoragePath] = useState<string>('');

  useEffect(() => {
    const initPath = async () => {
      const path = await getDownloadsFolderPath();
      setFileStoragePath(path);
    };
    initPath();
  }, []);

  const handleSend = async () => {
    setStatus("Preparing...");
    try {
      const result = await invoke<string>('send_file_command', { filePath:fileStoragePath, verbose: false });
      setStatus(result);
    } catch (error) {
      console.error('Error Prepating:', error);
      setStatus(`Error: ${error}`);
    }
  };

  return (
    <div style={{ padding: '2rem' }}>
      <h2>Send File</h2>
      {/* input for filepath we will need a selector */}
        <input
            type="text"
            placeholder="Enter file path here"
            value={fileStoragePath}
            onChange={(e) => setFileStoragePath(e.target.value)}
            style={{ width: '80%', marginBottom: '1rem' }}
        /> 
      <div>
        {/* <label>
          <input
            type="checkbox"
            checked={verbose}
            onChange={(e) => setVerbose(e.target.checked)}
          />
          Verbose
        </label> */}
      </div>
      <button onClick={handleSend}>Send File</button>
      <p>Send Code: {status}</p>
    </div>
  );
};

export default SendFile;
