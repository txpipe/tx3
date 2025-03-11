import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { rollup } from "rollup";
import path from "path";
import fs from "fs";
import tx3RollupPlugin from "../src";

describe("rollup-plugin-tx3", () => {
  const outputDir = "test/.tx3-output";
  const fixturesDir = path.join(__dirname, "fixtures");

  beforeEach(() => {
    // Clean output directory before each test
    if (fs.existsSync(outputDir)) {
      fs.rmSync(outputDir, { recursive: true });
    }
  });

  afterEach(() => {
    // Clean up after tests
    if (fs.existsSync(outputDir)) {
      fs.rmSync(outputDir, { recursive: true });
    }
  });

  it("should generate TypeScript bindings from tx3 files", async () => {
    const bundle = await rollup({
      input: "test/fixtures/dummy.js", // We need a dummy entry point
      plugins: [
        tx3RollupPlugin({
          inputFiles: ["test/fixtures/*.tx3"],
          outputDir,
          bingenPath: "tx3-bindgen", // Make sure tx3-bindgen is in your PATH
        }),
      ],
    });

    // Trigger the buildStart hook
    await bundle.generate({ format: "esm" });

    // Check if bindings were generated
    const outputFiles = fs.readdirSync(outputDir);
    expect(outputFiles.length).toBeGreaterThan(0);
    expect(outputFiles.some((file) => file.endsWith(".ts"))).toBe(true);

    // Clean up
    await bundle.close();
  });

  it("should handle plugin options correctly", () => {
    const plugin = tx3RollupPlugin({
      inputFiles: ["test/fixtures/*.tx3"],
      outputDir: "custom-output",
      bingenPath: "/custom/path/tx3-bindgen",
      bingenArgs: ["--verbose"],
    });

    expect(plugin.name).toBe("rollup-plugin-tx3");
  });
});
