<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue';
import { AxiosError } from 'axios';
import { useRouter } from 'vue-router';
import { getUserUrl, postOwnerResultUrl, disableEditWikiUrl } from '@/router/urls';
import type { WikiData, QueryForm, OneTimeWikis, EditRequestWiki } from '@/interface';
import { assetsUrl, baseUrl } from '@/setting';
import { useWikiStore } from '@/stores/wikis';
import { useOnetimeWikiStore } from '@/stores/onetimeWikis';
import { useEditRequestWikiStore } from '@/stores/editWikis';
import apiClient from '@/axiosClient';
import UserPrivacySetting from '@/components/UserPrivacySetting.vue';
import { useMessageModal } from '@/utils/useMessageModal';
import { useProtocolDetection } from '@/utils/useProtocolDetection';

// アプリケーションの通信プロトコル
const { isHttpsProtocol } = useProtocolDetection();

// ホスト名
const { protocol, hostname, port: urlPort } = new URL(window.location.href);
const hostName = `${protocol}//${hostname}:${urlPort}`;

// Login.vueへのリダイレクト
const router = useRouter();
const loginRedirect = (): void => {
  router.push('/account/login');
};

// Preview.vueへのリダイレクト
const previewRedirect = (id: string): void => {
  localStorage.setItem('prev-table-data', `table-dataid-${id}`);
  router.push(`/wiki/preview/${id}`);
};

// Create.vueへのリダイレクト
const createRedirect = (): void => {
  router.push('/wiki/create');
};

// Wiki ストア
const wikiStore = useWikiStore();
const wikiList = computed((): Map<string, WikiData> => {
  return wikiStore.wikiList;
});
// wikiデータが存在しなければ再取得
if (wikiList.value.size === 0) {
  wikiStore.initList();
}

// 変更リクエストストア
const editRequestWikiStore = useEditRequestWikiStore();
const editRequestWikiList = computed((): Map<string, EditRequestWiki> => {
  return editRequestWikiStore.editRequestWikiList;
});

// 一時URL発行済みのWikiを取得
const onetimeWikiStore = useOnetimeWikiStore();
onetimeWikiStore.initList();
const onetimeWikiList = computed((): Map<string, OneTimeWikis> => {
  return onetimeWikiStore.onetimeWikiList;
});

// 共有停止
const invalidShareUrl = async (id: string, title: string): Promise<void> => {
  try {
    await onetimeWikiStore.deleteOnetimeWiki(id);
    messageModalOpenClose(`「${title}」 の共有を停止しました。`);
  } catch (error) {
    console.error(error);
  }
};

// クエリを行うテキストボックスの初期値
const queryFormDataInit: QueryForm = {
  query1: '',
  query2: '',
};
const queryFormData = ref(queryFormDataInit);

watch(queryFormData.value, (): void => {
  onSearch();
});

// 検索実行関数
const onSearch = (reset: boolean = false): void => {
  localStorage.setItem('prev-table-data', '');
  try {
    if (reset) {
      wikiStore.queryWiki('', '');
    } else {
      wikiStore.queryWiki(queryFormData.value.query1, queryFormData.value.query2);
    }
  } catch (error) {
    console.log(error);
  }
};

// 現在ユーザーの取得
const currentUser = ref('');
const currenrUserId = ref('');
const getCurrentUser = async (): Promise<void> => {
  try {
    const response = await apiClient.get(getUserUrl);
    currentUser.value = response.data['public_name'];
    currenrUserId.value = response.data['id'];
  } catch (error) {
    loginRedirect();
  }
};
getCurrentUser();

