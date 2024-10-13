#!/bin/bash

run_esbuild_and_copy_template() {
    (
        cd client
        yarn build:watch &
        
        cp template.html dist/template.html
        
        while inotifywait -e modify template.html; do
            cp template.html dist/template.html
            echo "HTML template updated"
        done
    ) &
}

run_esbuild_and_copy_template

cargo watch -x "run -- \
    --client-build-dir ./client/dist \
    --client-bundle-path bundle.js \
    --server-bundle-path ./client/dist/ssr.js \
    --html-template-path ./client/dist/template.html \
    --server-port 8080"

# Kill background processes when the script exits
trap "kill 0" EXIT