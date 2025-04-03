import { useEffect, useState } from "react";

interface Parameter {
  name: string;
  type: string;
}

interface Tx {
  name: string;
  parameters: Parameter[];
  inputs: { name: string }[];
  outputs: { name: string }[];
  ir: string;
}

interface Data {
  parties: { name: string }[];
  txs: Tx[];
}

interface FieldProps {
  parameter: Parameter;
}

const Field: React.FC<FieldProps> = (props: FieldProps) => (
  <>
    <p className="text-neutral-300 text-sm my-1">
      {props.parameter.name}
    </p>
    <input
      type="text"
      name={props.parameter.name}
      placeholder={props.parameter.type}
      className="form-input w-full mb-2 px-3 py-2 text-base text-neutral-300 border-neutral-400 bg-neutral-800"
    />
  </>
);

interface TxProps {
  tx: Tx;
}

// TODO: Add form validations
const Tx: React.FC<TxProps> = (props: TxProps) => (
  <div>
    <h3 className="text-xl my-2 text-neutral-300">Tx {props.tx.name}</h3>
    {props.tx.parameters && props.tx.parameters.map((parameter, index) =>
      <Field key={index} parameter={parameter} />
    )}
    <div className="text-right">
      <input
        type="submit"
        value="Send tx"
        className="bg-transparent hover:bg-white text-white font-semibold hover:text-neutral-800 py-2 px-4 border border-white hover:border-transparent"
      />
    </div>
  </div>
);

// TODO: Add error handling, add loading states, split the components into their own files
function App() {
  const [data, setData] = useState<Data|null>(null);
  const [message, setMessage] = useState<string>("");

  useEffect(() => {
    window.addEventListener('message', handleMessage);
    vscode.postMessage({ event: 'init' });
    return () => window.removeEventListener('message', handleMessage);
  }, []);

  const handleMessage = (event: MessageEvent<Data>) => {
    setData(event.data);
  };

  const handleSubmit = (tx: Tx, event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const formData = new FormData(event.currentTarget);
    setMessage(JSON.stringify({
      tx: tx.name,
      ir: tx.ir,
      parameters: Object.fromEntries(formData)
    }, null, 2));
  };

  return (
    <div className="w-full min-h-screen p-4 bg-neutral-800">
      <h1 className="text-2xl mb-4 text-neutral-300">Tx3 preview</h1>
      {data && data.txs && data.txs.map((tx, index) =>
        <form key={index} onSubmit={(e) => handleSubmit(tx, e)}>
          <Tx tx={tx} />
        </form>
      )}
      <pre className="mt-4 overflow-x-hidden overflow-ellipsis">
        {message}
      </pre>
    </div>
  );
}

export default App