// 共有リンク発行済みWikiの確認
const isOpenShareWikis = ref(false);
const isOpenShareWikisHttp = ref(false);
const openCloseOnetimeUrls = (): void => {
  // HTTPS OR LOCALHOST
  if (isHttpsProtocol.value) {
    if (isOpenShareWikis.value) {
      isOpenShareWikis.value = false;
    } else {
      onetimeWikiStore.initList();
      isOpenShareWikis.value = true;
    }

    // HTTP
  } else {
    if (isOpenShareWikisHttp.value) {
      isOpenShareWikisHttp.value = false;
    } else {
      onetimeWikiStore.initList();
      isOpenShareWikisHttp.value = true;
    }
  }
};

// 更新リクエスト一覧モーダル
const isOpenEditRequestWikis = ref(false);
const openCloseEditRequestWikis = (): void => {
  editRequestWikiStore.initList();
  if (isOpenEditRequestWikis.value) {
    isOpenEditRequestWikis.value = false;
  } else {
    isOpenEditRequestWikis.value = true;
  }
};

// 更新リクエストステータス対応
const statusTable = {
  REJECT: '却下',
  REQUESTNOW: '申請中',
  DRAFT: '下書き',
  APPLIED: '承認',
};

// Diff表示モーダルの表示非表示
const showDiffPreviewModal = ref(false);
const clickedRequestWikiId = ref('');

const originalAreaRef = ref<HTMLElement | null>(null);
const modifiedAreaRef = ref<HTMLElement | null>(null);

