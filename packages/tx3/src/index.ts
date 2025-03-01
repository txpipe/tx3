type ArgValue = string | number | boolean | null;

type Args = {
  [key: string]: ArgValue;
};

export async function resolveTx(
  ir_base64: string,
  args: Args
): Promise<Uint8Array> {
  const response = await fetch("http://localhost:8000/resolve", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      jsonrpc: "2.0",
      method: "resolve_tx",
      params: {
        ir: ir_base64,
        args: args,
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

  // Expect the result to be a base64 or hex string representing the resolved transaction
  return new Uint8Array(result.result.tx);
}
