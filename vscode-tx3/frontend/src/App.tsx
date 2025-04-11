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
  const [trpEndpoint, setTrpEndpoint] = useState<string>("");
  const [trpHeaders, setTrpHeaders] = useState<Record<string, string>|undefined>(undefined);

  useEffect(() => {
    window.addEventListener('message', handleMessage);
    vscode.postMessage({ event: 'init' });
    return () => window.removeEventListener('message', handleMessage);
  }, []);

  const handleMessage = (event: MessageEvent<Tx[]>) => {
    setTxs(event.data);
  };

  return (
    <div className="root">
      <h3 className="panel-title">Tx3 Resolve</h3>

      <TrpServerForm
        onUpdate={(url: string, headers?: Record<string, string>) => {
          setTrpEndpoint(url);
          setTrpHeaders(headers);
        }}
      />
      
      <Title>Transactions</Title>
      {txs.map((tx, index) =>
        <TxForm
          key={index}
          tx={tx}
          trpEndpoint={trpEndpoint}
          trpHeaders={trpHeaders}
          collapsed={index !== 0}
        />
      )}
    </div>
  );
}

export default App