const onOpenCloseDiffModal = async (
  id: string,
  text1: string,
  text2: string,
  isClose: boolean = false,
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

    const span = document.createElement('span');
    span.textContent = text;

    switch (operation) {
      case -1:
        span.classList.add('delete');
        containerOriginal.appendChild(span);
        break;
      case 1:
        span.classList.add('added');
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
    id: clickedRequestWikiId.value,
    reject: isReject,
  };

  try {
    await apiClient.post(postOwnerResultUrl, payload);
    onOpenCloseDiffModal('', '', '', true);
    editRequestWikiStore.initList();
    wikiStore.initList();

    if (isReject) {
      messageModalOpenClose('却下しました。');
    } else {
      messageModalOpenClose('承認しました。');
    }
  } catch (error) {
    if (apiClient.isAxiosError(error)) {
      // エラーオブジェクトがAxiosError型であることが保証
      const axiosError = error as AxiosError<any>;
      const errorStatusCode = axiosError.response?.status;
      if (errorStatusCode === 404) {
        messageModalOpenClose('すでに申請者が取り下げた申請のため、変更は適用されませんでした。');
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
  try {
    await apiClient.delete(url);
    onOpenCloseDiffModal('', '', '', true);
    editRequestWikiStore.initList();
    messageModalOpenClose('更新申請を取り下げました。');
  } catch (error) {
    if (apiClient.isAxiosError(error)) {
      // エラーオブジェクトがAxiosError型であることが保証
      const axiosError = error as AxiosError<any>;
      const errorStatusCode = axiosError.response?.status;
      if (errorStatusCode === 404) {
        messageModalOpenClose('既に承認または取り下げが行われています。');
        showDiffPreviewModal.value = false;
        editRequestWikiStore.initList();
        return;
      }
    }
  }
};

// 更新日時を取得
const getUpdateAt = (dateStr: string, datetimeStr: string): string => {
  // 作成日と更新日時を比較
  if (areDatesSame(dateStr, datetimeStr)) {
    return '';
  }
  return ` 【更新：${formatDateJP(datetimeStr)} 】`;
};

// 日付時刻から日付のみを取り出す関数
function getDateForDateTime(dateTimeString: string): string {
  return dateTimeString.split('T')[0]!;
}

// 日付を比較する関数（同日編集の場合は対象外とする仕様）
function areDatesSame(dateString1: string, dateString2: string): boolean {
  // dateString1と2から日付部分だけを取り出す
  const datePartOfDateTime1 = getDateForDateTime(dateString1);
  const datePartOfDateTime2 = getDateForDateTime(dateString2);

  // 日付を比較する
  return datePartOfDateTime1 === datePartOfDateTime2;
}

// ショートカットキーを追加
const handleKeyDown = (event: KeyboardEvent) => {
  // Create.vueへ移動
  if (event.ctrlKey && event.key === '1') {
    event.preventDefault(); // デフォルトのブラウザのショートカットをキャンセル
    createRedirect();

    // 共有リンク発行Wiki一覧モーダルを表示
  } else if (event.ctrlKey && event.key === '2') {
    event.preventDefault();
    openCloseOnetimeUrls();
  } else if (event.ctrlKey && event.key === '3') {
    event.preventDefault();
    openCloseEditRequestWikis();

    // ユーザーのプライバシーセッティングモーダルを表示非表示
  } else if (event.ctrlKey && event.key === '4') {
    event.preventDefault();
    userPrivacySettingFunction();

    // 検索ワード1をフォーカス
  } else if (event.ctrlKey && event.key === '5') {
    event.preventDefault();
    const textElement = document.getElementById('query1');
    if (textElement) {
      textElement.focus();
    }
    // 検索ワード2をフォーカス
  } else if (event.ctrlKey && event.key === '6') {
    event.preventDefault();
    const textElement = document.getElementById('query2');
    if (textElement) {
      textElement.focus();
    }
    // 検索をクリア
  } else if (event.ctrlKey && event.key === '7') {
    event.preventDefault();
    onSearch(true);
  }
};

// コンポーネントマウント時にイベントリスナーを追加
onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});

// コンポーネントがアンマウントされた際にイベントリスナーを削除
onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

// テーブルスクロール
const scrolledTableId = localStorage.getItem('prev-table-data');
onMounted(() => {
  if (scrolledTableId) {
    if (scrolledTableId != '') {
      let targetTableRowIdElm = document.getElementById(scrolledTableId);
      targetTableRowIdElm?.scrollIntoView({
        block: 'start',
      });
      let tableElm = document.getElementById('wiki-table');
      if (scrolledTableId !== 'table-dataid-1') {
        tableElm?.scrollBy({
          top: -50,
        });
      }
    }
  }
});

// テーブルのソート
function onSort() {
  wikiStore.sortWiki();
}

// ワンタイムフルURLの取得
function getFullOnetimeUrl(url: string): string {
  return `${hostName}${url}`;
}

// ISO形式の日付を変換
function formatDateJP(
  isoString: string,
  isDayOnly: boolean = false,
  isJpFormat: boolean = false,
  addMinutes: number = 540,
): string {
  const date = new Date(isoString);

  // UTC時間を取得し、9時間（540分）加算してJSTに変換
  date.setMinutes(date.getMinutes() + addMinutes);

  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0'); // 月は0始まりのため+1
  const day = String(date.getDate()).padStart(2, '0');
  const hours = String(date.getHours()).padStart(2, '0');
  const minutes = String(date.getMinutes()).padStart(2, '0');

  if (isDayOnly) {
    if (isJpFormat) {
      return `${year}年${month}月${day}日`;
    } else {
      return `${year}-${month}-${day}`;
    }
  } else {
    if (isJpFormat) {
      return `${year}年${month}月${day}日 ${hours}時${minutes}分`;
    } else {
      return `${year}-${month}-${day} ${hours}:${minutes}`;
    }
  }
}

// 期限切れでないか検証
function isExpired(isoString: string): boolean {
  const serverTime = new Date(isoString);
  serverTime.setMinutes(serverTime.getMinutes() + 540);
  return serverTime.getTime() < Date.now();
}

// 与えられたelement idのテキストに次の処理
// HTTPS（localhost）プロトコル下ではクリップボードコピー HTTPではテキスト選択（IEは非対応）
function selectTextOrClipboardCopy(elementId: string) {
  let element = document.getElementById(elementId);
  if (!element || !element.textContent) {
    return;
  }

  if (isHttpsProtocol.value) {
    navigator.clipboard.writeText(element.textContent);
    messageModalOpenClose('クリップボードにコピーしました。');
  } else {
    if (window.getSelection) {
      let selection = window.getSelection();
      let range = document.createRange();
      try {
        range.selectNodeContents(element);
      } catch (e) {
        console.error(`Error selecting contents of element: ${e}`);
      }
      if (selection) {
        selection.removeAllRanges(); // 現在の選択をクリア
        selection.addRange(range); // 新しい範囲を選択
      }
    }
  }
}

// メッセージ表示モーダル機能
const { isMessageModal, messageText, messageModalOpenClose } = useMessageModal();

const userSettingModalRef = ref<{
  openCloseUserSettingModal: () => void;
  isUserPrivate: boolean;
  isInitialized: boolean;
} | null>(null);

const userPrivacySettingFunction = () => {
  if (userSettingModalRef.value) {
    userSettingModalRef.value.openCloseUserSettingModal();
  }
};

watch(
  () => userSettingModalRef.value?.isUserPrivate,
  (newValue, oldValue) => {
    if (!userSettingModalRef.value?.isInitialized) return; // 子コンポーネントでモーダルを起動していない場合は発火しない
    if (newValue) {
      messageModalOpenClose(
        'プライバシーモードが ON になりました。他のユーザーはあなたのデータにアクセスできません。',
      );
    } else {
      messageModalOpenClose(
        'プライバシーモードが OFF になりました。 他のユーザーにあなたの画像などをシェアすることができます。',
      );
    }
  },
);
</script>

<template>
  <!-- ユーザー設定変更モーダル -->
  <UserPrivacySetting ref="userSettingModalRef"></UserPrivacySetting>

  <div class="head-btn-and-search">
    <div class="btn-head-left">
      <button
        class="btn-head-image"
        title="Wiki作成画面へ遷移します。&#10;ショートカット: Ctrl + 1"
        v-on:click="createRedirect()"
      >
        <img :src="`${assetsUrl}add_fill24.png`" class="btn-img" alt="add_fill24.png" />
      </button>
      <button
        class="btn-head-image"
        title="共有済みWiki一覧&#10;ショートカット: Ctrl + 2"
        v-on:click="openCloseOnetimeUrls()"
      >
        <img :src="`${assetsUrl}family_line24.png`" class="btn-img" alt="family_line24.png" />
      </button>
      <button
        class="btn-head-image"
        title="更新リクエスト&#10;ショートカット: Ctrl + 3"
        v-on:click="openCloseEditRequestWikis()"
      >
        <img
          :src="`${assetsUrl}edit_notifications_24.png`"
          class="btn-img"
          alt="edit_notifications_24.png"
        />
      </button>
      <button
        v-on:click="userPrivacySettingFunction()"
        class="btn-head-image"
        title="ユーザー設定&#10;アカウントのプライバシー設定を変更します。&#10;ショートカット: Ctrl + 4"
      >
        <img
          :src="`${assetsUrl}manage_accounts_24.png`"
          class="btn-img"
          alt="manage_accounts_24.png"
        />
      </button>
    </div>
    <div class="search-area">
      <div class="form-area">
        <input
          type="text"
          class="query1"
          id="query1"
          title="1つ目の検索ワード&#10;ショートカット: Ctrl + 5"
          placeholder="検索ワード1"
          v-model="queryFormData.query1"
        />
        <input
          type="text"
          class="query2"
          id="query2"
          title="2つ目の検索ワード&#10;ショートカット: Ctrl + 6"
          placeholder="検索ワード2"
          v-model="queryFormData.query2"
        />
      </div>
      <div class="form-btn-area">
        <button
          class="btn-head-search"
          title="検索結果をクリア（作成日時でソート）&#10;ショートカット: Ctrl + 7"
          type="submit"
          v-on:click="onSearch(true)"
        >
          <img :src="`${assetsUrl}update_fill24.png`" class="btn-img" alt="update_fill24.png" />
        </button>
      </div>
    </div>
  </div>

  <div class="table_sticky" id="wiki-table">
    <table>
      <thead>
        <tr>
          <th v-on:click="onSort()" title="クリックにより更新日時でソート">Wiki</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="[id, wiki] in wikiList" v-bind:key="id">
          <td
            tabindex="0"
            :id="`table-dataid-${id}`"
            v-on:click="previewRedirect(id)"
            v-on:keydown.enter="previewRedirect(id)"
            :class="['pointer', { 'is-public': wiki.is_public, '': !wiki.is_public }]"
          >
            <div class="td-text">
              <div class="td-time-area">
                <div>{{ formatDateJP(wiki.date) }}</div>
                <div>{{ getUpdateAt(wiki.date, wiki.update_at) }}</div>
              </div>
              <div
                :class="{
                  'td-title-area tooltip': wiki.is_public,
                  'td-title-area': !wiki.is_public,
                }"
              >
                <div>{{ wiki.title }}</div>
                <span v-if="wiki.is_public" class="tooltiptext">パブリックWiki</span>
              </div>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
  </div>

  <!-- 共有済みWikiモーダル（https or localhost）-->
  <div id="overlay-onetimewiki-https-list" v-show="isOpenShareWikis">
    <div id="content-https-wikis" :style="{ width: onetimeWikiList.size === 0 ? '40%' : '73%' }">
      <h2 class="modal-h2">共有URL発行済みWiki</h2>
      <div v-if="onetimeWikiList.size === 0">
        <p style="text-align: center">共有中のWikiはありません。</p>
      </div>
      <div v-else class="table_sticky_onetime">
        <table>
          <thead>
            <tr>
              <th>Exp</th>
              <th>Title</th>
              <th>Url</th>
              <th>ShareStop</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="[id, onetimewiki] in onetimeWikiList" v-bind:key="id">
              <td
                v-if="isExpired(onetimewiki.expiration)"
                class="expired"
                title="期限切れのURLです。"
              >
                {{ formatDateJP(onetimewiki.expiration, false, true) }}
              </td>
              <td v-else="isExpired(onetimewiki.expiration)">
                {{ formatDateJP(onetimewiki.expiration, false, true) }}
              </td>

              <td v-if="isExpired(onetimewiki.expiration)" class="expired">
                {{ onetimewiki.title }}
              </td>
              <td v-else="isExpired(onetimewiki.expiration)">{{ onetimewiki.title }}</td>

              <td v-if="isExpired(onetimewiki.expiration)" :id="id" class="expired">
                {{ getFullOnetimeUrl(onetimewiki.url) }}
              </td>
              <td
                v-else="isExpired(onetimewiki.expiration)"
                v-on:click="selectTextOrClipboardCopy(id)"
                :id="id"
              >
                {{ getFullOnetimeUrl(onetimewiki.url) }}
              </td>

              <td v-if="isExpired(onetimewiki.expiration)">
                <button
                  v-on:click="invalidShareUrl(id, onetimewiki.title)"
                  class="btn-onetimewiki-stop-share"
                >
                  削除
                </button>
              </td>
              <td v-else="isExpired(onetimewiki.expiration)">
                <button
                  v-on:click="invalidShareUrl(id, onetimewiki.title)"
                  class="btn-onetimewiki-stop-share"
                >
                  共有停止
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="btn-close">
        <button v-on:click="openCloseOnetimeUrls()">閉じる</button>
      </div>
    </div>
  </div>

  <!-- 共有済みWikiモーダル（http） -->
  <div id="overlay-onetimewiki-http-list-no-data" v-show="isOpenShareWikisHttp">
    <div id="content-http-wikis" :style="{ width: onetimeWikiList.size === 0 ? '40%' : '73%' }">
      <h2 class="modal-h2">共有URL発行済みWiki</h2>
      <div v-if="onetimeWikiList.size === 0"><p>共有中のWikiはありません。</p></div>
      <div v-else class="table_sticky_onetime">
        <table>
          <thead>
            <tr>
              <th>Exp</th>
              <th>Title</th>
              <th>Url</th>
              <th>ShareStop</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="[id, onetimewiki] in onetimeWikiList" v-bind:key="id">
              <td
                v-if="isExpired(onetimewiki.expiration)"
                class="expired"
                title="期限切れのURLです。"
              >
                {{ formatDateJP(onetimewiki.expiration) }}
              </td>
              <td v-else="isExpired(onetimewiki.expiration)">
                {{ formatDateJP(onetimewiki.expiration) }}
              </td>

              <td v-if="isExpired(onetimewiki.expiration)" class="expired">
                {{ onetimewiki.title }}
              </td>
              <td v-else="isExpired(onetimewiki.expiration)">{{ onetimewiki.title }}</td>

              <td v-if="isExpired(onetimewiki.expiration)" :id="id" class="expired">
                {{ getFullOnetimeUrl(onetimewiki.url) }}
              </td>
              <td
                v-else="isExpired(onetimewiki.expiration)"
                v-on:click="selectTextOrClipboardCopy(id)"
                :id="id"
              >
                {{ getFullOnetimeUrl(onetimewiki.url) }}
              </td>

              <td v-if="isExpired(onetimewiki.expiration)">
                <button
                  v-on:click="invalidShareUrl(id, onetimewiki.title)"
                  class="btn-onetimewiki-stop-share"
                >
                  削除
                </button>
              </td>
              <td v-else="isExpired(onetimewiki.expiration)">
                <button
                  v-on:click="invalidShareUrl(id, onetimewiki.title)"
                  class="btn-onetimewiki-stop-share"
                >
                  共有停止
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="btn-close">
        <button v-on:click="openCloseOnetimeUrls()">閉じる</button>
      </div>
    </div>
  </div>

  <!-- 更新リクエストWikiモーダル-->
  <div id="overlay-edit-request-list" v-show="isOpenEditRequestWikis">
    <div
      id="content-edit-request-wikis"
      :style="{ width: editRequestWikiList.size === 0 ? '40%' : '73%' }"
    >
      <h2 class="modal-h2">更新リクエスト一覧</h2>
      <div v-if="editRequestWikiList.size === 0">
        <p style="text-align: center">申請中及び受け付けた変更リクエストはありません。</p>
      </div>
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
              <td
                v-if="currenrUserId === editRequestWiki.wiki_owner_id"
                v-on:click="
                  messageModalOpenClose(
                    `${editRequestWiki.request_public_user_name}：${editRequestWiki.request_message}`,
                  )
                "
                style="cursor: pointer"
              >
                {{ editRequestWiki.original_title }}
              </td>
              <td v-else>{{ editRequestWiki.original_title }}</td>
              <td>{{ statusTable[editRequestWiki.status] }}</td>
              <td v-if="currenrUserId === editRequestWiki.wiki_owner_id">
                <button
                  v-on:click="
                    onOpenCloseDiffModal(
                      editRequestWiki.id,
                      `${editRequestWiki.original_title}\n\n${editRequestWiki.original_body}`,
                      `${editRequestWiki.edit_request_title}\n\n${editRequestWiki.edit_request_body}`,
                    )
                  "
                >
                  確認する
                </button>
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
          <h2 id="diff-title">差分比較</h2>
          <p class="diff-header__sub">元の内容 / 更新リクエストの内容</p>
        </div>
        <button type="button" v-on:click="onOpenCloseDiffModal('', '', '', true)">閉じる</button>
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
        <p>
          <strong>{{ messageText }}</strong>
        </p>
      </div>
      <div class="btn-close">
        <button id="message-close-btn" v-on:click="messageModalOpenClose('No Message')">
          閉じる
        </button>
      </div>
    </div>
  </div>

  <footer>
    <p class="login-user">ログインユーザー：{{ currentUser }}</p>
  </footer>
