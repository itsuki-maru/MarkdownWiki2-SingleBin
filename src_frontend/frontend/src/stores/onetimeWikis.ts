import { defineStore } from 'pinia';
import type { OneTimeWikis } from '@/interface';
import { onetimeWikiListGetUrl, invalidateOntimeWikiUrl } from '@/router/urls';
import apiClient from '@/axiosClient';

interface State {
  onetimeWikiList: Map<string, OneTimeWikis>;
}

export const useOnetimeWikiStore = defineStore('onetimeWikis', {
  state: (): State => {
    return {
      onetimeWikiList: new Map<string, OneTimeWikis>(),
    };
  },
  getters: {
    getById: (state) => {
      return (id: string): OneTimeWikis => {
        const onetimeWiki = state.onetimeWikiList.get(id) as OneTimeWikis;
        return onetimeWiki;
      };
    },
  },
  actions: {
    async initList(): Promise<void> {
      try {
        const response = await apiClient.get(onetimeWikiListGetUrl);
        this.onetimeWikiList.clear();
        const onetimeWikisData = response.data;
        for (let key in onetimeWikisData) {
          this.onetimeWikiList.set(onetimeWikisData[key]['id'], {
            id: onetimeWikisData[key]['id'],
            user_id: onetimeWikisData[key]['user_id'],
            wiki_id: onetimeWikisData[key]['wiki_id'],
            url: onetimeWikisData[key]['url'],
            expiration: onetimeWikisData[key]['expiration'],
            title: onetimeWikisData[key]['title'],
            create_at: onetimeWikisData[key]['create_at'],
          });
        }
        this.sortWiki();
      } catch (error) {
        console.log('Init List Error.');
      }
    },
    async deleteOnetimeWiki(onetimeWiki_id: string): Promise<void> {
      try {
        const url = invalidateOntimeWikiUrl + `${onetimeWiki_id}`;
        const response = await apiClient.delete(url);
      } catch (error) {
        console.error('Error');
      }
      this.onetimeWikiList.delete(onetimeWiki_id);
    },
    sortWiki(): void {
      const onetimeWikiListArr = Array.from(this.onetimeWikiList.entries());
      onetimeWikiListArr.sort(([, b], [, a]) => {
        return new Date(b.expiration).getTime() - new Date(a.expiration).getTime();
      });
      this.onetimeWikiList = new Map(onetimeWikiListArr);
    },
  },
});
