import { createApp } from 'vue'
import { createPinia } from 'pinia'
import "./style.css"
import App from './AdminApp.vue'
import router from './router'

import './assets/github.css'

const app = createApp(App)

app.use(createPinia())
app.use(router)

app.mount('#app')
