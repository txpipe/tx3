import reactLogo from "./assets/react.svg";
import viteLogo from "/vite.svg";
import * as pepe from "tx3";
import "./App.css";
import { useCallback } from "react";

console.log(pepe);

const IR =
  "1000000001000000000000000f0000000c0000000000000070726f76696465645f676173010b00000009000000000000007265717565737465720300000001100000000100000000000000010b00000009000000000000007265717565737465720300000000010e0000000e0000000d0000000c0000000000000070726f76696465645f67617310000000010000000a00000001000000000000001c00000000000000ef7a1cebb2dc7de884ddf82f8fcbc91fe9750dcd8c12ec7643a99bbe0200000007000000000000004d59544f4b454e0b00000008000000000000007175616e74697479000000000000000001010a00000001000000000000001c00000000000000ef7a1cebb2dc7de884ddf82f8fcbc91fe9750dcd8c12ec7643a99bbe0200000007000000000000004d59544f4b454e0b00000008000000000000007175616e7469747900000000010b000000080000000000000070617373776f7264020000000000000000000000";

type ClaimWithPasswordParams = {
  password: string;
  quantity: number;
  requester: string;
};

function App() {
  const onClick = useCallback(() => {
    const args: ClaimWithPasswordParams = {
      password: "abc1",
      quantity: 1,
      requester:
        "addr1q8flellq7q8akyykwjsk3ywdcty5el23glwgnheqq95mmk9zsezg7hgfrk3gd82rdf7fxcp9rmp4h42cs48pk2t038hq2gr6hu",
    };
    pepe.resolveTx(IR, args);
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
