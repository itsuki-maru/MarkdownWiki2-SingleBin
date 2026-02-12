import { defineStore } from 'pinia';
import type { UserData, UpdateUserData } from '@/interface';
import apiClient from '@/axiosClient';
import { getUsersUrl, updateUserPublicNameUrl } from '@/router/urls';

interface State {
  usersList: Map<string, UserData>;
}

export const useUsersStore = defineStore('user', {
  state: (): State => {
    return {
      usersList: new Map<string, UserData>(),
    };
  },
  getters: {
    getById: (state) => {
      return (id: string): UserData => {
        const user = state.usersList.get(id) as UserData;
        return user;
      };
    },
  },
  actions: {
    async initList(): Promise<void> {
      try {
        const response = await apiClient.get(getUsersUrl);
        this.usersList.clear();
        const usersData = response.data;
        for (let key in usersData) {
          this.usersList.set(usersData[key]['id'], {
            id: usersData[key]['id'],
            username: usersData[key]['username'],
            public_name: usersData[key]['public_name'],
            password: usersData[key]['password'],
            create_at: usersData[key]['create_at'],
            is_superuser: usersData[key]['is_superuser'],
            is_locked: usersData[key]['is_locked'],
          });
        }
        let sortedDsc = new Map(
          [...this.usersList.entries()].sort((a, b) => (a[0] > b[0] ? 1 : -1)).reverse(),
        );
        this.usersList = sortedDsc;
      } catch (error) {
        console.log('Init List Error.');
      }
    },
    async updatePublicName(id: string, newName: string): Promise<void> {
      // ペイロードに追加
      const payload = {
        public_name: newName,
      };

      const updateUrlJoinId = updateUserPublicNameUrl + id;
      try {
        const response = await apiClient.put(updateUrlJoinId, payload);
        const updateUser = this.getById(id);
        updateUser.public_name = newName;
        this.initList();
      } catch (error) {
        console.log('Update Error.');
        return;
      }
    },
  },
});
