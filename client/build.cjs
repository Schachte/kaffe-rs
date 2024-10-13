const {
  nodeModulesPolyfillPlugin,
} = require("esbuild-plugins-node-modules-polyfill");
const esbuild = require("esbuild");
const fs = require("fs");
const path = require("path");

// Function to copy template.html to dist directory
function copyTemplateHtml() {
  const sourceFile = path.join(__dirname, "template.html");
  const destinationFile = path.join(__dirname, "dist", "template.html");

  fs.copyFile(sourceFile, destinationFile, (err) => {
    if (err) {
      console.error("Error copying template.html:", err);
    } else {
      console.log("template.html copied to dist directory successfully");
    }
  });
}

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
  .then(() => {
    console.log("Client bundle built successfully");
    copyTemplateHtml(); // Copy template.html after client bundle is built
  })
  .catch(() => process.exit(1));

// Server-side bundle for V8 environment
const watch = process.argv.includes("--watch");
const buildOptions = {
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
};

if (watch) {
  esbuild.context(buildOptions).then((context) => {
    context.watch();
    console.log("Watching for changes...");
    copyTemplateHtml(); // Copy template.html when watch mode starts
  });
} else {
  esbuild
    .build(buildOptions)
    .then(() => {
      console.log("Server bundle built successfully");
      copyTemplateHtml(); // Copy template.html after server bundle is built
    })
    .catch(() => process.exit(1));
}
