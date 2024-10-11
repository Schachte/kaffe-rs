const esbuild = require("esbuild");
const path = require("path");

// Client-side bundle
esbuild
  .build({
    entryPoints: ["src/index.tsx"],
    bundle: true,
    outfile: "dist/bundle.js",
    minify: true,
    sourcemap: true,
    target: ["es2015"],
    define: { "process.env.NODE_ENV": '"production"' },
    format: "esm",
  })
  .catch(() => process.exit(1));

esbuild
  .build({
    entryPoints: ["src/ssr.tsx"],
    bundle: true,
    outfile: "dist/ssr.js",
    format: "esm",
    platform: "node",
    target: ["es2020"],
    define: {
      "process.env.NODE_ENV": '"production"',
      global: "globalThis",
    },
    external: ["react", "react-dom"],
  })
  .catch(() => process.exit(1));
