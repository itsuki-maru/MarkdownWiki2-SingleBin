import { createRouter, createWebHistory } from 'vue-router';
import { useWikiStore } from '@/stores/wikis';
import type { RouteRecordRaw } from 'vue-router';
import { getUserUrl } from '@/router/urls';
import apiClient from '@/axiosClient';

const routeSettings: RouteRecordRaw[] = [
  {
    path: '/wiki/create',
    name: 'Create',
    component: () => {
      return import('@/views/wiki/CreateMobile.vue');
    },
  },
  {
    path: '/wiki/list',
    name: 'List',
    component: () => {
      return import('@/views/wiki/ListMobile.vue');
    },
  },
  {
    path: '/wiki/preview/:id',
    name: 'Preview',
    component: () => {
      return import('@/views/wiki/PreviewMobile.vue');
    },
    props: (routes) => {
      return {
        id: routes.params.id,
      };
    },
  },
  {
    path: '/wiki/update/:id',
    name: 'Update',
    component: () => {
      return import('@/views/wiki/UpdateMobile.vue');
    },
    props: (routes) => {
      return {
        id: routes.params.id,
      };
    },
  },
  {
    path: '/wiki/delete/:id',
    name: 'Delete',
    component: () => {
      return import('@/views/wiki/DeleteMobile.vue');
    },
    props: (routes) => {
      return {
        id: routes.params.id,
      };
    },
  },
  {
    path: '/account/login',
    name: 'login',
    component: () => {
      return import('@/views/auth/LoginViewMobile.vue');
    },
    beforeEnter: (to, from, next) => {
      const wikiStore = useWikiStore();
      wikiStore.clearWiki();
      next();
    },
  },
  {
    path: '/account/create',
    name: 'signup',
    component: () => {
      return import('@/views/auth/SignupViewMobile.vue');
    },
  },
];

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: routeSettings,
});

export default router;
