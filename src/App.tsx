import { useState, useEffect } from 'react';
import FileSend from './components/FileSend';
import FileReceive from './components/FileReceive';
import { ToastProvider } from './components/ui/toast';
import { Upload, Download, Activity } from 'lucide-react';
import { Button } from './components/ui/button';
import { logger } from './utils/logger';
import { TransferProvider, useTransfer } from './context/TransferContext';
import { getVersionString } from './utils/version';
import { listen } from '@tauri-apps/api/event';

function AppContent() {
  const [activeTab, setActiveTab] = useState<string>('send');
  const { transferState } = useTransfer();

  // Listen for background service events from Rust
  useEffect(() => {
    const unlistenStart = listen('start-background-service', () => {
      logger.info('App', 'Received start-background-service event');
      // Call Android plugin to start foreground service
      if ((window as any).BackgroundServicePlugin) {
        try {
          const result = (window as any).BackgroundServicePlugin.startBackgroundService();
          logger.info('App', `Background service started: ${result}`);
        } catch (error) {
          logger.error('App', `Failed to start background service: ${error}`);
        }
      }
    });

    const unlistenStop = listen('stop-background-service', () => {
      logger.info('App', 'Received stop-background-service event');
      // Call Android plugin to stop foreground service
      if ((window as any).BackgroundServicePlugin) {
        try {
          const result = (window as any).BackgroundServicePlugin.stopBackgroundService();
          logger.info('App', `Background service stopped: ${result}`);
        } catch (error) {
          logger.error('App', `Failed to stop background service: ${error}`);
        }
      }
    });

    return () => {
      unlistenStart.then(fn => fn());
      unlistenStop.then(fn => fn());
    };
  }, []);

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
            <p>Secure File Transfer • Send files safely • {getVersionString()}</p>
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
