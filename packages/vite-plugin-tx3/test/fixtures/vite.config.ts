import { defineConfig } from "vite";
import tx3Plugin from "../../src";

export default defineConfig({
  plugins: [
    tx3Plugin({
      inputFiles: ["src/**/*.tx3"],
      outputDir: "node_modules/.tx3",
    }),
  ],
});
