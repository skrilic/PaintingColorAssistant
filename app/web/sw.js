// Service worker of the web app: keeps the app's files, so it starts without a network
// connection once it has been opened. web/build.sh replaces @BUILD@ with a hash of the
// files: a new build is a new cache, which the browser installs in the background and
// uses from the next start on.
"use strict";

const CACHE = "pca-app-@BUILD@";
const FILES = [
    "./",
    "index.html",
    "gl.js",
    "pca.js",
    "painting_color_assistant.wasm",
    "manifest.webmanifest",
    "icons/favicon-32.png",
    "icons/icon-192.png",
    "icons/icon-512.png",
    "icons/icon-maskable-512.png",
    "icons/apple-touch-icon.png",
];

self.addEventListener("install", (event) => {
    event.waitUntil(
        caches
            .open(CACHE)
            // "reload": fetch from the server, not from the browser's HTTP cache
            .then((cache) => cache.addAll(FILES.map((f) => new Request(f, { cache: "reload" }))))
            .then(() => self.skipWaiting())
    );
});

self.addEventListener("activate", (event) => {
    event.waitUntil(
        caches
            .keys()
            // older builds, and the cache of the app's first home at the site root ("pca-<hash>")
            .then((keys) => Promise.all(keys.filter((k) => k.startsWith("pca-") && k !== CACHE).map((k) => caches.delete(k))))
            .then(() => self.clients.claim())
    );
});

self.addEventListener("fetch", (event) => {
    const request = event.request;
    if (request.method !== "GET" || new URL(request.url).origin !== self.location.origin) {
        return;
    }
    event.respondWith(caches.match(request, { ignoreSearch: true }).then((hit) => hit || fetch(request)));
});
