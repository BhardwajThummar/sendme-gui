import { useState } from 'react';
import FileSend from './components/FileSend';
import FileReceive from './components/FileReceive';
import { ToastProvider } from './components/ui/toast';
import { Upload, Download, Activity } from 'lucide-react';
import { Button } from './components/ui/button';
import { logger } from './utils/logger';
import { TransferProvider, useTransfer } from './context/TransferContext';

function AppContent() {
  const [activeTab, setActiveTab] = useState<string>('send');
  const { transferState } = useTransfer();

  const handleTabChange = (value: string) => {
    logger.debug('App', `Tab change requested: ${activeTab} → ${value}`);
    setActiveTab(value);

    // Note: We no longer stop sharing when switching tabs
    // This allows the transfer to continue in the background
    // Users can manually stop sharing if needed
  };

  return (
    <ToastProvider>
      <div className="min-h-screen bg-background flex flex-col">
        {/* Full-screen container with footer at bottom */}
        <div className="w-full flex flex-col flex-grow bg-background overflow-hidden">
          <header className="flex items-center justify-center py-3 border-b relative">
            <h1 className="text-lg font-bold text-foreground">
              Secure File Transfer
            </h1>

            {/* Active transfer indicator */}
            {transferState.isSharing && activeTab !== 'send' && (
              <div className="absolute right-4 flex items-center gap-2 text-xs text-primary">
                <Activity className="h-4 w-4 animate-pulse" />
                <span>Sharing active</span>
              </div>
            )}
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
                  {transferState.isSharing && activeTab !== 'send' && (
                    <div className="h-2 w-2 rounded-full bg-primary animate-pulse" />
                  )}
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

function App() {
  return (
    <TransferProvider>
      <AppContent />
    </TransferProvider>
  );
}

export default App;
