import type { Plugin } from "vite";
import path from "path";
import tx3RollupPlugin from "./tx3-rollup-plugin";
import type { Tx3PluginOptions } from "./tx3-rollup-plugin";

export type { Tx3PluginOptions };

export default function tx3VitePlugin(options: Tx3PluginOptions): Plugin {
  const rollupPlugin = tx3RollupPlugin(options);

  return {
    ...rollupPlugin,

    config(config) {
      return {
        resolve: {
          alias: {
            "@tx3": path.resolve(process.cwd(), "node_modules/.tx3"),
            ...(config.resolve?.alias || {}),
          },
        },
      };
    },

    // Add Vite-specific configuration
    configureServer(server) {
      const projectRoot = process.cwd();
      const filesToWatch = options.inputFiles.map((pattern) =>
        path.resolve(projectRoot, pattern)
      );

      server.watcher.add(filesToWatch);

      server.watcher.on("change", (changedFile) => {
        const absoluteChangedFile = path.resolve(projectRoot, changedFile);

        if (
          filesToWatch.some((pattern) => absoluteChangedFile.includes(pattern))
        ) {
          console.log("TX3 file changed, regenerating bindings...");
          rollupPlugin.regenerateBindings();

          // Vite-specific HMR handling
          server.moduleGraph.invalidateAll();
          server.ws.send({ type: "full-reload" });
        }
      });
    },
  };
}
