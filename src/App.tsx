// src/App.tsx
import React
// ,{ useState } 
from 'react';
// import { invoke } from '@tauri-apps/api/tauri';
import ReceiveFile from './receive';
import SendFile from './send';

const App: React.FC = () => {
  // const [sharing, setSharing] = useState<boolean>(false);

  // stop_sharing
  // const stopSharing = async () => {
  //   setSharing(false);
  //   try {
  //     let response = await invoke('stop_sharing_command');
  //     console.log(response);
  //     // show response in ui in simple alert
  //     alert(response);

  //     // alert('File shared successfully!');
  //   } catch (error) {
  //     console.error('Error sharing file:', error);
  //     alert('Error sharing file: ' + error);
  //   } finally {
  //     setSharing(false);
  //   }
  // };

  return (
    <div style={{ padding: '2rem' }}>
      <h1>SendMe File Sharing</h1>
      <SendFile />
      {/* <button onClick={stopSharing} disabled={!filePath || sharing}>
        {sharing ? 'Stopping...' : 'Stop Sharing'}
      </button> */}
      <ReceiveFile />
    </div>
  );
};

export default App;
