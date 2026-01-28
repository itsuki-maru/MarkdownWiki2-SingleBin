<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, inject, nextTick} from 'vue';
import type { Ref } from "vue";
import { useRouter } from "vue-router";
import type { WikiData, QueryForm, EditRequestWiki } from '@/interface';
import { assetsUrl } from "@/setting";
import { useWikiStore } from "@/stores/wikis";
import UserPrivacySetting from "@/components/UserPrivacySetting.vue";
import { AxiosError } from "axios";
import { postOwnerResultUrl, disableEditWikiUrl, getUserUrl } from '@/router/urls';
import { useEditRequestWikiStore } from "@/stores/editWikis";
import apiClient from "@/axiosClient";

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

// 現在ユーザーの取得
const currentUser = ref("");
const currenrUserId = ref("");
const getCurrentUser = async (): Promise<void> => {
  try {
    const response = await apiClient.get(
      getUserUrl
    );
    currentUser.value = response.data["public_name"];
    currenrUserId.value = response.data["id"];
  } catch (error) {
    loginRedirect();
  }
};
getCurrentUser();

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

// 変更リクエストストア
const editRequestWikiStore = useEditRequestWikiStore();
const editRequestWikiList = computed(
  (): Map<string, EditRequestWiki> => {
    return editRequestWikiStore.editRequestWikiList;
  }
)

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

// 更新リクエスト一覧モーダル
const isOpenEditRequestWikis = ref(false);
const openCloseEditRequestWikis = (): void => {
  editRequestWikiStore.initList();
  if (isOpenEditRequestWikis.value) {
    isOpenEditRequestWikis.value = false;
  } else {
    isOpenEditRequestWikis.value = true;
  }
}

// 更新リクエストステータス対応
const statusTable = {
  "REJECT": "却下",
  "REQUESTNOW": "申請中",
  "DRAFT": "下書き",
  "APPLIED": "承認",
}

// Diff表示モーダルの表示非表示
const showDiffPreviewModal = ref(false);
const clickedRequestWikiId = ref("");

const originalAreaRef = ref<HTMLElement | null>(null);
const modifiedAreaRef = ref<HTMLElement | null>(null);

const onOpenCloseDiffModal = async (
  id: string,
  text1: string,
  text2: string,  
  isClose: boolean = false
) => {
  clickedRequestWikiId.value = id;

  if (isClose) {
    showDiffPreviewModal.value = false;
    return;
  }

  showDiffPreviewModal.value = true;

  await nextTick();
  displayDiffs(text1, text2);
};

const diff_match_patch: any = (window as any).diff_match_patch;
const dmp = new diff_match_patch();

function displayDiffs(text1: string, text2: string) {
  const diffs = dmp.diff_main(text1, text2);
  dmp.diff_cleanupSemantic(diffs);

  const containerOriginal = originalAreaRef.value;
  const containerModified = modifiedAreaRef.value;
  if (!containerOriginal || !containerModified) return;

  containerOriginal.replaceChildren();
  containerModified.replaceChildren();

  diffs.forEach((diff: any[]) => {
    const operation = diff[0];
    const text = diff[1];

    const span = document.createElement("span");
    span.textContent = text;

    switch (operation) {
      case -1:
        span.classList.add("delete");
        containerOriginal.appendChild(span);
        break;
      case 1:
        span.classList.add("added");
        containerModified.appendChild(span);
        break;
      case 0:
        containerOriginal.appendChild(span.cloneNode(true));
        containerModified.appendChild(span);
        break;
    }
  });
}

// 承認・却下
const resultOwnerRequest = async (isReject: boolean): Promise<void> => {
  const payload = {
    "id": clickedRequestWikiId.value,
    "reject": isReject, 
  }

  try {
    await apiClient.post(
      postOwnerResultUrl,
      payload
    );
    onOpenCloseDiffModal("", "", "", true);
    editRequestWikiStore.initList();
    wikiStore.initList();

    if (isReject) {
      messageModalOpenClose("却下しました。");
    } else {
      messageModalOpenClose("承認しました。");
    }
  } catch (error) {
    if (apiClient.isAxiosError(error)) {
      // エラーオブジェクトがAxiosError型であることが保証
      const axiosError = error as AxiosError<any>;
      const errorStatusCode = axiosError.response?.status;
      if (errorStatusCode === 404) {
        messageModalOpenClose("すでに申請者が取り下げた申請のため、変更は適用されませんでした。");
        showDiffPreviewModal.value = false;
        editRequestWikiStore.initList();
        return;
      }
    }
  }
};

