import { createRouter, createWebHistory } from "vue-router";
import { useWikiStore } from "@/stores/wikis";
import { useImageStore } from "@/stores/images";
import type { RouteRecordRaw } from "vue-router";
import { getUserUrl } from "@/router/urls";
import apiClient from "@/axiosClient";

const routeSettings: RouteRecordRaw[] = [
  {
    path: "/wiki/create",
    name: "Create",
    component: () => {
      return import("@/views/wiki/Create.vue");
    },
    beforeEnter: async (to, from, next) => {
      // 認証確認
      try {
        const res = await apiClient.get(getUserUrl);
        // 認証に成功したら画像一覧データを取得
        const imageStore = useImageStore();
        imageStore.initList();
        next();
      } catch (error) {
        // 失敗したらログイン画面に飛ばす
        next({ name: "login" });
      }
    }
  },
  {
    path: "/wiki/list",
    name: "List",
    component: () => {
      return import("@/views/wiki/List.vue");
    },
    beforeEnter: async (to, from, next) => {
      // 認証確認
      try {
        const res = await apiClient.get(getUserUrl);
        // 正常処理
        next();
      } catch (error) {
        // 失敗したらログイン画面に飛ばす
        next({ name: "login" });
      }
    }
  },
  {
    path: "/wiki/preview/:id",
    name: "Preview",
    component: () => {
      return import("@/views/wiki/Preview.vue");
    },
    props: (routes) => {
      return {
        id: routes.params.id
      };
    }
  },
  {
    path: "/wiki/update/:id",
    name: "Update",
    component: () => {
      return import("@/views/wiki/Update.vue");
    },
    props: (routes) => {
      return {
        id: routes.params.id
      };
    }
  },
  {
    path: "/wiki/delete/:id",
    name: "Delete",
    component: () => {
      return import("@/views/wiki/Delete.vue");
    },
    props: (routes) => {
      return {
        id: routes.params.id
      };
    }
  },
  {
    path: "/account/login",
    name: "login",
    component: () => {
      return import("@/views/auth/LoginView.vue");
    },
    beforeEnter: (to, from, next) => {
      // ログイン画面遷移時に情報を残さないためにwiki情報を初期化
      const wikiStore = useWikiStore();
      wikiStore.clearWiki();
      next()
    }
  },
  {
    path: "/account/create",
    name: "signup",
    component: () => {
      return import("@/views/auth/SignupView.vue");
    }
  }
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: routeSettings,
})

export default router
