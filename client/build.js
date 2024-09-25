// const esbuild = require("esbuild");
import esbuild from "esbuild";

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

// Server-side bundle
esbuild
  .build({
    entryPoints: ["src/ssr.tsx"],
    bundle: true,
    outfile: "dist/ssr.js",
    minify: true,
    platform: "node",
    target: ["es2015"],
    define: { "process.env.NODE_ENV": '"production"' },
    format: "esm",
    external: ["stream"], // Add this line to exclude the "stream" module from bundling
  })
  .catch(() => process.exit(1));
