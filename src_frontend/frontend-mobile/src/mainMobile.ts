import { createApp } from 'vue'
import { createPinia } from 'pinia'

import App from './AppMobile.vue'
import router from './router/indexMobile'
import { assetsUrl } from "@/setting";
import "./style.css"

import './assets/github.css'

const app = createApp(App)

app.use(createPinia())
app.use(router)

app.mount('#app')

if ("serviceWorker" in navigator) {
    window.addEventListener("load", () => {
        navigator.serviceWorker
            .register(`${assetsUrl}service-worker.js`)
            .then(registration => {
                console.log("Service Worker registered: ", registration);
            })
            .catch(error => {
                console.log("Service Worker registration failed:", error);
            });
    });
}
