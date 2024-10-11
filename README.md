Still super beta, just experimenting with building a custom static site renderer for React while using Rust. Ideally this would serve as the core for a custom SSG framework.

To test:

- `cargo build` to grab the Rust deps
- `cd client && yarn && yarn build` to grab the clientside deps and output the IIFE bundle (built using esbuild)
- `cargo run` will kick off the server from which you can invoke something like `curl localhost:8080` to validate proper SSR rendering is taking place
