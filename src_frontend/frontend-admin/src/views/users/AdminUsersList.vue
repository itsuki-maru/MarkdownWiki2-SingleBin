<script setup lang="ts">
import { ref, computed } from 'vue';
import apiClient from "@/axiosClient";
import { useRouter } from "vue-router";
import { getUserUrl, resetUserPasswordUrl, unlockUserAccountUrl, createUserUrl } from '@/router/urls';
import type { UserData, UpdateUserData } from '@/interface';
import { useUsersStore } from "@/stores/users";
import InlineEdit from "@/components/InlineEdit.vue";
import { baseUrl, assetsUrl } from "@/setting";
import { AxiosError } from "axios";

// Login.vueへのリダイレクト
const router = useRouter();
const loginRedirect = (): void => {
  router.push("/account/login");
}

const usersStore = useUsersStore();
const userList = computed(
  (): Map<string, UserData> => {
    return usersStore.usersList;
  }
);
// userデータが存在しなければ再取得
if (userList.value.size === 0) {
  usersStore.initList();
}

// ユーザーのロック解除
const unlockUserRequest = async (user_id: string): Promise<void> => {
  const unlockUrl = `${unlockUserAccountUrl}${user_id}`
  try {
    const response = await apiClient.post(
      unlockUrl,
    );
    usersStore.initList();

  } catch (error) {
    messageModalOpenClose("アカウントロック解除に失敗しました。");
    return
  }
  messageModalOpenClose("アカウントロックを解除しました。");
}

// 現在ユーザーの取得
const currentUser = ref("");
const getCurrentUser = async (): Promise<void> => {
  try {
    const response = await apiClient.get(
      getUserUrl
    );
    currentUser.value = response.data["username"];
  } catch (error) {
    loginRedirect();
  }
};
getCurrentUser();

// 日付時刻から日付のみを取り出す関数
function getDateForDateTime(dateTimeString: string): string {
  return dateTimeString.split("T")[0];
}

// ユーザーデータの初期化
const updateUserDataInit: UpdateUserData = {
  id: "",
  username: "",
  public_name: "",
  new_password: "",
  is_superuser: false,
};
const updateUserData = ref(updateUserDataInit);
const newPasswordRef = ref("");
const checkPasswordRef = ref("");

// ユーザーデータ表示モーダル
const showUpdateUserContent = ref(false);
const openUserModal = (selectedUserId: string): void => {
  // モーダルが開いていたら閉じ、閉じていたらクローズ
  if (showUpdateUserContent.value || selectedUserId === "") {
    showUpdateUserContent.value = false;
    return
  } else {
    showUpdateUserContent.value = true;
  }
  // ストアからユーザーを取得
  const updateUserFromStore = usersStore.getById(selectedUserId);
  updateUserData.value.id = updateUserFromStore.id;
  updateUserData.value.username = updateUserFromStore.username;
}


// 更新確認モーダル
const showUpdateUserCheckContent = ref(false);
const openUpdateCheckModal = (): void => {
  if (showUpdateUserCheckContent.value) {
    showUpdateUserCheckContent.value = false;
  } else {
    showUpdateUserCheckContent.value = true;
  }
}

// 更新処理
const updateUser = async (): Promise<void> => {
  const newPassword = newPasswordRef.value;
  const checkPassword = checkPasswordRef.value;

  // 入力項目の検証
  if (newPassword === "") {
    messageModalOpenClose("パスワードが入力されていません。");
    return
  }

  // 入力項目の検証
  if (newPassword !== checkPassword) {
    messageModalOpenClose("パスワードが一致しません。");
    return
  }

  const data = {
    "new_password": newPassword,
  }

  const updateUserApiUrl = `${resetUserPasswordUrl}${updateUserData.value.id}`
  try {
    const response = await apiClient.post(
      updateUserApiUrl,
      data,
    );
    console.log(response.data["message"]);
  } catch (error) {
    messageModalOpenClose("更新に失敗しました。");
    showUpdateUserCheckContent.value = false;
    return
  }

  openUpdateCheckModal();
  messageModalOpenClose("更新しました。");
  showUpdateUserContent.value = false;
}

// メッセージ表示モーダル機能
const isMessageModal = ref(false);
const messageText = ref("");
const messageModalOpenClose = (message: string): void => {
  if (!isMessageModal.value) {
    messageText.value = message;
    isMessageModal.value = true;
  } else {
    isMessageModal.value = false;
    messageText.value = "";
  }
};

