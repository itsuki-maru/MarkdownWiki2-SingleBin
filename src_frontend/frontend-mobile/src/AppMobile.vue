<script setup lang="ts">
import { reactive, provide, ref, onMounted, watch } from "vue";
import type { LoginUser } from "./interface";
import { useRouter } from "vue-router";
import { assetsUrl } from "@/setting";
import axios from "axios";
import { getAppTitleUrl } from "./router/urls";

const appTitle = ref("")
const getAppTitle = async (): Promise<void> => {
  try {
    const response = await axios.get(getAppTitleUrl);
    appTitle.value = response.data["app_name"];
  } catch (error) {
    console.log("Error.");
  }
}
getAppTitle();

// Login User Status Provide.
const loginUser: LoginUser = {
  isAuthenticated: false,
}
provide("loginUser", reactive(loginUser));

// メモアイコンの表示非表示管理
const isShowMemoIcon = ref(true);
// 他の子コンポーネントで表示・非表示を切り替えられるようにprovide
provide("isShowMemoIcon", isShowMemoIcon);

// List.vueへリダイレクト
const router = useRouter();
const listRedirect = (): void => {
  router.push("/wiki/list");
}

listRedirect();

const showSplashScreen = ref(true);
// スプラッシュスクリーンを3秒後に非表示にする
onMounted(() => {
  setTimeout(() => {
    showSplashScreen.value = false;
  }, 1800);
});
</script>

<template>
  <div class="container">
    <div v-if="showSplashScreen" id="splash-screen">
      <img :src="`${assetsUrl}icon-512x512.png`" alt="App Logo" class="logo"/>
      <h1 id="splash-title">MarkdownWiki2</h1>
    </div>
    <header class="parent-header">
      <h1 class="app-header" id="application-title">{{ appTitle }}</h1>
    </header>
    <RouterView />
  </div>
</template>

<style>
html {
  scroll-behavior: smooth;
}

.app-header {
  color: #4183C4;
  font-size: 28px;
  text-shadow: 2px 1px 2px rgb(165, 165, 165);
  letter-spacing: 1px;
}


.app-header a {
  color: #4183C4;
  font-size: 28px;
}

a {
  text-decoration: none;
}

/* 目次のリンク（github.cssを上書き） */
.toc-content a {
  text-decoration: none;
  color: black;
}

/* ホバー */
.toc-content a:hover {
  background-color: #c1d1d6;
  display: inline-block;
  text-decoration: underline;
  /* transform を適用するために必要 */
}

.head1 {
  font-size: 22px;
  margin-bottom: -1%;
  border-bottom: solid 3px #d7d7d7;
}

.head2 {
  font-size: 18px;
  color: black;
  margin-top: 2%;
  padding: 0.1em 0.3em;
  background: #f4f4f4;
  border-left: solid 5px #daac9e;
  border-bottom: solid 3px #d7d7d7;
}

#splash-screen {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  background: #ffffff;
  color: white;
  text-align: center;
  z-index: 1000;
  z-index: 10;
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
}

#splash-title {
  color: #4183C4;
  font-size: 28px;
  text-shadow: 2px 1px 2px rgb(165, 165, 165);
  letter-spacing: 1px;
  letter-spacing: 1px;
  text-align: center;
  letter-spacing: 2px;
  margin: 0;
  padding: 20px 10px;
  animation: fade-in 1.5s ease-in-out;
}

.logo {
  width: 100px;
  height: auto;
  animation: fade-in 1.5s ease-in-out;
  z-index: 1000;
  border: none;
  box-shadow: none;
}

.message {
  margin-top: 20px;
  font-size: 18px;
  animation: fade-in 2s ease-in-out;
}

@keyframes fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.parent-header {
  border-bottom: solid 1px gray;
  display: flex;
  justify-content: space-between;
  margin-bottom: 7px;
}
</style>