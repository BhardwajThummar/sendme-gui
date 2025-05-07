// src/App.tsx
import { useState } from 'react';
import FileShare from './components/FileShare';
import FileReceive from './components/FileReceive';
import { invoke } from '@tauri-apps/api/core';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './components/ui/tabs';
import { ToastProvider } from './components/ui/toast';
import { Upload, Download } from 'lucide-react';

function App() {
  const [activeTab, setActiveTab] = useState<string>('share');

  // Function to handle stopping sharing when switching tabs
  const handleTabChange = async (value: string) => {
    if (activeTab === 'share' && value !== 'share') {
      await invoke<string>('stop_sharing_command', {
        verbose: false
      });
    }
    setActiveTab(value);
  };

  return (
    <ToastProvider>
      <div className="min-h-screen bg-background flex flex-col items-center justify-center p-4">
        {/* Mobile-like container with fixed width */}
        <div className="w-full max-w-[360px] h-[640px] flex flex-col bg-background border border-border rounded-xl shadow-lg overflow-hidden">
          <header className="flex items-center justify-center py-3 border-b">
            <h1 className="text-lg font-bold text-foreground">Secure File Transfer</h1>
          </header>

          <Tabs
            defaultValue="share"
            value={activeTab}
            onValueChange={handleTabChange}
            className="flex flex-col flex-1"
          >
            <div className="border-b px-4 py-2">
              <TabsList className="w-full h-12 bg-muted rounded-lg p-1 gap-2">
                <TabsTrigger
                  value="share"
                  className="flex-1 rounded-md data-[state=active]:bg-primary data-[state=active]:text-primary-foreground"
                >
                  <div className="flex items-center justify-center gap-2">
                    <Upload className="h-4 w-4" />
                    <span>Share</span>
                  </div>
                </TabsTrigger>
                <TabsTrigger
                  value="receive"
                  className="flex-1 rounded-md data-[state=active]:bg-primary data-[state=active]:text-primary-foreground"
                >
                  <div className="flex items-center justify-center gap-2">
                    <Download className="h-4 w-4" />
                    <span>Receive</span>
                  </div>
                </TabsTrigger>
              </TabsList>
            </div>

            <main className="flex-1 overflow-auto">
              <TabsContent value="share" className="flex-1 p-4 m-0 h-full overflow-auto data-[state=active]:block">
                <FileShare />
              </TabsContent>

              <TabsContent value="receive" className="flex-1 p-4 m-0 h-full overflow-auto data-[state=active]:block">
                <FileReceive />
              </TabsContent>
            </main>
          </Tabs>

          <footer className="text-center text-xs text-muted-foreground py-3 border-t">
            <p>Secure File Transfer • Share files safely</p>
          </footer>
        </div>
      </div>
    </ToastProvider>
  );
}

export default App;