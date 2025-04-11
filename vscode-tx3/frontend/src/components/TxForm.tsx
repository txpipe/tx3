import React, { useState } from "react";
import { TRPClient, TxEnvelope, ProtoTx } from "tx3-trp";

import Box from "./Box";
import Form, { FieldType } from "./Form";

import type { Tx } from "../App";

interface TxFormProps {
  tx: Tx;
  trpEndpoint: string;
  trpHeaders?: Record<string, string>;
  collapsed?: boolean;
}

const TxForm: React.FC<TxFormProps> = (props: TxFormProps) => {
  const [response, setResponse] = useState<string>("");
  const [error, setError] = useState<string>("");

  const handleSubmit = async (args: Record<string, string|number|boolean>) => {
    setError("");
    setResponse("");
    
    const protoTx: ProtoTx = {
      tir: {
        version: 'v1alpha1',
        encoding: 'hex',
        bytecode: props.tx.tir
      },
      args
    };
    
    const result = await executeTx(protoTx).catch(error => {
      console.error(error);
      setError(`${error.message}${error.cause?`\n${error.cause}`:""}`);
    });

    if (result) {
      setResponse(result.tx);
    }
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
      <Form
        onSubmit={handleSubmit}
        fields={
          Object.entries(props.tx.parameters).map(([name, type]) => ({
            name,
            placeholder: type,
            type: FieldType[type as keyof typeof FieldType],
            required: true,
          }))
        }
      />

      {response.length > 0 &&
        <div className="tx-response">
          <h3 className="tx-response-title">Resolved Tx</h3>
          <p className="tx-response-content">{response}</p>
        </div>
      }

      {error.length > 0 &&
        <div className="tx-response">
          <h3 className="tx-response-title">Error resolving Tx</h3>
          <p className="tx-response-content whitespace-pre-line">{error}</p>
        </div>
      }
    </Box>
  );
}

export default TxForm;