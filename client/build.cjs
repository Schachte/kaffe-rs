const {
  nodeModulesPolyfillPlugin,
} = require("esbuild-plugins-node-modules-polyfill");
const esbuild = require("esbuild");

// Client-side bundle
esbuild
  .build({
    entryPoints: ["client/dist/client-entry.tsx"],
    bundle: true,
    outfile: "client/dist/bundle.js",
    minify: true,
    sourcemap: true,
    target: ["es2015"],
    define: { "process.env.NODE_ENV": '"development"' },
    format: "esm",
  })
  .catch(() => process.exit(1));

// Server-side bundle for V8 environment
const watch = process.argv.includes("--watch");
const buildOptions = {
  entryPoints: ["client/dist/server-entry.tsx"],
  bundle: true,
  minify: false,
  outfile: "client/dist/ssr.js",
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
  inject: ["client/polyfills/URL.js"],
  external: [],
};

if (watch) {
  esbuild.context(buildOptions).then((context) => {
    context.watch();
    console.log("Watching for changes...");
  });
} else {
  esbuild
    .build(buildOptions)
    .then(() => {
      console.log("Server bundle built successfully");
    })
    .catch((e) => console.log(e));
}
