type ArgValue = string | number | boolean | null | Uint8Array;

type Args = {
  [key: string]: ArgValue;
};

export type TirEnvelope = {
  version: string;
  bytecode: string;
  encoding: "base64" | "hex" | string;
};

export type ProtoTx = {
  tir: TirEnvelope;
  args: Args;
};

export type TxEnvelope = {
  tx: string;
  bytes: string;
  encoding: "base64" | "hex" | string;
};

export type ClientOptions = {
  endpoint: string;
  headers?: Record<string, string>;
  envArgs?: Args;
};

export class TRPClient {
  private readonly options: ClientOptions;

  constructor(options: ClientOptions) {
    this.options = options;
  }

  async resolve(protoTx: ProtoTx): Promise<TxEnvelope> {
    const response = await fetch(this.options.endpoint, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...this.options.headers,
      },
      body: JSON.stringify({
        jsonrpc: "2.0",
        method: "trp.resolve",
        params: {
          tir: protoTx.tir,
          args: protoTx.args,
          env: this.options.envArgs,
        },
        id: crypto.randomUUID(),
      }),
    });

    if (!response.ok) {
      throw new Error(`Failed to resolve transaction: ${response.statusText}`);
    }

    const result = await response.json();

    if (result.error) {
      throw new Error(`JSON-RPC error: ${result.error.message}`, { cause: result.error.data });
    }

    return result.result as TxEnvelope;
  }
}
