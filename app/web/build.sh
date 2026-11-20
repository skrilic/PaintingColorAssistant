#!/bin/sh
# Builds the web version of the app into target/web/. Serve that folder over HTTP, e.g.
#   python3 -m http.server -d target/web 8080
set -eu
cd "$(dirname "$0")/.."

cargo build --release --target wasm32-unknown-unknown

out=target/web
rm -rf "$out"
mkdir -p "$out"
cp web/index.html web/pca.js "$out/"
# gl.js must come from the same miniquad as the build (the vendored copy).
cp vendor/miniquad/js/gl.js "$out/"
cp assets/icon-64.png "$out/"
cp target/wasm32-unknown-unknown/release/painting_color_assistant.wasm "$out/"
echo "Web build in $(pwd)/$out"
