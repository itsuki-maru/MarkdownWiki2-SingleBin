import { defineStore } from "pinia";
import type { EditRequestWiki } from "@/interface";
import { getEditWikiRequestsUrl } from "@/router/urls";
import apiClient from "@/axiosClient";

interface State {
    editRequestWikiList: Map<string, EditRequestWiki>;
}

export const useEditRequestWikiStore = defineStore("editRequestWiki", {
    state: (): State => {
        return {
            editRequestWikiList: new Map<string, EditRequestWiki>()
        };
    },
    getters: {
        getById: (state) => {
            return (id: string): EditRequestWiki => {
                const wiki = state.editRequestWikiList.get(id) as EditRequestWiki;
                return wiki;
            };
        }
    },
    actions: {
        async initList(): Promise<void> {
            try {
                const response = await apiClient.get(
                    getEditWikiRequestsUrl,
                );
                this.editRequestWikiList.clear();
                const wikisData = response.data;
                for (let key in wikisData) {
                    this.editRequestWikiList.set(
                        wikisData[key]["id"], {
                            id: wikisData[key]["id"],
                            wiki_owner_id: wikisData[key]["wiki_owner_id"],
                            request_public_user_name: wikisData[key]["request_public_user_name"],
                            request_wiki_id: wikisData[key]["request_wiki_id"],
                            original_title: wikisData[key]["original_title"],
                            original_body: wikisData[key]["original_body"],
                            edit_request_title: wikisData[key]["edit_request_title"],
                            edit_request_body: wikisData[key]["edit_request_body"],
                            create_at: wikisData[key]["create_at"],
                            status: wikisData[key]["status"],
                        }
                    );
                }
                this.sortEditRequestWiki();
            } catch(error) {
                console.error("Init List Error.");
            }
        },
        addEditRequestWiki(wiki: EditRequestWiki): void {
            this.editRequestWikiList.set(
                wiki.id, {
                    id: wiki.id,
                    wiki_owner_id: wiki.wiki_owner_id, 
                    request_public_user_name: wiki.request_public_user_name,
                    request_wiki_id: wiki.request_wiki_id,
                    original_title: wiki.original_title,
                    original_body: wiki.original_body,
                    edit_request_title: wiki.edit_request_title,
                    edit_request_body: wiki.edit_request_body,
                    create_at: wiki.create_at,
                    status: wiki.status,
                }
            );
            let sortedDsc = new Map([...this.editRequestWikiList.entries()].sort((a, b) => a[0] > b[0] ? 1: -1).reverse());
            this.editRequestWikiList = sortedDsc;
        },
        deleteEditRequestWiki(wiki_id: string): void {
            console.log("Delete wiki edit request.");
            this.editRequestWikiList.delete(wiki_id);
        },
        clearEditRequestWiki(): void {
            this.editRequestWikiList.clear();
        },
        sortEditRequestWiki(): void {
            const editRequestWikiListArr = Array.from(this.editRequestWikiList.entries());
            editRequestWikiListArr.sort(([, a], [, b]) => {
                return new Date(b.create_at).getTime() - new Date(a.create_at).getTime();
            });
            this.editRequestWikiList = new Map(editRequestWikiListArr);
        }
    }
});