// 取り下げ
const disableEditRequest = async (id: string): Promise<void> => {
  const url = `${disableEditWikiUrl}${id}`;
  console.log(url);
  try {
    await apiClient.delete(
      url,
    );
    onOpenCloseDiffModal("", "", "", true);
    editRequestWikiStore.initList();
    messageModalOpenClose("更新申請を取り下げました。");
  } catch (error) {
    if (apiClient.isAxiosError(error)) {
      // エラーオブジェクトがAxiosError型であることが保証
      const axiosError = error as AxiosError<any>;
      const errorStatusCode = axiosError.response?.status;
      if (errorStatusCode === 404) {
        messageModalOpenClose("オーナーが却下後に承認しました。");
        showDiffPreviewModal.value = false;
        editRequestWikiStore.initList();
        return;
      }
    }
  }
};

// 更新日時を取得
const getUpdateAt = (dateStr: string, datetimeStr: string): string => {
  if (areDatesSame(dateStr, datetimeStr)) {
    return "";
  }
  const dateStrSplit = datetimeStr.split("T")[0];
  const timeStrSplit = datetimeStr.split("T")[1];
  return ` 【更新：${dateStrSplit} ${timeStrSplit!.substring(0, 5)} 】`;
}

// 日付時刻から日付のみを取り出す関数
function getDateForDateTime(dateTimeString: string): string {
  return dateTimeString.split("T")[0]!;
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
    <button title="更新リクエスト" v-on:click="openCloseEditRequestWikis()" class="btn-privacy-icon">
      <img :src="`${assetsUrl}edit_notifications_24.png`" class="btn-img" alt="edit_notifications_24.png"></button>
    <button v-if="userSettingModalRef?.isUserPrivate" v-on:click="userPrivacySettingFunction()" class="btn-privacy-icon"
      title="ユーザー設定&#10;アカウントのプライバシー設定を変更します。">
      <img :src="`${assetsUrl}lock_24.png`" class="btn-img" alt="lock_24.png"></button>
    <button v-if="!userSettingModalRef?.isUserPrivate" v-on:click="userPrivacySettingFunction()"
      class="btn-privacy-icon" title="ユーザー設定&#10;アカウントのプライバシー設定を変更します。">
      <img :src="`${assetsUrl}lock_open_24.png`" class="btn-img" alt="lock_open_24.png"></button>
  </div>

  <!-- ユーザー設定変更モーダル -->
  <UserPrivacySetting ref="userSettingModalRef"></UserPrivacySetting>

  <!-- 更新リクエストWikiモーダル-->
  <div id="overlay-edit-request-list" v-show="isOpenEditRequestWikis">
    <div id="content-edit-request-wikis">
      <h2 class="modal-h2">更新リクエスト一覧</h2>
      <div v-if="editRequestWikiList.size === 0"><p style="text-align: center;">申請中及び受け付けた変更リクエストはありません。</p></div>
      <div v-else class="table_sticky_edit_requests">
        <table>
          <thead>
            <tr>
              <th>申請者</th>
              <th>タイトル</th>
              <th>状況</th>
              <th>アクション</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="[id, editRequestWiki] in editRequestWikiList" v-bind:key="id">
              <td>{{ editRequestWiki.request_public_user_name }}</td>
              <td>{{ editRequestWiki.original_title }}</td>
              <td>{{ statusTable[editRequestWiki.status] }}</td>
              <td v-if="currenrUserId === editRequestWiki.wiki_owner_id">
                <button v-on:click="onOpenCloseDiffModal(
                  editRequestWiki.id,
                  `${editRequestWiki.original_title}\n\n${editRequestWiki.original_body}`,
                  `${editRequestWiki.edit_request_title}\n\n${editRequestWiki.edit_request_body}`,
                  )">確認する</button>
              </td>
              <td v-if="currenrUserId !== editRequestWiki.wiki_owner_id">
                <button v-on:click="disableEditRequest(editRequestWiki.id)">取り下げ</button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="btn-close">
        <button v-on:click="openCloseEditRequestWikis()">閉じる</button>
      </div>
    </div>
  </div>

  <!-- Diff表示モーダル -->
  <div
    id="overlay-diff"
    v-show="showDiffPreviewModal"
    role="dialog"
    aria-modal="true"
    aria-labelledby="diff-title"
    @click.self="onOpenCloseDiffModal('', '', '', true)"
  >
    <div id="content-diff">
      <header class="diff-header">
        <div class="diff-header__title">
          <h4 id="diff-title">差分比較</h4>
        </div>
        <button type="button" v-on:click="onOpenCloseDiffModal('', '', '', true)">
          閉じる
        </button>
      </header>

      <div class="diff-grid">
        <section class="diff-panel">
          <div class="diff-panel__head">
            <h3>元の内容</h3>
          </div>
          <div class="diff-panel__body" ref="originalAreaRef"></div>
        </section>

        <section class="diff-panel">
          <div class="diff-panel__head">
            <h3>更新リクエストの内容</h3>
          </div>
          <div class="diff-panel__body" ref="modifiedAreaRef"></div>
        </section>
      </div>

      <footer class="diff-footer">
        <div class="diff-footer__spacer"></div>
        <div class="diff-footer__actions">
          <button type="button" v-on:click="resultOwnerRequest(true)">却下</button>
          <button type="button" v-on:click="resultOwnerRequest(false)">承認</button>
        </div>
      </footer>
    </div>
  </div>

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

