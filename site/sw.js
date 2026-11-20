// Until November 2026 the web app lived at the root of this site, with a service worker
// here that kept its files. The app is now under app/; this replacement removes the old
// copy from browsers that installed it there and reloads their pages, which then show the
// website (with a link to the app). It does nothing else and unregisters itself.
"use strict";

self.addEventListener("install", () => self.skipWaiting());

self.addEventListener("activate", (event) => {
    event.waitUntil(
        (async () => {
            const keys = await caches.keys();
            // only the old app's cache ("pca-<hash>"); the app under app/ uses "pca-app-<hash>"
            await Promise.all(keys.filter((k) => /^pca-[0-9a-f]+$/.test(k)).map((k) => caches.delete(k)));
            await self.registration.unregister();
            const pages = await self.clients.matchAll({ type: "window" });
            pages.forEach((page) => page.navigate(page.url));
        })()
    );
});
