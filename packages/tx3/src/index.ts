type ArgValue = string | number | boolean | null | Uint8Array;

type Args = {
  [key: string]: ArgValue;
};

export type IrEnvelope = {
  version: string;
  bytecode: string;
  encoding: "base64" | "hex";
};

export type IRInstruction = {
  name: string;
  args: Args;
};

export type ProtoTx = {
  ir: IrEnvelope;
  args: Args;
};

export type Tx = {
  hex: string;
};

export async function resolveProtoTx(protoTx: ProtoTx): Promise<Tx> {
  const response = await fetch("http://localhost:8000/", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      jsonrpc: "2.0",
      method: "tx3v1alpha1.resolve_proto_tx",
      params: {
        ir: protoTx.ir,
        args: protoTx.args,
      },
      id: "1",
    }),
  });

  if (!response.ok) {
    throw new Error(`Failed to resolve transaction: ${response.statusText}`);
  }

  const result = await response.json();

  if (result.error) {
    throw new Error(`JSON-RPC error: ${result.error.message}`);
  }

  return result.result as Tx;
}
