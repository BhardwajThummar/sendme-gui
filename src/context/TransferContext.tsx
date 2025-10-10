import React, { createContext, useContext, useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';

interface TransferState {
  isSharing: boolean;
  sendCode: string | null;
  transferProgress: number;
}

interface TransferContextType {
  transferState: TransferState;
  setTransferState: React.Dispatch<React.SetStateAction<TransferState>>;
}

const TransferContext = createContext<TransferContextType | undefined>(undefined);

export const TransferProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [transferState, setTransferState] = useState<TransferState>({
    isSharing: false,
    sendCode: null,
    transferProgress: 0,
  });

  useEffect(() => {
    // Listen for transfer events to update global state
    const setupListeners = async () => {
      const unlisteners: (() => void)[] = [];

      unlisteners.push(
        await listen('send_transfer_started', () => {
          setTransferState(prev => ({ ...prev, isSharing: true }));
        })
      );

      unlisteners.push(
        await listen<{ percentage: number }>('send_transfer_progress', (event) => {
          setTransferState(prev => ({
            ...prev,
            isSharing: true,
            transferProgress: event.payload.percentage,
          }));
        })
      );

      return () => {
        unlisteners.forEach(unlisten => unlisten());
      };
    };

    const cleanupPromise = setupListeners();

    return () => {
      cleanupPromise.then(cleanup => cleanup());
    };
  }, []);

  return (
    <TransferContext.Provider value={{ transferState, setTransferState }}>
      {children}
    </TransferContext.Provider>
  );
};

export const useTransfer = () => {
  const context = useContext(TransferContext);
  if (context === undefined) {
    throw new Error('useTransfer must be used within a TransferProvider');
  }
  return context;
};
