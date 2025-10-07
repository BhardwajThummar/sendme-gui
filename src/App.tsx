// src/App.tsx
import { useState } from 'react';
import FileSend from './components/FileSend';
import FileReceive from './components/FileReceive';
import { invoke } from '@tauri-apps/api/core';
import { ToastProvider } from './components/ui/toast';
import { Upload, Download } from 'lucide-react';
import { Button } from './components/ui/button';

function App() {
  const [activeTab, setActiveTab] = useState<string>('send');

  // Function to handle tab change - simplified to be synchronous
  const handleTabChange = (value: string) => {
    console.log('[App] handleTabChange called with:', value);
    console.log('[App] Current activeTab state:', activeTab);

    // Set tab immediately - don't wait for async
    setActiveTab(value);
    console.log('[App] setActiveTab called with:', value);

    // Stop sharing in background if switching away from send
    if (activeTab === 'send' && value !== 'send') {
      console.log('[App] Calling stop_sharing_command in background');
      invoke<string>('stop_sharing_command', { verbose: false })
        .then(() => console.log('[App] Stop sharing succeeded'))
        .catch((error) => console.error('[App] Stop sharing failed:', error));
    }
  };

  return (
    <ToastProvider>
      <div className="min-h-screen bg-background flex flex-col">
        {/* Full-screen container with footer at bottom */}
        <div className="w-full flex flex-col flex-grow bg-background overflow-hidden">
          <header className="flex items-center justify-center py-3 border-b">
            <h1 className="text-lg font-bold text-foreground">
              Secure File Transfer
            </h1>
          </header>

          {/* Debug info */}
          <div className="text-xs bg-yellow-100 text-black p-2 text-center font-bold">
            Active tab: {activeTab} | Tap tabs below to switch
          </div>

          {/* Simple button-based tabs */}
          <div className="border-b px-4 py-2">
            <div className="w-full h-14 bg-background border border-border rounded-xl p-1.5 gap-3 flex">
              <Button
                onClick={() => {
                  console.log('[App] Send button clicked');
                  handleTabChange('send');
                }}
                className={`flex-1 rounded-lg h-full transition-all duration-200 ${
                  activeTab === 'send'
                    ? 'bg-primary text-primary-foreground shadow-sm'
                    : 'bg-transparent text-foreground hover:bg-muted'
                }`}
                variant={activeTab === 'send' ? 'default' : 'ghost'}
              >
                <div className="flex items-center justify-center gap-2">
                  <Upload className="h-4 w-4" />
                  <span className="font-medium">Send</span>
                </div>
              </Button>
              <Button
                onClick={() => {
                  console.log('[App] Receive button clicked');
                  handleTabChange('receive');
                }}
                className={`flex-1 rounded-lg h-full transition-all duration-200 ${
                  activeTab === 'receive'
                    ? 'bg-primary text-primary-foreground shadow-sm'
                    : 'bg-transparent text-foreground hover:bg-muted'
                }`}
                variant={activeTab === 'receive' ? 'default' : 'ghost'}
              >
                <div className="flex items-center justify-center gap-2">
                  <Download className="h-4 w-4" />
                  <span className="font-medium">Receive</span>
                </div>
              </Button>
            </div>
          </div>

          {/* Tab content */}
          <main className="flex-grow overflow-auto">
            {/* Debug: show both tabs visibility */}
            <div className="text-xs bg-blue-100 text-black p-2 text-center">
              Send visible: {activeTab === 'send' ? 'YES' : 'NO'} | Receive visible: {activeTab === 'receive' ? 'YES' : 'NO'}
            </div>

            <div className={`h-full p-4 ${activeTab === 'send' ? 'block' : 'hidden'}`}>
              <FileSend />
            </div>
            <div className={`h-full p-4 ${activeTab === 'receive' ? 'block' : 'hidden'}`}>
              <FileReceive />
            </div>
          </main>

          <footer className="mt-auto text-center text-xs text-muted-foreground py-3 border-t">
            <p>Secure File Transfer • Send files safely</p>
          </footer>
        </div>
      </div>
    </ToastProvider>
  );
}

export default App;
