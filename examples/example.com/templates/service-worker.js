"use strict";

function clearOldCaches() {
    return debug("Clean old caches"), caches.keys().then(e => Promise.all(e.filter(e => 0 !== e.indexOf(offlineCache)).map(e => caches["delete"](e))))
}

function cacheOfflinePage() {
    return debug("Cache offline page"), caches.open(offlineCache).then(e => e.addAll([offlinePage]))["catch"](e => {
        debug(e)
    })
}

function debug(e) {
    debugMode && console.log(e)
}
const cacheVersion = 1623496433,
    offlineCache = "offline-" + cacheVersion,
    offlinePage = "/offline.html",
    debugMode = !1;
self.addEventListener("install", e => {
    debug("Installing Service Worker"), e.waitUntil(cacheOfflinePage().then(() => self.skipWaiting()))
}), self.addEventListener("activate", e => {
    debug("Activating Service Worker"), e.waitUntil(clearOldCaches().then(() => self.clients.claim()))
}), self.addEventListener("fetch", e => {
    let n = e.request;
    "GET" === n.method ? e.respondWith(caches.match(n).then(e => (debug("Fetching " + n.url), e && debug("Found in cache: " + n.url), e || fetch(n).then(e => {
        if (debug("Going to network: " + n.url), e && e.ok) {
            debug("Saving in cache: " + n.url);
            let t = e.clone();
            caches.open(offlineCache).then(e => e.put(n, t))
        }
        return e
    })["catch"](e => (debug("Offline and no cache for: " + n.url + ": " + e), "navigate" == n.mode ? (debug("Showing offline page"), caches.match(offlinePage)) : -1 !== n.headers.get("Accept").indexOf("image") ? new Response('<svg role="img" aria-labelledby="offline-title" viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg"><title id="offline-title">Offline</title><g fill="none" fill-rule="evenodd"><path fill="#D8D8D8" d="M0 0h400v300H0z"/><text fill="#9B9B9B" font-family="Helvetica Neue,Arial,Helvetica,sans-serif" font-size="72" font-weight="bold"><tspan x="93" y="172">offline</tspan></text></g></svg>', {
        headers: {
            "Content-Type": "image/svg+xml"
        }
    }) : void 0))))) : debug("Ignoring non GET request")
});