function ServiceWorkerSetup() {
    this.init()
}
ServiceWorkerSetup.prototype.init = function () {
    "serviceWorker" in navigator && navigator.serviceWorker.register("/service-worker.js").then(function (e) {
        console.log("ServiceWorker registration successful with scope: ", e.scope)
    })["catch"](function (e) {
        console.log("ServiceWorker registration failed: ", e)
    })
},
    window.ServiceWorkerSetup = new ServiceWorkerSetup;
