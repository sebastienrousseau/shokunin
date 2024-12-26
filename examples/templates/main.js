"use strict";

/**
 * Class to handle registration of a service worker.
 */
class ServiceWorkerSetup {
    /**
     * Constructor for the ServiceWorkerSetup class.
     * Checks if service workers are supported and initiates registration if they are.
     * If not, logs a warning to the console.
     */
    constructor() {
        if ("serviceWorker" in navigator) {
            // Deferring service worker registration until after the page has loaded.
            window.addEventListener('load', () => {
                this.registerServiceWorker();
            });
        } else {
            console.warn("Service workers are not supported by this browser");
        }
    }

    /**
     * Method to register a service worker.
     * Logs a success message with the registration scope if registration succeeds,
     * or an error message if registration fails.
     * Also checks for a new service worker installation and triggers an update if found.
     */
    registerServiceWorker() {
        navigator.serviceWorker.register("/sw.js", {scope: './'})
            .then(registration => {
                console.log("ServiceWorker registration successful with scope: ", registration.scope);

                // If there's no controller, this page wasn't loaded via a service worker, so they're looking at the latest version.
                // Exit early
                if (!navigator.serviceWorker.controller) return;

                // If there's a worker waiting, that means a new version has been found and the waiting worker can be updated
                if (registration.waiting) {
                    this.updateServiceWorker(registration.waiting);
                    return;
                }

                // If there's a worker installing, track its progress. If it becomes "installed", we can update the service worker.
                if (registration.installing) {
                    this.trackInstallingWorker(registration.installing);
                    return;
                }

                // If none of the above, then listen for new installing workers arriving.
                // If one arrives, track its progress.
                // If it becomes "installed", our service worker code can be updated.
                registration.addEventListener('updatefound', () => {
                    this.trackInstallingWorker(registration.installing);
                });
            })
            .catch(error => {
                console.error("ServiceWorker registration failed: ", error);
            });

        // Ensure refresh is only called once.
        // This works around a bug in "force update on reload".
        let refreshing;
        navigator.serviceWorker.addEventListener('controllerchange', () => {
            if (refreshing) return;
            window.location.reload();
            refreshing = true;
        });
    }

    /**
     * Sends a 'skipWaiting' message to a service worker indicating that it should activate immediately.
     * @param {ServiceWorker} worker - The service worker that should be updated.
     */
    updateServiceWorker(worker) {
        worker.postMessage({action: 'skipWaiting'});
    }

    /**
     * Listens for a state change on a service worker. If the state becomes 'installed',
     * this means the service worker is ready to take over from the current one.
     * Call updateServiceWorker() to trigger the new service worker to become active immediately.
     * @param {ServiceWorker} worker - The service worker that is being installed.
     */
    trackInstallingWorker(worker) {
        worker.addEventListener('statechange', () => {
            if (worker.state === 'installed') {
                this.updateServiceWorker(worker);
            }
        });
    }
}

// Create an instance of the ServiceWorkerSetup class and attach it to the global window object.
// This makes the instance accessible from anywhere in your code that has access to the global scope.
window.serviceWorkerSetup = new ServiceWorkerSetup();