// ユーザーの作成
const showCreateUserContent = ref(false);
const openCloseUserCreateModal = (): void => {
  if (showCreateUserContent.value) {
    showCreateUserContent.value = false;
  } else {
    showCreateUserContent.value = true;
  }
}
// サインアップ処理
const signupPost = async (): Promise<void> => {
  const username = signupInfoInit.username;
  const public_name = signupInfoInit.public_name;
  const password = signupInfoInit.password;

  if (username == "" || password == "" || public_name == "") {
    messageModalOpenClose("入力は全て必須です。");
    return
  }

  const payload = {
    "username": username,
    "public_name": public_name,
    "password": password,
  }

  try {
    const response = await apiClient.post(
      createUserUrl,
      payload
    );
    messageModalOpenClose("ユーザーの作成に成功しました。")
    usersStore.initList();
    signupInfo.value.username = "";
    signupInfo.value.public_name = "";
    signupInfo.value.password = "";
    return;

  } catch (error) {
    if (apiClient.isAxiosError(error)) {
      // エラーオブジェクトがAxiosError型であることが保証
      const axiosError = error as AxiosError<any>;
      if (axiosError.response) {
        const errorStatus = axiosError.response.data["error"];

        if (errorStatus === "Conflict") {
          messageModalOpenClose("既に使用されているユーザー名です。");

        } else {
          messageModalOpenClose("ユーザーの作成に失敗しました。");
        }
      }
    }
  }
};

interface typeSignup {
  username: string;
  public_name: string;
  password: string;
}

const signupInfoInit: typeSignup = {
  username: "",
  public_name: "",
  password: "",
};

const signupInfo = ref(signupInfoInit);
</script>

<template>
  <div class="admin-content">

    <div id="btn-head-zone">
      <button class="btn-head-image" title="新規ユーザーを作成します。"
        v-on:click="openCloseUserCreateModal">
        <img :src="`${assetsUrl}person_add_24.png`" class="btn-img" alt="person_add_24.png">
      </button>
    </div>

    <div class="users-list-area">
      <div id="table-title-row">
        <h1>ユーザーリスト</h1>
      </div>
      <div id="table-area-row">
        <div class="table_sticky">
          <table>
            <thead>
              <tr>
                <th>ID</th>
                <th>UserName</th>
                <th>PublicName</th>
                <th>PassWord</th>
                <th>CreateAt</th>
                <th>SuperUser</th>
                <th>Password</th>
                <th>Lock</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="[id, user] in userList" v-bind:key="id">
                <td style="text-align: center;">{{ user.id }}</td>
                <td>{{ user.username }}</td>
                <InlineEdit v-model="user.public_name" :data-key="id" />
                <td>*******************</td>
                <td>{{ getDateForDateTime(user.create_at) }}</td>
                <td>{{ user.is_superuser }}</td>
                <td><button class="btn-table" v-on:click="openUserModal(user.id)">Reset</button></td>
                <td v-if="user.is_locked"><button class="btn-table" v-on:click=unlockUserRequest(user.id)>Unlock</button></td>
                <td v-else="user.is_locked"></td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>

  <!-- 更新対象選択モーダル -->
  <transition>
    <div id="overlay-user-info" v-if="showUpdateUserContent">
      <div id="content-user-info">
        <h2 style="text-align: center;">パスワード更新</h2>
        <div class="title-input">
          <label class="row"><strong>更新対象：{{ updateUserData.username }}</strong></label>
          <input type="password" class="input-text user-list-password" placeholder="New Password"
            v-model="newPasswordRef">
          <input type="password" class="input-text user-list-password" placeholder="Check Password"
            v-model="checkPasswordRef">
        </div>
        <div class="btn-zone">
          <button v-on:click="openUserModal('')">閉じる</button>
          <button v-on:click="openUpdateCheckModal()">更新</button>
        </div>
      </div>
    </div>
  </transition>

  <!-- ユーザー作成モーダル -->
  <transition>
    <div id="overlay-create-user" v-if="showCreateUserContent">
      <div id="content-create-user">
        <h2 style="text-align: center;">ユーザー作成</h2>
        <div>
          <input type="text" pattern="^[A-Za-z0-9]{3,}$" title="3文字以上。半角英数字が使用可能。" class="input-text user-list-password"
            placeholder="ユーザー名（3文字以上、半角英数字が使用可）" autocomplete="username" required v-model="signupInfo.username" />
          <input type="password" pattern=".{8,}" title="8文字以上で入力してください。" placeholder="パスワード（8文字以上）" autocomplete="current-password" required
            v-model="signupInfo.password" class="input-text user-list-password" />
          <input type="text" title="2文字以上10文字以下" placeholder="表記ユーザー名" name="public_name" required
            minlength="2" maxlength="10" v-model="signupInfo.public_name" class="input-text user-list-password" />
        </div>
        <div class="btn-zone">
          <button v-on:click="openCloseUserCreateModal()">閉じる</button>
          <button v-on:click="signupPost()">アカウント作成</button>
        </div>
      </div>
    </div>
  </transition>

  <!-- 更新確認モーダル -->
  <transition>
    <div id="overlay-update-check" v-if="showUpdateUserCheckContent">
      <div id="content-update-check">
        <h2>最終確認</h2>
        <p><strong>パスワードを更新しますか？</strong></p>
        <div class="btn-zone">
          <button v-on:click="openUpdateCheckModal()">やめる</button>
          <button v-on:click="updateUser">更新</button>
        </div>
      </div>
    </div>
  </transition>

  <!-- 各種メッセージモーダル -->
  <transition>
    <div id="overlay-message" v-show="isMessageModal">
      <div id="content-message">
        <h2>メッセージ</h2>
        <div class="input-text-zone">
          <p><strong>{{ messageText }}</strong></p>
        </div>
        <div class="btn-close">
          <button id="message-close-btn" v-on:click="messageModalOpenClose('No Message')">閉じる</button>
        </div>
      </div>
    </div>
  </transition>
  <footer>
    <p class="login-user">ログインユーザー: {{ currentUser }}</p>
  </footer>