</template>

<style scoped>
.head-btn-and-search {
  display: flex;
  justify-content: space-between;
}

.btn-head-search {
  width: 50px;
  height: 35px;
  background: white;
  box-shadow: 3px 3px 5px 0 rgba(75, 75, 75, 0.5);
  color: #fff;
  padding: 5px 7px;
  text-decoration: none;
  border: 1px solid rgb(207, 207, 207);
  border-radius: 14px;
  transition-property: opacity;
  -webkit-transition-property: opacity;
  transition-duration: 0.5s;
  -webkit-transition-duration: 0.5s;
  transition: background-color 0.3s;
  margin: 5px 5px 10px 5px;
}

.btn-head-search:hover {
  background: rgb(235, 235, 235);
}

.btn-zone {
  margin-top: 10px;
  margin-bottom: 5px;
  display: flex;
  justify-content: space-between;
}

.form-btn-area {
  display: flex;
}

.search-area {
  display: flex;
}

.form-area {
  display: flex;
  margin-top: 5px;
}

.query1 {
  margin-right: 2%;
  font-size: 20px;
  border-radius: 6px;
}

.query2 {
  margin-right: 3%;
  font-size: 20px;
  border-radius: 6px;
}

.query1,
.query2 {
  display: inline-block;
  height: 35px;
  width: 110%;
  box-sizing: border-box;
  border-radius: 5px;
  text-align: center;
}

