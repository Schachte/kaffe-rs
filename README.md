## Background

Experimental POC for doing server side rendering with React in Rust using the V8 core implementation from Deno

### _Motivation:_

The goal is to build out the SSR (server side rendering) component of a larger SSG (static site generator) tool for quickly turning Markdown files which contain React components into a hosted site.

### _Current progress:_

- Experimenting with V8 lib from `deno_core` for compiling and running JS bundles in Rust
- Got a rudimentary SSR setup running while also supporting clientside hydration

### _Thoughts I'm tinkering with:_

- Reusable lib for easy self-hosting
- Hot module reloading setup
- Custom Markdown AST generation that supports embedding React components (similar to ReactMDX)

## Getting started:

1. Initial step is to get some client program setup. I've added a demo inside of `client` which is a basic React site with bundle generation being done via `esbuild`.
   - Run: `yarn && yarn build`
   - Bundles will exist in `client/dist/`
2. Two bundles will get generated, one for the server and one for the client
   - `bundle.js`
   - `ssr.js`
3. Generate a release binary for `Kaffe` by running `cargo build --release`

You can now run the SSR server with Kaffe like so:

```bash
./target/release/ssr \
  --client-build-dir ./client/dist \
  --client-bundle-path ./client/dist/bundle.js \
  --server-bundle-path ./client/dist/ssr.js \
  --server-port 8080
```

```
Starting Kaffe Server...
=========================
Port: 8080
Client Build Dir: ./client/dist
Client Bundle: ./client/dist/bundle.js
Server Bundle: ./client/dist/ssr.js
=========================

Server is running!
Local: http://localhost:8080
Network: http://127.0.0.1:8080

Press Ctrl+C to stop the server
```

### Automatic compilation

I've added a poor mans version of HMR (hot module reloading), which you can see inside `./dev.sh`. This will auto recompile any of the clientside or backend Rust bundles/binaries when developing.