</template>

<style scoped>
.admin-content {
  height: 77vh;
}

#btn-head-zone {
  display: flex;
  justify-content: space-between;
}

input[type="text"],
input[type="password"],
input[type="number"],
textarea {
    border-radius: 8px;
    border: 1px solid transparent;
    padding: 0.6em 1.2em;
    font-family: inherit;
    box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
    box-sizing: border-box;
}

input[type="text"],
input[type="password"],
input[type="number"],
textarea,
button {
    outline: none;
}

.title-input {
  width: 100%;
}

.row {
  display: flex;
  margin-bottom: 3px;
  margin-left: 5px;
}

.input-text {
  font-size: 20px;
  width: 100%;
  padding: 0.6em 1.2em;
  display: flex;
  text-align: center;
  margin-bottom: 2%;
  border-radius: 5px;
  box-sizing: border-box;
}

.input-text:focus {
  outline: none;
  border-color: #007BFF;
  box-shadow: 0 0 5px rgba(0, 123, 255, 0.5);
}

.users-list-area {
  margin-bottom: 2%;
}

#table-title-row {
  margin-bottom: -1%;
}

.btn-zone {
  display: flex;
  justify-content: space-between;
}

.table_sticky table {
  margin-top: 0;
  width: 100%;
}

.table_sticky {
  display: block;
  overflow-y: auto;
  height: auto;
  margin-top: 1%;
}

.table_sticky td {
  text-align: center;
  font-size: 12px;
}

/* テーブルのホバー：ボディ部分の行のみホバー時のスタイルを適用 */
.table_sticky table tbody tr:hover {
  background-color: #72a5b4;
}

.table_sticky thead th {
  position: sticky;
  top: 0;
  z-index: 1;
  background: rgb(44, 52, 78);
  color: whitesmoke;
  text-align: center;
}

.table_stickyt h:nth-child(1) {
  font-size: 14px;
  width: 5%;
}

.table_stickyt h:nth-child(2) {
  font-size: 14px;
  width: 15%;
}

.table_stickyt h:nth-child(3) {
  font-size: 14px;
  width: 20%;
}

.table_stickyt h:nth-child(4) {
  font-size: 14px;
  width: 20%;
}

.table_stickyt h:nth-child(5) {
  font-size: 14px;
  width: 10%;
}

.table_stickyt h:nth-child(6) {
  font-size: 14px;
  width: 20%;
}

.table_sticky th:nth-child(7) {
  font-size: 14px;
  width: 10%;
}

.table_sticky th:nth-child(8) {
  font-size: 14px;
  width: 10%;
}

#user-list-username {
  font-size: 16px;
  border-radius: 3px;
}

.user-list-password {
  font-size: 16px;
  border-radius: 3px;
}

.login-user {
  margin-top: 15px;
  text-align: right;
  font-size: 16px;
  font-weight: bold;
  text-shadow: 1px 1px 2px rgb(202, 202, 202);
}

/* ユーザー現在情報モーダル */
#overlay-user-info, #overlay-create-user {
  z-index: 1;
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
}

#content-user-info, #content-create-user {
  z-index: 2;
  width: 30%;
  padding: 1em;
  background: #fff;
  border-radius: 10px;
  margin: 20px auto;
  margin-top: -20px;
  padding: 20px;
  box-sizing: border-box;
}

/* 更新確認モーダル */
#overlay-update-check {
  z-index: 1;
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
}

#content-update-check {
  z-index: 3;
  width: 20%;
  text-align: center;
  padding: 1em;
  background: #fff;
  border-radius: 10px;
}

/* メッセージモーダル */
#overlay-message {
  z-index: 3;
  text-align: center;
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
}

#content-message {
  z-index: 4;
  width: 23%;
  padding: 1em;
  background: whitesmoke;
  border-radius: 10px;
}

.lockuser-btn-zone {
  margin: 0 0 0 auto;
  text-align: right;
}

.get-lock-user-btn {
  height: 40px;
  background: rgb(45, 110, 145);
  color: #fff;
  padding: 10px 7px;
  text-decoration: none;
  border: 1px;
  border-radius: 10px;
  transition-property: opacity;
  -webkit-transition-property: opacity;
  transition-duration: .5s;
  -webkit-transition-duration: .5s;
  text-align: right;
}
</style>