/* テーブルのホバー：ボディ部分の行のみホバー時のスタイルを適用 */
table tbody tr:hover {
  background-color: #bfdbe6;
}

.table_sticky table {
  margin-top: 0;
  width: 100%;
}

.table_sticky {
  display: block;
  overflow-y: auto;
  height: 70vh;
  margin-top: 1%;
}

.table_sticky thead th {
  position: sticky;
  font-size: 18px;
  top: 0;
  width: 100%;
  z-index: 1;
  background: rgb(44, 52, 78);
  color: whitesmoke;
}

.td-time-area {
  justify-content: space-between;
  display: flex;
}

.td-title-area {
  font-size: 16px;
  display: flex;
  justify-content: space-between;
}

.is-public:hover {
  background-color: #c0dbcd;
}

.login-user {
  position: fixed;
  bottom: 1px;
  right: 1%;
  text-align: right;
  font-size: 14px;
  font-weight: bold;
  text-shadow: 1px 1px 2px rgb(202, 202, 202);
}

.switch-label {
  position: relative;
  display: flex;
  align-items: center;
}

input[type='checkbox'] {
  position: absolute;
  width: 0;
  height: 0;
  opacity: 0;
}

.base {
  width: 42px;
  border-radius: 12px;
  height: 24px;
  background-color: rgb(219, 234, 254);
}

