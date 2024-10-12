const {
  nodeModulesPolyfillPlugin,
} = require("esbuild-plugins-node-modules-polyfill");
const esbuild = require("esbuild");

// Client-side bundle
esbuild
  .build({
    entryPoints: ["src/client-entry.tsx"],
    bundle: true,
    outfile: "dist/bundle.js",
    minify: true,
    sourcemap: true,
    target: ["es2015"],
    define: { "process.env.NODE_ENV": '"development"' },
    format: "esm",
  })
  .catch(() => process.exit(1));

// Server-side bundle for V8 environment
esbuild
  .build({
    entryPoints: ["src/server-entry.tsx"],
    bundle: true,
    minify: false,
    outfile: "dist/ssr.js",
    format: "esm",
    target: ["es2020"],
    platform: "neutral",
    mainFields: ["module", "main"],
    define: {
      "process.env.NODE_ENV": '"development"',
      global: "globalThis",
      window: "globalThis",
      self: "globalThis",
      URL: "globalThis.URL",
    },
    plugins: [
      nodeModulesPolyfillPlugin({
        modules: ["url", "path", "stream", "util"],
      }),
    ],
    inject: ["./polyfills/URL.js"],
    external: [],
  })
  .catch(() => process.exit(1));
