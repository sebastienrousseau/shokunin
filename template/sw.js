"use strict";

/**
 * Class to handle service worker caching and request serving.
 */
class ServiceWorkerManager {
    constructor(offlinePage, debugMode) {
        this.cacheVersion = Date.now();
        this.offlineCache = "offline-" + this.cacheVersion;
        this.offlinePage = offlinePage;
        this.debugMode = debugMode;

        this.init();
    }

    init() {
        self.addEventListener("install", this.onInstall.bind(this));
        self.addEventListener("activate", this.onActivate.bind(this));
        self.addEventListener("fetch", this.onFetch.bind(this));
    }

    /**
     * Logs a message to the console if debug mode is enabled.
     * @param {any} message The message to log.
     */
    debug(message) {
        if (this.debugMode) {
            console.log(message);
        }
    }

    /**
     * Event handler for the install event.
     * Caches the offline page and forces the waiting service worker to become the active service worker.
     */
    onInstall(event) {
        this.debug("Installing Service Worker");
        event.waitUntil(this.cacheOfflinePage().then(() => self.skipWaiting()));
    }

    /**
     * Event handler for the activate event.
     * Clears old caches and takes control of all clients.
     */
    onActivate(event) {
        this.debug("Activating Service Worker");
        event.waitUntil(this.clearOldCaches().then(() => self.clients.claim()));
    }

    /**
     * Event handler for the fetch event.
     * Serves requests from the cache if possible, otherwise fetches from the network.
     * Caches successful network responses.
     */
    onFetch(event) {
        const request = event.request;
        if (request.url.startsWith(self.location.origin) && request.url.startsWith("http")) {
            if (request.method === "GET") {
                event.respondWith(
                    // Fetch response from cache or network.
                    caches.match(request).then(cachedResponse => {
                        // Return cached response if available.
                        if (cachedResponse) {
                            this.debug(`Fetching ${request.url}`);
                            this.debug(`Found in cache: ${request.url}`);
                            return cachedResponse;
                        }

                        this.debug(`Going to network: ${request.url}`);
                        return fetch(request).then(response => {
                            // Cache response if request was successful.
                            if (response && response.ok) {
                                this.debug(`Saving in cache: ${request.url}`);
                                const clonedResponse = response.clone();
                                return caches.open(this.offlineCache).then(cache => {
                                    cache.put(request, clonedResponse);
                                    return response;
                                });
                            }

                            return response;
                        }).catch(error => {
                            this.debug(`Offline and no cache for: ${request.url}: ${error}`);
                            // Return offline page or offline image if navigation or image request fails.
                            if (request.mode === "navigate") {
                                this.debug("Showing offline page");
                                return caches.match(this.offlinePage);
                            } else if (request.headers.get("Accept").includes("image")) {
                                return new Response('<svg role="img" aria-labelledby="offline-title" viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg"><title id="offline-title">Offline</title><g fill="none" fill-rule="evenodd"><path fill="#D8D8D8" d="M0 0h400v300H0z"/><text fill="#9B9B9B" font-family="Helvetica Neue,Arial,Helvetica,sans-serif" font-size="72" font-weight="bold"><tspan x="93" y="172">offline</tspan></text></g></svg>', {
                                    headers: {
                                        "Content-Type": "image/svg+xml"
                                    }
                                });
                            }
                        });
                    })
                );
            } else {
                this.debug("Ignoring non-GET request");
            }
        }
    }

    /**
     * Caches the offline page.
     * @returns {Promise} A promise that resolves once the offline page is cached.
     */
    cacheOfflinePage() {
        this.debug("Cache offline page");
        return caches.open(this.offlineCache).then(cache =>
            cache.addAll([this.offlinePage])
        ).catch(this.debug);
    }

    /**
     * Deletes all caches that do not match the current offline cache.
     * @returns {Promise} A promise that resolves once old caches are deleted.
     */
    clearOldCaches() {
        this.debug("Clean old caches");
        return caches.keys().then(keys =>
            Promise.all(keys.filter(key => key !== this.offlineCache).map(key => caches.delete(key)))
        );
    }
}

new ServiceWorkerManager("/offline/index.html", false);
