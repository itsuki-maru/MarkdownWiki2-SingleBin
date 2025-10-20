<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, inject } from 'vue';
import type { Ref } from "vue";
import { useRouter } from "vue-router";
import type { WikiData, QueryForm } from '@/interface';
import { assetsUrl } from "@/setting";
import { useWikiStore } from "@/stores/wikis";
import UserPrivacySetting from "@/components/UserPrivacySetting.vue";

// App.vueで定義したメモアイコンの表示非表示管理変数をinject
const isShowMemoIcon = inject("isShowMemoIcon") as Ref<boolean>;
// サインアップ・ログイン画面では非表示にする
isShowMemoIcon.value = true;

// Login.vueへのリダイレクト
const router = useRouter();
const loginRedirect = (): void => {
  router.push("/account/login");
};

// Preview.vueへのリダイレクト
const previewRedirect = (id: string): void => {
  localStorage.setItem("prev-table-data", `table-dataid-${id}`);
  router.push(`/wiki/preview/${id}`);
};

// Create.vueへのリダイレクト
const createRedirect = (): void => {
  router.push("/wiki/create");
}

const wikiStore = useWikiStore();
const wikiList = computed(
  (): Map<string, WikiData> => {
    return wikiStore.wikiList;
  }
);
// wikiデータが存在しなければ再取得
if (wikiList.value.size === 0) {
  wikiStore.initList();
}

const queryFormDataInit: QueryForm = {
  query1: "",
  query2: "",
};

const queryFormData = ref(queryFormDataInit);
watch(queryFormData.value, (): void => {
  onSearch();
});

// 検索実行関数
const onSearch = (reset: boolean = false): void => {
  localStorage.setItem("prev-table-data", "");
  try {
    if (reset) {
      wikiStore.queryWiki("", "");
    } else {
      wikiStore.queryWiki(queryFormData.value.query1, queryFormData.value.query2);
    }
  } catch (error) {
    console.log(error);
  }
}

// 更新日時を取得
const getUpdateAt = (dateStr: string, datetimeStr: string): string => {
  if (areDatesSame(dateStr, datetimeStr)) {
    return "";
  }
  const dateStrSplit = datetimeStr.split("T")[0];
  const timeStrSplit = datetimeStr.split("T")[1];
  return ` 【更新：${dateStrSplit} ${timeStrSplit.substring(0, 5)} 】`;
}

// 日付時刻から日付のみを取り出す関数
function getDateForDateTime(dateTimeString: string): string {
  return dateTimeString.split("T")[0];
}

// 日付を比較する関数
function areDatesSame(dateString1: string, dateString2: string): boolean {
  // dateString1と2から日付部分だけを取り出す
  const datePartOfDateTime1 = getDateForDateTime(dateString1);
  const datePartOfDateTime2 = getDateForDateTime(dateString2);

  // 日付を比較する
  return datePartOfDateTime1 === datePartOfDateTime2;
}

// ISO形式の日付を変換
function formatDateJP(isoString: string, isDayOnly: boolean = false, isJpFormat: boolean = false, addMinutes: number = 540): string {
  const date = new Date(isoString);
  // UTC時間を取得し、9時間（540分）加算してJSTに変換
  date.setMinutes(date.getMinutes() + addMinutes);
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0"); // 月は0始まりのため+1
  const day = String(date.getDate()).padStart(2, "0");
  const hours = String(date.getHours()).padStart(2, "0");
  const minutes = String(date.getMinutes()).padStart(2, "0");
  if (isDayOnly) {
    if (isJpFormat) {
      return `${year}年${month}月${day}日`
    } else {
      return `${year}-${month}-${day}`
    }
  } else {
    if (isJpFormat) {
      return `${year}年${month}月${day}日 ${hours}時${minutes}分`
    } else {
      return `${year}-${month}-${day} ${hours}:${minutes}`
    }
  }
}

// テーブルスクロール
const scrolledTableId = localStorage.getItem("prev-table-data");
onMounted(() => {
  if (scrolledTableId) {
    if (scrolledTableId != "") {
      let targetTableRowIdElm = document.getElementById(scrolledTableId);
      targetTableRowIdElm?.scrollIntoView({
        block: "start"
      });
      let tableElm = document.getElementById("wiki-table");
      if (scrolledTableId !== "table-dataid-1") {
        tableElm?.scrollBy({
          top: -50,
        })
      }
    }
  }
});

// キーボードショートカットを追加
const handleKeyDown = (event: KeyboardEvent) => {
  // Createへ遷移
  if (event.ctrlKey && event.key == "1") {
    event.preventDefault();
    createRedirect();

    // 検索Word1にフォーカス
  } else if (event.ctrlKey && event.key == "3") {
    event.preventDefault();
    const textElement = document.getElementById("query1");
    if (textElement) {
      textElement.focus();
    }

    // 検索Word2にフォーカス
  } else if (event.ctrlKey && event.key == "4") {
    event.preventDefault();
    const textElement = document.getElementById("query2");
    if (textElement) {
      textElement.focus();
    }

    // 検索の実行
  } else if (event.ctrlKey && event.key == "5") {
    event.preventDefault();
    onSearch(false);

    // 検索のクリア
  } else if (event.ctrlKey && event.key == "6") {
    event.preventDefault();
    onSearch(true);
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeyDown);
});
onUnmounted(() => {
  window.removeEventListener("keydown", handleKeyDown);
});

