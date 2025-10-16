import { defineStore } from "pinia";
import type { ApplicationInit } from "../interface";
import axios from "axios";
import { getAppIniturl } from "@/router/urls";


const DEFAULT_STATE: ApplicationInit = {
    appTitle: "",
    allowUserAccountCreate: false,
    allowOrigins: "",
};

interface State {
    appInitData: ApplicationInit
};

export const useApplicationInitStore = defineStore({
    id: "appInits",
    state: (): State => {
        return {
            appInitData: DEFAULT_STATE
        };
    },
    actions: {
        async init(): Promise<void> {
            try {
                const response = await axios.get(getAppIniturl);
                if (response) {
                    this.appInitData.appTitle = response.data["app_title"];
                    if (response.data["allow_user_account_create"] === "true") {
                        this.appInitData.allowUserAccountCreate = true;
                    } else {
                        this.appInitData.allowUserAccountCreate = false;
                    }
                    this.appInitData.allowOrigins = response.data["allow_origins"];
                }
            } catch (error) {
                console.error("Init data get error.");
            }
        },
        // クリア
        clear() {
            this.appInitData = DEFAULT_STATE;
        }
    }
});
