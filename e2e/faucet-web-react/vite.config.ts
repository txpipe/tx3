import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tx3Plugin from "./src/vite-plugin-tx3";

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    tx3Plugin({
      // Adjust these paths according to where your tx3 files are located
      inputFiles: ["faucet.tx3"],
      // Optional: specify path to tx3-bindgen if it's not in PATH
      // bingenPath: '../../target/debug/tx3-bindgen',
      // Optional: additional arguments for tx3-bindgen
      // bingenArgs: ['--some-flag'],
    }),
    react(),
  ],
});