.switch-title {
  margin-left: 3px;
  font-size: 12px;
}

input:checked ~ .base {
  background-color: rgb(219, 234, 254);
  transition: 0.5s;
}

input:checked ~ .circle {
  transform: translateX(100%);
  background-color: blue;
}

.circle {
  position: absolute;
  top: 3.5px;
  left: 3.5px;
  width: 16px;
  height: 16px;
  border-radius: 8px;
  background-color: rgb(23, 168, 72);
  transition: 0.5s;
}

.switch {
  position: relative;
}

/* カスタムツールチップ */
/* ツールチップコンテナ */
.tooltip {
  position: relative;
  display: inline-block;
  width: 100%;
}

/* ツールチップテキスト */
.tooltip .tooltiptext {
  visibility: hidden;
  width: 120px;
  background-color: rgb(12, 185, 56);
  color: #fff;
  text-align: center;
  border-radius: 6px;
  padding: 5px 0;

  /* ツールチップの位置を調整 */
  position: absolute;
  z-index: 1;
  bottom: 100%;
  left: 85%;
  margin-left: -60px;

  /* ホバー時のアニメーション */
  opacity: 0;
  transition: opacity 0.5s;
}

/* 要素にホバーしたときにツールチップを表示 */
.tooltip:hover .tooltiptext {
  visibility: visible;
  opacity: 1;
}

