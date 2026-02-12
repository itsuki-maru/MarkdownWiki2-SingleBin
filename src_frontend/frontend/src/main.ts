import { createApp } from 'vue';
import { createPinia } from 'pinia';
import './style.css';
import App from './App.vue';
import router from './router';
import { assetsUrl } from '@/setting';
import './assets/github.css';
import { useApplicationInitStore } from './stores/appInits';
import { useImageStore } from './stores/images';
import { useEditRequestWikiStore } from './stores/editWikis';
import { useWikiStore } from './stores/wikis';

const app = createApp(App);
const pinia = createPinia();
app.use(pinia);
app.use(router);

if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker
      .register(`${assetsUrl}service-worker.js`)
      .then((registration) => {
        console.log('Service Worker registered: ', registration);
      })
      .catch((error) => {
        console.log('Service Worker registration failed:', error);
      });
  });
}

// 初期情報データ取得（非同期）
useApplicationInitStore(pinia)
  .init()
  .finally(() => {
    const appInitStore = useApplicationInitStore();
    appInitStore.init();

    const wikiStore = useWikiStore();
    wikiStore.initList();

    const imagesStore = useImageStore();
    imagesStore.initList();

    const editRequestWikiStore = useEditRequestWikiStore();
    editRequestWikiStore.initList();

    app.mount('#app');
  });
