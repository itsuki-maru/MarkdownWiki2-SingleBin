import { defineStore } from "pinia";
import type { WikiData, UpdateWikiData } from "@/interface";
import {wikiDataListCountUrl, wikiDataListQueryUrl } from "@/router/urls";
import apiClient from "@/axiosClient";

interface State {
    wikiList: Map<string, WikiData>;
}

export const useWikiStore = defineStore({
    id: "wiki",
    state: (): State => {
        return {
            wikiList: new Map<string, WikiData>()
        };
    },
    getters: {
        getById: (state) => {
            return (id: string): WikiData => {
                const wiki = state.wikiList.get(id) as WikiData;
                return wiki;
            };
        }
    },
    actions: {
        async initList(): Promise<void> {
            try {
                const response = await apiClient.get(
                    wikiDataListCountUrl,
                );
                this.wikiList.clear();
                const wikisData = response.data;
                for (let key in wikisData) {
                    this.wikiList.set(
                        wikisData[key]["id"], {
                            id: wikisData[key]["id"],
                            user_id: wikisData[key]["user_id"],
                            date: wikisData[key]["date"],
                            title: wikisData[key]["title"],
                            body: wikisData[key]["body"],
                            update_at: wikisData[key]["update_at"],
                            is_public: wikisData[key]["is_public"]
                        }
                    );
                }
                this.sortWiki();
            } catch(error) {
                console.error("Init List Error.");
            }
        },
        async queryWiki(query1: string, query2: string): Promise<void> {
            try {
                const response = await apiClient.get(
                    wikiDataListQueryUrl + `?query1=${query1}&query2=${query2}`
                );
                this.wikiList.clear();
                const wikisData = response.data;
                for (let key in wikisData) {
                    this.wikiList.set(
                        wikisData[key]["id"], {
                            id: wikisData[key]["id"],
                            user_id: wikisData[key]["user_id"],
                            date: wikisData[key]["date"],
                            title: wikisData[key]["title"],
                            body: wikisData[key]["body"],
                            update_at: wikisData[key]["update_at"],
                            is_public: wikisData[key]["is_public"]
                        }
                    );
                }
                
                let sortedDsc = new Map([...this.wikiList.entries()].sort((a, b) => a[0] > b[0] ? 1: -1).reverse());
                this.wikiList = sortedDsc;
            } catch(erroe) {
                console.error("List Query Error.");
            }
        },
        addWiki(wiki: WikiData): void {
            this.wikiList.set(wiki.id, {id: wiki.id,
                                            user_id: wiki.user_id, 
                                            date: getTimeStamp(), // サーバーUTCに9時間加算した時刻を取得
                                            title: wiki.title, 
                                            body: wiki.body,
                                            update_at: wiki.update_at,
                                            is_public: wiki.is_public});
            let sortedDsc = new Map([...this.wikiList.entries()].sort((a, b) => a[0] > b[0] ? 1: -1).reverse());
            this.wikiList = sortedDsc;
        },
        updateWiki(wiki: UpdateWikiData): void {
            const updateWiki = this.getById(wiki.id);
            updateWiki.title = wiki.title;
            updateWiki.body = wiki.body;
            updateWiki.is_public = wiki.is_public
        },
        deleteWiki(wiki_id: string): void {
            console.log("Delete wiki.");
            this.wikiList.delete(wiki_id);
        },
        clearWiki(): void {
            this.wikiList.clear();
        },
        async getPublicOnlyData(): Promise<void> {
            try {
                const response = await apiClient.get(
                    wikiDataListCountUrl,
                );
                this.wikiList.clear();
                const wikisData = response.data;
                for (let key in wikisData) {
                    if (wikisData[key]["is_public"] === true) {
                        this.wikiList.set(
                            wikisData[key]["id"], {
                                id: wikisData[key]["id"],
                                user_id: wikisData[key]["user_id"],
                                date: wikisData[key]["date"],
                                title: wikisData[key]["title"],
                                body: wikisData[key]["body"],
                                update_at: wikisData[key]["update_at"],
                                is_public: wikisData[key]["is_public"]
                            }
                        );
                    }
                }
                let sortedDsc = new Map([...this.wikiList.entries()].sort((a, b) => a[0] > b[0] ? 1: -1).reverse());
                this.wikiList = sortedDsc;
            } catch(error) {
                console.log("Get Public Only Data Error.");
            }
        },
        sortWiki(): void {
            const wikiListArr = Array.from(this.wikiList.entries());
            wikiListArr.sort(([, a], [, b]) => {
                return new Date(b.update_at).getTime() - new Date(a.update_at).getTime();
            });
            this.wikiList = new Map(wikiListArr);
        }
    }
});

// Wiki作成時の時刻生成（リスト表示時に540分加算されてしまうため、これを打ち消す）
function getTimeStamp() {
    const now = new Date();
    now.setMinutes(now.getMinutes() - 540);
    const year = now.getUTCFullYear();
    const month = String(now.getUTCMonth() + 1).padStart(2, "0");
    const day = String(now.getUTCDate()).padStart(2, "0");
    const hours = String(now.getUTCHours()).padStart(2, "0");
    const minutes = String(now.getUTCMinutes()).padStart(2, "0");
    const seconds = String(now.getUTCSeconds()).padStart(2, "0");
    const milliseconds = String(now.getUTCMilliseconds()).padStart(3, "0");
    const nanoseconds = milliseconds + "000000"; // ナノ秒精度のゼロ埋め
    return `${year}-${month}-${day}T${hours}:${minutes}:${seconds}.${nanoseconds}Z`;
}