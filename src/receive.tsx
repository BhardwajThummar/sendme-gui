import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getDownloadsFolderPath } from './utils/paths';
import { logger } from './utils/logger';

const ReceiveFile: React.FC = () => {
  const [ticket, setTicket] = useState<string>('');
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

  const handleReceive = async () => {
    setStatus("Receiving file...");
    try {
      const result = await invoke<string>('receive_file_command', { ticket, fileStoragePath, verbose : false });
      setStatus(result);
    } catch (error) {
      logger.error('ReceiveFile', 'Error receiving file', error);
      setStatus(`Error: ${error}`);
    }
  };

  return (
    <div style={{ padding: '2rem' }}>
      <h2>Receive File</h2>
      <input
        type="text"
        placeholder="Enter Receive Code Here"
        value={ticket}
        onChange={(e) => setTicket(e.target.value)}
        style={{ width: '80%', marginBottom: '1rem' }}
      />
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
      <button onClick={handleReceive}>Receive File</button>
      <p>Status: {status}</p>
    </div>
  );
};

export default ReceiveFile;
