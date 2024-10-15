const {
  nodeModulesPolyfillPlugin,
} = require("esbuild-plugins-node-modules-polyfill");
const esbuild = require("esbuild");
const path = require("path");
const cssPlugin = require("esbuild-css-modules-plugin");

const outputName = process.argv[2] || "bundle";

// Client-side bundle
esbuild
  .build({
    entryPoints: ["client/dist/client-entry.tsx"],
    bundle: true,
    outfile: `client/dist/${outputName}`,
    minify: true,
    sourcemap: false,
    target: ["es2015"],
    define: { "process.env.NODE_ENV": '"development"' },
    format: "esm",
    plugins: [
      cssPlugin({
        force: true,
        emitDeclarationFile: true,
        localsConvention: "camelCaseOnly",
        inject: false,
      }),
    ],
  })
  .catch(() => process.exit(1));

// Server-side bundle for V8 environment
const watch = process.argv.includes("--watch");
const buildOptions = {
  entryPoints: ["client/dist/server-entry.tsx"],
  bundle: true,
  minify: false,
  outfile: `client/dist/ssr-${outputName}`,
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
      console.log(`Server bundle (${outputName}-ssr.js) built successfully`);
    })
    .catch((e) => console.log(e));
}
