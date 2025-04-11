import React, { useEffect, useState } from "react";

import Title from "./components/Title";
import TrpServerForm from "./components/TrpServerForm";
import TxForm from "./components/TxForm";

export interface Tx {
  name: string;
  parameters: Record<string, string>;
  tir: string;
}

function App() {
  const [txs, setTxs] = useState<Tx[]>([]);
  const [trpServer, setTrpServer] = useState<TrpServer>(config.trpServers[0]);
  const [trpServers, setTrpServers] = useState<TrpServer[]>(config.trpServers);

  useEffect(() => {
    window.addEventListener('message', handleMessage);
    vscode.postMessage({ event: 'init' });
    return () => window.removeEventListener('message', handleMessage);
  }, []);

  const handleMessage = (event: MessageEvent<AppEvent>) => {
    const eventData = event.data;
    switch (eventData.type) {
      // If servers changed, notify the form
      case 'config':
        setTrpServers(eventData.data.trpServers);
        break;
      case 'txs':
        setTxs(eventData.data);
        break;
      default:
        break;
    }
  };

  return (
    <div className="root">
      <TrpServerForm
        onUpdate={setTrpServer}
        trpServers={trpServers}
      />
      <Title>Transactions</Title>
      {txs.map((tx, index) =>
        <TxForm
          key={index}
          tx={tx}
          trpEndpoint={trpServer.url}
          trpHeaders={trpServer.headers}
          collapsed={index !== 0}
        />
      )}
    </div>
  );
}

export default App
