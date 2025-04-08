import React, { useState } from "react";
import { TRPClient, TxEnvelope, ProtoTx } from "tx3-trp";

import Box from "./Box";
import Input from "./Input";
import Button from "./Button";

import type { Tx } from "../App";

interface TxFormProps {
  tx: Tx;
  trpEndpoint: string;
  trpHeaders?: Record<string, string>;
  collapsed?: boolean;
}

const getParameterType = (key: string, tx: Tx): string => (
  Object.entries(tx.parameters).find(([name, type]) => name === key)!![1]
);

const getFormValue = (key: string, value: FormDataEntryValue, tx: Tx): string|number => (
  getParameterType(key, tx) === "Int" ? parseInt(value.toString()) : value.toString()
);

const TxForm: React.FC<TxFormProps> = (props: TxFormProps) => {
  const [loading, setLoading] = useState<boolean>(false);
  const [response, setResponse] = useState<string>("");
  const [error, setError] = useState<string>("");

  const handleSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    setLoading(true);
    setError("");
    setResponse("");

    event.preventDefault();
    const formData = new FormData(event.currentTarget);
    const args = Object.fromEntries([...formData].map(([key, value]) => [key, getFormValue(key, value, props.tx)]));
    
    const protoTx: ProtoTx = {
      tir: {
        version: 'v1alpha1',
        encoding: 'hex',
        bytecode: props.tx.tir
      },
      args
    };

    executeTx(protoTx)
      .then((response: TxEnvelope) => {
        console.log(response);
        setResponse((response as any).tx);
      })
      .catch((error: Error) => {
        console.error(error);
        setError(error.message);
      })
      .finally(() => setLoading(false));
  };

  const executeTx = async (tx: ProtoTx): Promise<TxEnvelope> => {
    const client = new TRPClient({
      endpoint: props.trpEndpoint,
      headers: props.trpHeaders,
    });
    return await client.resolve(tx);
  }

  return (
    <Box
      collapsible
      collapsed={props.collapsed}
      title={`Tx ${props.tx.name}`}
    >
      <form onSubmit={handleSubmit}>
        {props.tx.parameters && Object.entries(props.tx.parameters).map(([name, type], index) =>
          <Input
            key={index}
            label={name}
            placeholder={type}
            name={name}
            type={type}
            disabled={loading}
          />
        )}

        <div className="text-right">
          <Button
            loading={loading}
            loadingLabel="Resolving..."
            label="Resolve Tx"
          />
        </div>
      </form>

      {response.length > 0 &&
        <div className="tx-response">
          <h3 className="tx-response-title">Resolved Tx</h3>
          <p className="tx-response-content">{response}</p>
        </div>
      }

      {error.length > 0 &&
        <div className="tx-response">
          <h3 className="tx-response-title">Error resolving Tx</h3>
          <p className="tx-response-content">{error}</p>
        </div>
      }
    </Box>
  );
}

export default TxForm;