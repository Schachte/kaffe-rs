const esbuild = require("esbuild");

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
    minify: false,
    platform: "node",
    target: ["node20"],
    define: { "process.env.NODE_ENV": '"production"' },
    format: "cjs",
    external: ["stream", "path", "fs", "crypto"],
  })
  .catch(() => process.exit(1));
