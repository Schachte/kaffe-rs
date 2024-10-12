#!/bin/bash

# Function to run esbuild in watch mode
run_esbuild() {
    (cd client && yarn build:watch) &
}

run_esbuild

# Run the Rust application with cargo watch
cargo watch -x "run -- \
    --client-build-dir ./client/dist \
    --client-bundle-path ./client/dist/bundle.js \
    --server-bundle-path ./client/dist/ssr.js \
    --server-port 8080"

# Kill background processes when the script exits
trap "kill 0" EXIT