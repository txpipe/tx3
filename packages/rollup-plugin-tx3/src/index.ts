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
  // TRP endpoint to use for code generation
  trpEndpoint?: string;
  // TRP headers to use for code generation
  trpHeaders?: Record<string, string>;
  // Env args to use for code generation
  envArgs?: Record<string, string>;
}

interface SanitizedOptions {
  bingenPath: string;
  outputDir: string;
  inputFiles: string[];
  bingenArgs: string[];
  trpEndpoint: string;
  trpHeaders: Record<string, string>;
  envArgs: Record<string, string>;
}

/**
 * Spread input files into absolute paths
 * @param inputFiles - Input files or glob patterns
 * @returns Absolute paths to the input files
 */
function spreadInputFiles(inputFiles: string[]) {
  const projectRoot = process.cwd();
  return inputFiles
    .map((pattern) => path.resolve(projectRoot, pattern))
    .flatMap((pattern) => globSync(pattern));
}

/**
 * Ensure the output directory exists
 * @param options - Plugin options
 */
function ensureOutputDir(outputDir: string | undefined): string {
  const outputDirPath = path.resolve(
    process.cwd(),
    outputDir || "node_modules/.tx3"
  );

  fs.mkdirSync(outputDirPath, { recursive: true });

  return outputDirPath;
}

function generateBindings(options: SanitizedOptions) {
  const {
    bingenPath,
    outputDir,
    inputFiles,
    bingenArgs,
    trpEndpoint,
    trpHeaders,
    envArgs,
  } = options;

  const trpHeadersAsCmd: string[] = Object.entries(trpHeaders).map(
    ([key, value]) => `--trp-header ${key}=${value}`
  );

  const envArgsAsCmd: string[] = Object.entries(envArgs).map(
    ([key, value]) => `--env-arg ${key}=${value}`
  );

  const command = [
    bingenPath,
    "-i",
    ...inputFiles,
    `-o ${outputDir}`,
    "-t typescript",
    `--trp-endpoint ${trpEndpoint}`,
    ...trpHeadersAsCmd,
    ...envArgsAsCmd,
    ...bingenArgs,
  ].join(" ");

  try {
    execSync(command, { stdio: "inherit" });
  } catch (error) {
    console.error("Failed to generate TX3 bindings:", error);
    throw error;
  }
}

function sanitizeOptions(options: Tx3PluginOptions): SanitizedOptions {
  const {
    inputFiles,
    outputDir,
    bingenPath,
    bingenArgs,
    trpEndpoint,
    trpHeaders,
    envArgs,
  } = options;

  return {
    inputFiles: spreadInputFiles(inputFiles),
    outputDir: ensureOutputDir(outputDir),
    bingenPath: bingenPath || "tx3-bindgen",
    bingenArgs: bingenArgs || [],
    trpEndpoint: trpEndpoint || "http://localhost:3000",
    trpHeaders: trpHeaders || {},
    envArgs: envArgs || {},
  };
}

type Tx3Plugin = Plugin & {
  regenerateBindings: () => void;
  filesToWatch: () => string[];
};

export default function tx3RollupPlugin(options: Tx3PluginOptions): Tx3Plugin {
  const sanitizedOptions = sanitizeOptions(options);

  const plugin: Tx3Plugin = {
    name: "rollup-plugin-tx3",

    buildStart: () => generateBindings(sanitizedOptions),

    // Ensure bindings exist for production builds
    buildEnd() {
      const files = globSync(`${sanitizedOptions.outputDir}/**/*.ts`);

      if (files.length === 0) {
        generateBindings(sanitizedOptions);
      }
    },

    // Expose regenerateBindings for the Vite plugin
    regenerateBindings: () => generateBindings(sanitizedOptions),

    filesToWatch: () => spreadInputFiles(sanitizedOptions.inputFiles),
  };

  return plugin;
}
