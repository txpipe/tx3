import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";
import tx3Plugin from "vite-plugin-tx3";

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");

  return {
    plugins: [
      tx3Plugin({
        inputFiles: ["tx3/**/*.tx3"],
        trpEndpoint: env.TRP_ENDPOINT,
        trpHeaders: {
          "dmtr-api-key": env.DMTR_API_KEY,
        },
        envArgs: {
          timelock_script: env.TIMELOCK_SCRIPT,
        },
      }),
      react(),
    ],
  };
});
