"use strict";

function clearOldCaches() {
    return debug("Clean old caches"), caches.keys().then(e => Promise.all(e.filter(e => 0 !== e.indexOf(offlineCache)).map(e => caches.delete(e))))
}

function cacheOfflinePage() {
    return debug("Cache offline page"), caches.open(offlineCache).then(e => e.addAll([offlinePage])).catch(e => {
        debug(e)
    })
}

function debug(e) {
    debugMode && console.log(e)
}
const cacheVersion = Date.now(),
    offlineCache = "offline-" + cacheVersion,
    offlinePage = "/offline/index.html",
    debugMode = !1;

self.addEventListener("install", e => {
    debug("Installing Service Worker"), e.waitUntil(cacheOfflinePage().then(() => self.skipWaiting()))
})

self.addEventListener("activate", e => {
    debug("Activating Service Worker"), e.waitUntil(clearOldCaches().then(() => self.clients.claim()))
})

self.addEventListener("fetch", e => {
    const n = e.request;
    if (n.url.startsWith(self.location.origin) && n.url.startsWith("http")) { // Exclude chrome-extension requests
        if (n.method === "GET") {
            e.respondWith(
                caches.match(n).then(cachedResponse => {
                    if (cachedResponse) {
                        debug("Fetching " + n.url);
                        debug("Found in cache: " + n.url);
                        return cachedResponse;
                    }

                    debug("Going to network: " + n.url);
                    return fetch(n).then(response => {
                        if (response && response.ok) {
                            debug("Saving in cache: " + n.url);
                            const clonedResponse = response.clone();
                            return caches.open(offlineCache).then(cache => {
                                cache.put(n, clonedResponse);
                                return response;
                            });
                        }
                        return response;
                    }).catch(error => {
                        debug("Offline and no cache for: " + n.url + ": " + error);
                        if (n.mode === "navigate") {
                            debug("Showing offline page");
                            return caches.match(offlinePage);
                        } else if (n.headers.get("Accept").includes("image")) {
                            return new Response(
                                '<svg role="img" aria-labelledby="offline-title" viewBox="0 0 100 50" xmlns="http://www.w3.org/2000/svg"><text x="50" y="25" font-family="monospace" font-size="12" text-anchor="middle" dominant-baseline="middle">Offline</text></svg>',
                                {
                                    headers: {
                                        "Content-Type": "image/svg+xml"
                                    }
                                }
                            );
                        }
                    });
                })
            );
        } else {
            debug("Ignoring non-GET request");
        }
    }
});

