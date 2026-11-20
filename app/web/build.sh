#!/bin/sh
# Builds the web version of the app into target/web/. Serve that folder over HTTP, e.g.
#   python3 -m http.server -d target/web 8080
set -eu
cd "$(dirname "$0")/.."

cargo build --release --locked --target wasm32-unknown-unknown

out=target/web
rm -rf "$out"
mkdir -p "$out"
cp web/index.html web/pca.js web/manifest.webmanifest "$out/"
cp -R web/icons "$out/"
# gl.js must come from the same miniquad as the build (the vendored copy).
cp vendor/miniquad/js/gl.js "$out/"
cp target/wasm32-unknown-unknown/release/painting_color_assistant.wasm "$out/"

# The service worker's cache is named after a hash of everything it keeps, so a changed
# build is picked up by installed copies of the app.
build=$(cd "$out" && find . -type f | LC_ALL=C sort | xargs cat | { sha256sum 2>/dev/null || shasum -a 256; } | cut -c1-12)
sed "s/@BUILD@/$build/" web/sw.js > "$out/sw.js"
echo "Web build in $(pwd)/$out"
