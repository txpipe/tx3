import { useEffect, useState } from "react";
import { TRPClient, TxEnvelope, ProtoTx } from "tx3-trp";

interface Tx {
  name: string;
  parameters: Object;
  tir: string;
}

interface FieldProps {
  name: string;
  type: string;
}

const Field: React.FC<FieldProps> = (props: FieldProps) => (
  <>
    <p className="text-neutral-300 text-sm my-1">
      {props.name}
    </p>
    <input
      name={props.name}
      placeholder={props.type}
      type={props.type === "Int" ? "number" : "text"}
      className="form-input w-full mb-2 px-3 py-2 text-base text-neutral-300 border-neutral-400 bg-neutral-800"
    />
  </>
);

interface TxProps {
  tx: Tx;
}

// TODO: Add form validations
const Tx: React.FC<TxProps> = (props: TxProps) => (
  <div className="mt-2 mb-4 pt-2 pb-4 px-4 border border-neutral-600">
    <h3 className="text-xl my-2 text-neutral-300">Tx {props.tx.name}</h3>
    {props.tx.parameters && Object.entries(props.tx.parameters).map(([name, type], index) =>
      <Field key={index} name={name} type={type} />
    )}
    <div className="text-right">
      <input
        type="submit"
        value="Resolve Tx"
        className="bg-transparent hover:bg-white text-white font-semibold hover:text-neutral-800 mt-2 py-2 px-4 border border-white hover:border-transparent"
      />
    </div>
  </div>
);

const getParameterType = (key: string, tx: Tx): string => (
  Object.entries(tx.parameters).find(([name, type]) => name === key)!![1]
);

const getFormValue = (key: string, value: FormDataEntryValue, tx: Tx): string|number => (
  getParameterType(key, tx) === "Int" ? parseInt(value.toString()) : value.toString()
);

// TODO: Add error handling, add loading states, split the components into their own files
function App() {
  const [txs, setTxs] = useState<Tx[]>([]);
  const [response, setResponse] = useState<string>("");
  const [message, setMessage] = useState<string>("");

  const [trpEndpoint, setTrpEndpoint] = useState<string>("https://cardano-preview.trp-m1.demeter.run");
  const [trpHeaders, setTrpHeaders] = useState<string>("{\"dmtr-api-key\": \"trpjodqbmjblunzpbikpcrl\"}");

  useEffect(() => {
    window.addEventListener('message', handleMessage);
    vscode.postMessage({ event: 'init' });
    return () => window.removeEventListener('message', handleMessage);
  }, []);

  const handleMessage = (event: MessageEvent<Tx[]>) => {
    setTxs(event.data);
  };

  const handleSubmit = (tx: Tx, event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const formData = new FormData(event.currentTarget);
    const args = Object.fromEntries([...formData].map(([key, value]) => [key, getFormValue(key, value, tx)]));
    
    const protoTx: ProtoTx = {
      tir: {
        version: 'v1alpha1',
        encoding: 'hex',
        bytecode: tx.tir
      },
      args
    };

    const ctx = { tx: tx.name, endpoint: trpEndpoint, headers: JSON.parse(trpHeaders), protoTx } as any;
    setMessage(JSON.stringify(ctx, null, 2));

    executeTx(protoTx)
      .then((response: TxEnvelope) => {
        ctx.response = response;
        setMessage(JSON.stringify(ctx, null, 2));
        setResponse((response as any).tx);
      }).catch((error: Error) => {
        ctx.error = error.message;
        setMessage(JSON.stringify(ctx, null, 2));
      });
  };

  const executeTx = async (tx: ProtoTx): Promise<TxEnvelope> => {
    const client = new TRPClient({
      endpoint: trpEndpoint,
      headers: JSON.parse(trpHeaders),
    });
    return await client.resolve(tx);
  }

  return (
    <div className="w-full min-h-screen p-4 bg-neutral-800">
      <h1 className="text-2xl mb-4 text-neutral-300">Tx3 preview</h1>
      <div className="mt-2 mb-4 py-2 px-4 border border-neutral-600">
        <p className="text-neutral-300 text-sm my-1">
          TRP Endpoint
        </p>
        <input
          type="text"
          value={trpEndpoint}
          onChange={(e) => setTrpEndpoint(e.target.value)}
          className="form-input w-full mb-2 px-3 py-2 text-base text-neutral-300 border-neutral-400 bg-neutral-800"
        />
        <p className="text-neutral-300 text-sm my-1">
          TRP Headers
        </p>
        <input
          type="text"
          value={trpHeaders}
          onChange={(e) => setTrpHeaders(e.target.value)}
          className="form-input w-full mb-2 px-3 py-2 text-base text-neutral-300 border-neutral-400 bg-neutral-800"
        />
      </div>
      {txs.map((tx, index) =>
        <form key={index} onSubmit={(e) => handleSubmit(tx, e)}>
          <Tx tx={tx} />
        </form>
      )}
      {response !== "" &&
        <div className="mt-2 mb-4 p-2 px-4 border border-neutral-600">
          <div className="mt-2 mb-4 flex justify-between items-center">
            <h3 className="m-0 p-0 text-xl text-neutral-300">Resolved Tx</h3>
            <button
              className="bg-transparent hover:bg-white text-xs text-white font-semibold hover:text-neutral-800 py-2 px-4 border border-white hover:border-transparent"
              onClick={() => navigator.clipboard.writeText(response)}
            >
              Copy Tx
            </button>
          </div>
          <textarea
            rows={10}
            value={response}
            className="form-input w-full mb-2 px-3 py-2 font-mono text-base text-neutral-300 border-neutral-400 bg-neutral-900"
          />
        </div>
      }
      {message !== "" &&
        <pre className="mt-4 p-4 bg-neutral-900 overflow-x-hidden overflow-ellipsis">
          {message}
        </pre>
      }
    </div>
  );
}

export default App