// テーブルのソート
function onSort() {
  wikiStore.sortWiki();
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


const userSettingModalRef = ref<{
  openCloseUserSettingModal: () => void;
  isUserPrivate: boolean;
  isInitialized: boolean;
} | null>(null);

const userPrivacySettingFunction = () => {
  if (userSettingModalRef.value) {
    userSettingModalRef.value.openCloseUserSettingModal();
  }
}

watch(() => userSettingModalRef.value?.isUserPrivate,
  (newValue, oldValue) => {
    if (!userSettingModalRef.value?.isInitialized) return; // 子コンポーネントでモーダルを起動していない場合は発火しない
    if (newValue) {
      messageModalOpenClose("プライバシーモードが ON になりました。他のユーザーはあなたのデータにアクセスできません。");
    } else {
      messageModalOpenClose("プライバシーモードが OFF になりました。 他のユーザーにあなたの画像などをシェアすることができます。");
    }
  });
</script>

<template>
  <div v-if="isShowMemoIcon" id="privacy-setting">
    <button v-if="userSettingModalRef?.isUserPrivate" v-on:click="userPrivacySettingFunction()" class="btn-privacy-icon"
      title="ユーザー設定&#10;アカウントのプライバシー設定を変更します。">
      <img :src="`${assetsUrl}lock_24.png`" class="btn-img" alt="lock_24.png"></button>
    <button v-if="!userSettingModalRef?.isUserPrivate" v-on:click="userPrivacySettingFunction()"
      class="btn-privacy-icon" title="ユーザー設定&#10;アカウントのプライバシー設定を変更します。">
      <img :src="`${assetsUrl}lock_open_24.png`" class="btn-img" alt="lock_open_24.png"></button>
  </div>

  <!-- ユーザー設定変更モーダル -->
  <UserPrivacySetting ref="userSettingModalRef"></UserPrivacySetting>

  <!-- 各種メッセージモーダル -->
  <div id="overlay-message" v-show="isMessageModal">
    <div id="content-message">
      <h2 class="modal-h2">メッセージ</h2>
      <div class="input-text-zone">
        <p><strong>{{ messageText }}</strong></p>
      </div>
      <div class="btn-close">
        <button v-on:click="messageModalOpenClose('No Message')">閉じる</button>
      </div>
    </div>
  </div>

  <div class="head-btn-zone">
    <div class="btn-area">
      <button class="btn-head-img" v-on:click="createRedirect()"><img :src="`${assetsUrl}add_fill24.png`"
          class="btn-img" alt="add_fill24.png"></button>
      <button class="btn-head-img" type="submit" v-on:click="onSearch(true)"><img :src="`${assetsUrl}update_fill24.png`"
          class="btn-img" alt="update_fill24.png"></button>
    </div>
    <div class="form-area">
      <input type="text" id="query1" class="query-input" placeholder="検索ワード1" v-model="queryFormData.query1">
      <input type="text" id="query2" class="query-input" placeholder="検索ワード2" v-model="queryFormData.query2">
    </div>
  </div>

  <div class="table_sticky" id="wiki-table">
    <table>
      <thead>
        <tr>
          <th v-on:click="onSort()" title="タップにより更新日時でソート">Wiki</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="[id, wiki] in wikiList" v-bind:key="id">
          <td :id="`table-dataid-${id}`" v-on:click="previewRedirect(id)">{{ formatDateJP(wiki.date) }}<br>{{ wiki.title
            }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.btn-area {
  display: flex;
}

.form-area {
  display: flex;
  gap: 4px;
  padding: 4px;
}

.table_sticky table {
  margin-top: 0;
  width: 100%;
}

.table_sticky {
  display: block;
  overflow-y: auto;
  width: 100%;
  height: 70vh;
  margin-top: 1%;
}

.table_sticky thead th {
  position: sticky;
  top: 0;
  width: 100%;
  z-index: 1;
  background: rgb(44, 52, 78);
  color: whitesmoke;
}

td {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100px;
}

#privacy-setting {
  position: absolute;
  top: 10px;
  right: 10px;
}

/* メッセージモーダル */
#overlay-message {
  z-index: 19;
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  ;
  align-items: center;
  justify-content: center;
}

#content-message {
  z-index: 20;
  width: 90%;
  padding: 1em;
  background: whitesmoke;
  border-radius: 10px;
}

.btn-privacy-icon {
  width: 50px;
  height: 35px;
  box-shadow: 3px 3px 5px 0 rgba(75, 75, 75, 0.5);
  background: white;
  color: #fff;
  padding: 5px 7px;
  text-decoration: none;
  border: 1px solid rgb(207, 207, 207);
  border-radius: 14px;
  transition-property: opacity;
  -webkit-transition-property: opacity;
  transition-duration: .5s;
  -webkit-transition-duration: .5s;
  transition: background-color 0.3s;
  margin: 5px 5px 10px 5px;
}
</style>