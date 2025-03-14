import { useCallback } from "react";

import { protocol as faucetProtocol } from "@tx3/faucet";
import { protocol as vestingProtocol } from "@tx3/vesting";

import reactLogo from "./assets/react.svg";
import viteLogo from "/vite.svg";
import "./App.css";

function App() {
  const onClick = useCallback(async () => {
    const tx = await faucetProtocol.claimWithPasswordTx({
      password: crypto.getRandomValues(new Uint8Array(32)),
      quantity: 1,
      reminder: 0,

      //requester: "addr1q8flellq7q8akyykwjsk3ywdcty5el23glwgnheqq95mmk9zsezg7hgfrk3gd82rdf7fxcp9rmp4h42cs48pk2t038hq2gr6hu",
    });

    //const txId = await faucetProtocol.submit(tx);

    const vestingTx = await vestingProtocol.lockTx({
      quantity: 1,
      until: Date.now() + 1000 * 60 * 60 * 24 * 30,
    });

    const unlockTx = await vestingProtocol.unlockTx({
      lockedUtxo: "abced#1",
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
