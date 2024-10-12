 <center>Experimental POC for doing server side rendering with React in Rust using the V8 core implementation from Deno</center>

---

### _Motivation:_

The goal is to build out the SSR (server side rendering) component of a larger SSG (static site generator) tool for quickly turning Markdown files which contain React components into a hosted site.

### _Current progress:_

- Experimenting with V8 lib from `deno_core` for compiling and running JS bundles in Rust
- Got a rudimentary SSR setup running while also supporting clientside hydration

### _Thoughts I'm tinkering with:_

- Reusable lib for easy self-hosting
- Hot module reloading setup
- Custom Markdown AST generation that supports embedding React components (similar to ReactMDX)

---

### _Basic setup:_

- `cargo build` to grab the Rust deps
- `cd client && yarn && yarn build` to grab the clientside deps and output the IIFE bundle (built using esbuild)
- `cargo run` will kick off the server from which you can invoke something like `curl localhost:8080` to validate proper SSR rendering is taking place
