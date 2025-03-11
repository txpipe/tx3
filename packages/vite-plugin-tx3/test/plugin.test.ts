import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { createServer, normalizePath } from "vite";
import path from "path";
import fs from "fs";
import tx3VitePlugin from "../src";

describe("vite-plugin-tx3", () => {
  const outputDir = "test/.tx3-output";
  const fixturesDir = path.join(__dirname, "fixtures");

  beforeEach(() => {
    // Clean output directory before each test
    if (fs.existsSync(outputDir)) {
      //fs.rmSync(outputDir, { recursive: true });
    }
  });

  afterEach(() => {
    // Clean up after tests
    if (fs.existsSync(outputDir)) {
      //fs.rmSync(outputDir, { recursive: true });
    }
  });

  it("should generate TypeScript bindings from tx3 files", async () => {
    const server = await createServer({
      configFile: false,
      plugins: [
        tx3VitePlugin({
          inputFiles: ["test/fixtures/*.tx3"],
          outputDir,
          bingenPath: "tx3-bindgen", // Make sure tx3-bindgen is in your PATH
        }),
      ],
    });

    await server.pluginContainer.buildStart({});

    // Check if bindings were generated
    const outputFiles = fs.readdirSync(outputDir);
    expect(outputFiles.length).toBeGreaterThan(0);
    expect(outputFiles.some((file) => file.endsWith(".ts"))).toBe(true);

    await server.close();
  });

  it("should handle plugin options correctly", () => {
    const plugin = tx3VitePlugin({
      inputFiles: ["test/fixtures/*.tx3"],
      outputDir: "custom-output",
      bingenPath: "/custom/path/tx3-bindgen",
      bingenArgs: ["--verbose"],
    });

    expect(plugin.name).toBe("vite-plugin-tx3");
  });

  it("should set up HMR correctly", async () => {
    const server = await createServer({
      configFile: false,
      plugins: [
        tx3VitePlugin({
          inputFiles: ["test/fixtures/*.tx3"],
          outputDir,
          bingenPath: "tx3-bindgen",
        }),
      ],
    });

    await server.waitForRequestsIdle();

    const watchedFolders = Object.keys(server.watcher.getWatched());

    expect(watchedFolders.some((file) => file.includes("test/fixtures"))).toBe(
      true
    );

    await server.close();
  });

  it("should configure alias resolution correctly", async () => {
    const server = await createServer({
      configFile: false,
      plugins: [
        tx3VitePlugin({
          inputFiles: ["test/fixtures/*.tx3"],
          outputDir,
          bingenPath: "tx3-bindgen",
        }),
      ],
    });

    // Get the resolved config
    const resolvedConfig = server.config;

    // Check that @tx3 alias is configured
    const tx3Alias = resolvedConfig.resolve.alias.find(
      (alias) => alias.find === "@tx3"
    );

    expect(tx3Alias).toBeDefined();
    expect(tx3Alias?.replacement).toBe(
      normalizePath(path.resolve(process.cwd(), "node_modules/.tx3"))
    );

    // Test actual resolution
    const result = await server.pluginContainer.resolveId("@tx3/sample");
    expect(result).not.toBeNull();
    expect(normalizePath(result?.id || "")).toContain(
      "node_modules/.tx3/sample"
    );

    await server.close();
  });

  it("should merge alias configuration with existing aliases", async () => {
    const server = await createServer({
      configFile: false,
      resolve: {
        alias: {
          "@existing": "/path/to/existing",
        },
      },
      plugins: [
        tx3VitePlugin({
          inputFiles: ["test/fixtures/*.tx3"],
          outputDir,
          bingenPath: "tx3-bindgen",
        }),
      ],
    });

    // Get the resolved config
    const resolvedConfig = server.config;

    // Check that both aliases exist
    const aliases = resolvedConfig.resolve.alias;

    const hasExistingAlias = aliases.some(
      (alias) => alias.find === "@existing"
    );
    const hasTx3Alias = aliases.some((alias) => alias.find === "@tx3");

    expect(hasExistingAlias).toBe(true);
    expect(hasTx3Alias).toBe(true);

    await server.close();
  });
});
