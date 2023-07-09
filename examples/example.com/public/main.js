class ServiceWorkerSetup {
    constructor() {
        this.init();
    }
    init() {
        "serviceWorker" in navigator && navigator.serviceWorker.register("/sw.js").then(function (e) {
            console.log("ServiceWorker registration successful with scope: ", e.scope);
        })["catch"](function (e) {
            console.log("ServiceWorker registration failed: ", e);
        });
    }
}
window.ServiceWorker = new ServiceWorkerSetup;