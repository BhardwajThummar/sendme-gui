// src/App.tsx
import { useState } from 'react';
import FileShare from './components/FileShare';
import FileReceive from './components/FileReceive';
import { invoke } from '@tauri-apps/api/core';
import { Button } from './components/ui/button';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from './components/ui/card';
import { ToastProvider } from './components/ui/toast';
import { ArrowLeft, Upload, Download } from 'lucide-react';

function App() {
  const [activeView, setActiveView] = useState<'home' | 'share' | 'receive'>('home');

  return (
    <ToastProvider>
      <div className="min-h-screen bg-background flex flex-col">
        <div className="w-full px-4 py-4 sm:py-6 flex-1 flex flex-col">
          <header className="flex items-center justify-between mb-6 pb-3 border-b max-w-4xl mx-auto w-full">
            <h1 className="text-xl sm:text-2xl font-bold text-primary">Secure File Transfer</h1>
            {activeView !== 'home' && (
              <Button
                variant="ghost"
                size="sm"
                className="flex items-center gap-1"
                onClick={async () => {
                  await invoke<string>('stop_sharing_command', {
                    verbose: false
                  });
                  setActiveView('home')
                }}
              >
                <ArrowLeft className="h-4 w-4" />
                <span className="text-sm">Back</span>
              </Button>
            )}
          </header>

          <main className="flex-1 flex flex-col items-center justify-center w-full">
            {activeView === 'home' && (
              <div className="grid grid-cols-1 gap-6 w-full max-w-md mx-auto">
                <Card
                  className="cursor-pointer hover:shadow-lg transition-all duration-300 border-primary/20 hover:border-primary"
                  onClick={() => setActiveView('share')}
                >
                  <CardHeader className="text-center pb-0 pt-6">
                    <div className="w-16 h-16 sm:w-20 sm:h-20 rounded-full bg-primary/10 flex items-center justify-center mx-auto mb-4">
                      <Upload className="h-8 w-8 sm:h-10 sm:w-10 text-primary" />
                    </div>
                    <CardTitle className="text-lg sm:text-xl">Share Files</CardTitle>
                  </CardHeader>
                  <CardContent className="text-center pt-2 pb-4">
                    <p className="text-muted-foreground text-sm">
                      Generate a code to share your files securely
                    </p>
                  </CardContent>
                  <CardFooter className="pb-6">
                    <Button variant="default" className="w-full">
                      Start Sharing
                    </Button>
                  </CardFooter>
                </Card>

                <Card
                  className="cursor-pointer hover:shadow-lg transition-all duration-300 border-secondary/20 hover:border-secondary"
                  onClick={() => setActiveView('receive')}
                >
                  <CardHeader className="text-center pb-0 pt-6">
                    <div className="w-16 h-16 sm:w-20 sm:h-20 rounded-full bg-secondary/10 flex items-center justify-center mx-auto mb-4">
                      <Download className="h-8 w-8 sm:h-10 sm:w-10 text-secondary" />
                    </div>
                    <CardTitle className="text-lg sm:text-xl">Receive Files</CardTitle>
                  </CardHeader>
                  <CardContent className="text-center pt-2 pb-4">
                    <p className="text-muted-foreground text-sm">
                      Enter a code to receive shared files
                    </p>
                  </CardContent>
                  <CardFooter className="pb-6">
                    <Button variant="secondary" className="w-full">
                      Receive Files
                    </Button>
                  </CardFooter>
                </Card>
              </div>
            )}

            {activeView === 'share' && <FileShare />}
            {activeView === 'receive' && <FileReceive />}
          </main>

          <footer className="mt-6 text-center text-xs sm:text-sm text-muted-foreground py-4 max-w-4xl mx-auto w-full">
            <p>Secure File Transfer • Share files safely across devices</p>
          </footer>
        </div>
      </div>
    </ToastProvider>
  );
}

export default App;