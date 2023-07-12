"use strict";

/**
 * Class to handle registration of a service worker.
 */
class ServiceWorkerSetup {
    /**
     * Constructor for the ServiceWorkerSetup class.
     * Checks if service workers are supported and initiates registration if they are.
     */
    constructor() {
        if ("serviceWorker" in navigator) {
            this.registerServiceWorker();
        } else {
            console.warn("Service workers are not supported by this browser");
        }
    }

    /**
     * Method to register a service worker.
     * Logs a success message with the registration scope if registration succeeds,
     * or an error message if registration fails.
     */
    registerServiceWorker() {
        navigator.serviceWorker.register("/sw.js")
            .then(registration => {
                console.log("ServiceWorker registration successful with scope: ", registration.scope);
            })
            .catch(error => {
                console.error("ServiceWorker registration failed: ", error);
            });
    }
}

// Create an instance of the ServiceWorkerSetup class and attach it to the global window object.
// This makes the instance accessible from anywhere in your code that has access to the global scope.
window.serviceWorkerSetup = new ServiceWorkerSetup();