#overlay-edit-request-list {
  z-index: 10;
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

#content-edit-request-wikis {
  z-index: 11;
  width: 92%;
  padding: 1em;
  background: #fff;
  border-radius: 10px;
}

.table_sticky_edit_requests table {
  margin-top: 0;
  width: 99%;
}

.table_sticky_edit_requests thead {
  font-size: 9px;
}

.table_sticky_edit_requests td {
  font-size: 12px;
}

.table_sticky_edit_requests {
  display: block;
  overflow-y: auto;
  margin-bottom: 1%;
}

.table_sticky_edit_requests thead th:nth-child(1) {
  width: 10%;
}

.table_sticky_edit_requests thead th:nth-child(2) {
  width: 30%;
}

.table_sticky_edit_requests thead th:nth-child(3) {
  width: 5%;
}

.table_sticky_edit_requests thead th:nth-child(4) {
  width: 5%;
}

.table_sticky_edit_requests tbody td:nth-child(1),
.table_sticky_edit_requests tbody td:nth-child(4),
.table_sticky_edit_requests tbody td:nth-child(3) {
  text-align: center;
}

/* Diff表示モーダル */
#overlay-diff {
  z-index: 10;
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  padding: 16px;
}

#content-diff {
  width: min(1200px, 100%);
  height: min(92vh, 980px);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: var(--shadow);
  overflow: hidden; /* header/footer固定のため */
  display: grid;
  grid-template-rows: auto 1fr auto; /* header / body / footer */
  background: rgb(250, 250, 250);
  border-radius: 10px;
}

/* ===== Header ===== */
.diff-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 18px 14px;
  background: linear-gradient(to bottom, rgba(255, 255, 255, 0.98), rgba(255, 255, 255, 0.86));
  border-bottom: 1px solid var(--border);
}

.diff-header__title h2 {
  margin: 0;
  font-size: 20px;
  letter-spacing: 0.2px;
}

.diff-header__sub {
  margin: 4px 0 0;
  font-size: 12px;
  color: var(--muted);
}

/* ===== Body grid ===== */
.diff-grid {
  padding: 16px;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
  overflow: auto;
}

/* スマホは縦積み */
@media (max-width: 860px) {
  .diff-grid {
    grid-template-columns: 1fr;
  }
}

/* ===== Panel ===== */
.diff-panel {
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: 14px;
  overflow: hidden;

  display: grid;
  grid-template-rows: auto 1fr;
  min-height: 0; /* 子のoverflowを効かせる */
}

.diff-panel__head {
  padding: 12px 12px 10px;
  border-bottom: 1px solid var(--border);
  background: rgba(2, 6, 23, 0.02);
}

.diff-panel__head h3 {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: rgba(15, 23, 42, 0.82);
}

/* ===== Diff text area ===== */
.diff-panel__body {
  padding: 12px;
  overflow: auto;

  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.65;
  font-size: 14px;

  /* “コードビュー”っぽさ */
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}

/* ===== Footer ===== */
.diff-footer {
  padding: 12px 16px;
  border-top: 1px solid var(--border);
  background: rgba(255, 255, 255, 0.92);

  display: flex;
  align-items: center;
  justify-content: space-between;
}

.diff-footer__actions {
  display: flex;
  gap: 10px;
}
</style>