/* 各種モーダル */
#overlay-onetimewiki-http-list,
#overlay-onetimewiki-https-list,
#overlay-edit-request-list {
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

/* 各種モーダルのコンテンツ */
#content-https-wikis,
#content-http-wikis,
#content-edit-request-wikis {
  z-index: 2;
  padding: 1em;
  background: #fff;
  border-radius: 10px;
}

.table_sticky_edit_requests table {
  margin-top: 0;
  width: 100%;
}

.table_sticky_edit_requests {
  display: block;
  overflow-y: auto;
  margin-bottom: 1%;
}

.table_sticky_edit_requests thead th:nth-child(1) {
  width: 5%;
}

.table_sticky_edit_requests thead th:nth-child(2) {
  width: 40%;
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

.table_sticky_onetime table {
  margin-top: 0;
  width: 100%;
}

.table_sticky_onetime {
  display: block;
  overflow-y: auto;
  height: 40vh;
  margin-bottom: 1%;
}

.table_sticky_onetime td {
  padding: 1px 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 5px;
}

.expired {
  color: #dd1010;
}

.table_sticky_edit_requests thead th {
  text-align: center;
  position: sticky;
  font-size: 14px;
  top: 0;
  width: 100%;
  z-index: 1;
  background: rgb(44, 52, 78);
  color: whitesmoke;
}

.table_sticky_onetime thead th {
  text-align: center;
  position: sticky;
  font-size: 14px;
  top: 0;
  width: 100%;
  z-index: 1;
  background: rgb(44, 52, 78);
  color: whitesmoke;
}

.table_sticky_onetime thead th:nth-child(1) {
  width: 15%;
}

.table_sticky_onetime thead th:nth-child(2) {
  width: 15%;
}

.table_sticky_onetime thead th:nth-child(3) {
  width: 30%;
}

.table_sticky_onetime thead th:nth-child(4) {
  width: 5%;
}

.table_sticky_onetime tbody td:nth-child(1),
.table_sticky_onetime tbody td:nth-child(4),
.table_sticky_onetime tbody td:nth-child(3) {
  text-align: center;
}

/* メッセージモーダル */
#overlay-message {
  z-index: 3;
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
  text-align: center;
}

