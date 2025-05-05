// src/App.tsx
import { useState } from 'react';
import './App.css';
import FileShare from './components/FileShare';
import FileReceive from './components/FileReceive';
import { invoke } from '@tauri-apps/api/core';

function App() {
  const [activeView, setActiveView] = useState<'home' | 'share' | 'receive'>('home');

  return (
    <div className="app-container">
      <div className="app-header">
        <h1>Secure File Transfer</h1>
        {activeView !== 'home' && (
          <button 
            className="back-button"
            onClick={async () => {
              await invoke<string>('stop_sharing_command', {
                verbose: false 
              });
              setActiveView('home')
            }}
          >
            Back to Home
          </button>
        )}
      </div>

      {activeView === 'home' && (
        <div className="card-container">
          <div 
            className="action-card send-card"
            onClick={() => setActiveView('share')}
          >
            <div className="card-icon">
              <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                <polyline points="17 8 12 3 7 8"></polyline>
                <line x1="12" y1="3" x2="12" y2="15"></line>
              </svg>
            </div>
            <h2>Share Files</h2>
            <p>Generate a code to share your files securely</p>
          </div>

          <div 
            className="action-card receive-card"
            onClick={() => setActiveView('receive')}
          >
            <div className="card-icon">
              <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                <polyline points="7 10 12 15 17 10"></polyline>
                <line x1="12" y1="15" x2="12" y2="3"></line>
              </svg>
            </div>
            <h2>Receive Files</h2>
            <p>Enter a code to receive shared files</p>
          </div>
        </div>
      )}

      {activeView === 'share' && <FileShare />}
      {activeView === 'receive' && <FileReceive />}
    </div>
  );
}

export default App;