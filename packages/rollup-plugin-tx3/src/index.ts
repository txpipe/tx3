import type { Plugin } from "rollup";
import { execSync } from "child_process";
import path from "path";
import fs from "fs";
import { globSync } from "glob";

export interface Tx3PluginOptions {
  // Path to the tx3-bindgen executable
  bingenPath?: string;
  // Input tx3 files or glob patterns
  inputFiles: string[];
  // Output directory for generated bindings (relative to project root)
  outputDir?: string;
  // Additional arguments to pass to tx3-bindgen
  bingenArgs?: string[];
}

export default function tx3RollupPlugin(
  options: Tx3PluginOptions
): Plugin & { regenerateBindings: () => void } {
  const {
    bingenPath = "tx3-bindgen",
    outputDir = "node_modules/.tx3",
    inputFiles,
    bingenArgs = [],
  } = options;

  let projectRoot: string;
  let absoluteOutputDir: string;

  function generateBindings() {
    fs.mkdirSync(absoluteOutputDir, { recursive: true });

    const command = [
      bingenPath,
      "-i",
      ...inputFiles,
      "-o",
      absoluteOutputDir,
      ...bingenArgs,
      "-t",
      "typescript",
    ].join(" ");

    try {
      execSync(command, { stdio: "inherit" });
    } catch (error) {
      console.error("Failed to generate TX3 bindings:", error);
      throw error;
    }
  }

  const plugin: Plugin & { regenerateBindings: () => void } = {
    name: "rollup-plugin-tx3",

    buildStart() {
      projectRoot = process.cwd();
      absoluteOutputDir = path.resolve(projectRoot, outputDir);

      generateBindings();
    },

    // Ensure bindings exist for production builds
    buildEnd() {
      const files = globSync(`${absoluteOutputDir}/**/*.ts`);

      if (files.length === 0) {
        generateBindings();
      }
    },

    // Expose regenerateBindings for the Vite plugin
    regenerateBindings: generateBindings,
  };

  return plugin;
}