.btn-onetimewiki-list-close,
#message-close-btn {
  width: 70px;
  background: gray;
  box-shadow: 3px 3px 5px 0 rgba(75, 75, 75, 0.5);
  color: #fff;
  padding: 10px 7px;
  text-decoration: none;
  border: 1px;
  border-radius: 5px;
  transition-property: opacity;
  -webkit-transition-property: opacity;
  transition-duration: 0.5s;
  -webkit-transition-duration: 0.5s;
  transition: background-color 0.3s;
  margin: 5px 5px 10px 5px;
}

.btn-onetimewiki-list-close:hover,
#message-close-btn:hover {
  background: rgb(90, 90, 90);
}

.btn-onetimewiki-stop-share {
  width: 70px;
  font-size: 12px;
  background: gray;
  box-shadow: 3px 3px 5px 0 rgba(75, 75, 75, 0.5);
  color: #fff;
  padding: 10px 7px;
  text-decoration: none;
  border: 1px;
  border-radius: 8px;
  transition-property: opacity;
  -webkit-transition-property: opacity;
  transition-duration: 0.5s;
  -webkit-transition-duration: 0.5s;
  transition: background-color 0.3s;
  margin: 5px 5px 10px 5px;
}

.btn-onetimewiki-stop-share:hover {
  background: rgb(56, 56, 56);
}

/* Diff表示モーダル */
#overlay-diff {
  z-index: 1;
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
  font-family:
    ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New',
    monospace;
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
