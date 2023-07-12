"use strict";

/**
 * Class to handle service worker caching and request serving.
 * @param {string} offlinePage - The URL of the page to show when offline.
 * @param {boolean} debugMode - If true, debug information will be logged to the console.
 */
class ServiceWorkerManager {
    constructor(offlinePage, debugMode) {
        /**
         * Cache version used for cache management.
         */
        this.CACHE_VERSION = 'v1';

        /**
         * List of cache keys this service worker cares about. Used to keep track and delete old caches.
         */
        this.CACHE_KEYS = [this.CACHE_VERSION];

        this.offlinePage = offlinePage;
        this.debugMode = debugMode;

        this.init();
    }

    /**
     * Initialize the service worker by setting up event listeners for install, activate, and fetch events.
     */
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
     * @param {InstallEvent} event The install event.
     */
    onInstall(event) {
        this.debug("Installing Service Worker");
        event.waitUntil(
            this.cacheOfflinePage().then(() => self.skipWaiting()).catch((error) => {
                this.debug(`Install event error: ${error}`);
            })
        );
    }

    /**
     * Event handler for the activate event.
     * Clears old caches and takes control of all clients.
     * @param {ExtendableEvent} event The activate event.
     */
    onActivate(event) {
        this.debug("Activating Service Worker");
        event.waitUntil(
            this.clearOldCaches().then(() => self.clients.claim()).catch((error) => {
                this.debug(`Activate event error: ${error}`);
            })
        );
    }

    /**
     * Event handler for the fetch event.
     * Serves requests from the cache if possible, otherwise fetches from the network.
     * Caches successful network responses.
     * @param {FetchEvent} event The fetch event.
     */
    onFetch(event) {
        const request = event.request;
        if (request.url.startsWith(self.location.origin) && request.url.startsWith("http") && request.method === "GET") {
            event.respondWith(
                caches.match(request).then(cachedResponse => {
                    if (cachedResponse) {
                        this.debug(`Fetching ${request.url}`);
                        this.debug(`Found in cache: ${request.url}`);
                        return cachedResponse;
                    }
                    
                    this.debug(`Going to network: ${request.url}`);
                    return fetch(request).then(response => {
                        if (response && response.ok) {
                            this.debug(`Saving in cache: ${request.url}`);
                            return caches.open(this.CACHE_VERSION).then(cache => {
                                cache.put(request, response.clone());
                                return response;
                            });
                        }
                        return response;
                    }).catch(error => this.handleFetchError(request, error));
                })
            );
        } else {
            this.debug("Ignoring non-GET request");
        }
    }

    /**
     * Handle fetch errors, such as failing to retrieve a resource from the cache or network.
     * For navigation requests, an offline page is shown.
     * For image requests, an offline image is shown.
     * @param {Request} request The failed request.
     * @param {Error} error The error that caused the fetch to fail.
     */
    handleFetchError(request, error) {
        this.debug(`Offline and no cache for: ${request.url}: ${error}`);
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
    }

    /**
     * Caches the offline page.
     * @returns {Promise} A promise that resolves once the offline page is cached.
     */
    cacheOfflinePage() {
        this.debug("Cache offline page");
        return caches.open(this.CACHE_VERSION).then(cache =>
            cache.addAll([this.offlinePage])
        );
    }

    /**
     * Deletes all caches that do not match the current cache version.
     * @returns {Promise} A promise that resolves once old caches are deleted.
     */
    clearOldCaches() {
        this.debug("Clean old caches");
        return caches.keys().then(keys =>
            Promise.all(keys.filter(key => !this.CACHE_KEYS.includes(key)).map(key => caches.delete(key)))
        );
    }
}

new ServiceWorkerManager("/offline/index.html", false);
