import { useCallback } from "react";

import { claimWithPassword } from "@tx3/transactions";

import reactLogo from "./assets/react.svg";
import viteLogo from "/vite.svg";
import "./App.css";

function App() {
  const onClick = useCallback(async () => {
    const tx = await claimWithPassword({
      password: crypto.getRandomValues(new Uint8Array(32)),
      quantity: 1,
      reminder: 0,

      //requester: "addr1q8flellq7q8akyykwjsk3ywdcty5el23glwgnheqq95mmk9zsezg7hgfrk3gd82rdf7fxcp9rmp4h42cs48pk2t038hq2gr6hu",
    });

    console.log(tx);
  }, []);

  return (
    <>
      <div>
        <a href="https://vite.dev" target="_blank">
          <img src={viteLogo} className="logo" alt="Vite logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <h1>Vite + React</h1>
      <div className="card">
        <button onClick={onClick}>Resolve Tx</button>
        <p>
          Edit <code>src/App.tsx</code> and save to test HMR
        </p>
      </div>
      <p className="read-the-docs">
        Click on the Vite and React logos to learn more
      </p>
    </>
  );
}

export default App;
