import { defineStore } from "pinia";

export const useAuthStore = defineStore({
    id: "auth",
    state: () => ({
        isAuthenticated: true, // デフォルトでは認証されているとする
    }),
    actions: {
        logout() {
            this.isAuthenticated = false;
        },
    },
});