import { useState } from 'react';
import FileSend from './components/FileSend';
import FileReceive from './components/FileReceive';
import { invoke } from '@tauri-apps/api/core';
import { ToastProvider } from './components/ui/toast';
import { Upload, Download } from 'lucide-react';
import { Button } from './components/ui/button';
import { logger } from './utils/logger';

function App() {
  const [activeTab, setActiveTab] = useState<string>('send');

  const handleTabChange = (value: string) => {
    logger.debug('App', `Tab change requested: ${activeTab} → ${value}`);

    setActiveTab(value);

    // Stop sharing in background if switching away from send
    if (activeTab === 'send' && value !== 'send') {
      invoke<string>('stop_sharing_command', { verbose: false })
        .then(() => logger.info('App', 'File sharing stopped'))
        .catch((error) => logger.error('App', 'Failed to stop sharing', error));
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

          <div className="border-b px-4 py-2">
            <div className="w-full h-14 bg-background border border-border rounded-xl p-1.5 gap-3 flex">
              <Button
                onClick={() => handleTabChange('send')}
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
                onClick={() => handleTabChange('receive')}
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

          <main className="flex-grow overflow-auto">
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
