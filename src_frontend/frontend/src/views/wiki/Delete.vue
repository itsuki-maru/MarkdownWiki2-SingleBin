<script setup lang="ts">
import { marked, Renderer } from "marked";
import type { Tokens, MarkedOptions } from "marked";
import type { deleteWikiData, WikiData } from "@/interface";
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import { deleteWikiUrl, wikiOwnerGetUrl, getUserUrl } from "@/router/urls";
import { assetsUrl } from "@/setting";
import { AxiosError } from "axios";
import { useWikiStore } from "@/stores/wikis";
import { FilterXSS, getDefaultWhiteList } from "xss";
import type { IFilterXSSOptions } from "xss";
import { 
  videoToken,
  detailsToken,
  noteToken,
  warningToken,
  mathExtentionToken,
  youtubeToken,
  renderIframe
} from "@/utils/markedSetup";
import apiClient from "@/axiosClient";
import "katex/dist/katex.min.css";

const mermaid: any = (window as any).mermaid;

// Mermaidの初期読み込みを阻止（MarkedによるHTMLレンダリング後にinitで読み込み）
mermaid.initialize({startOnLoad: false});

// markedのスラッグ化機能をカスタマイズ
const renderer = new Renderer();
let headingIndex = -1; // 見出しのインデックス
renderer.heading = function (tokens: Tokens.Heading) {
  const id = `heading-${headingIndex++}`; // インデックスに基づいてIDを生成
  return `<h${tokens.depth} class="head${tokens.depth}">${tokens.text}</h${tokens.depth}>\n`; // class属性のCSSはトップレベル（App.vue）で定義
};

// [テキスト](URL)で定義された外部リンクを別タブで開かせるカスタムレンダラ設定
// 元のlink関数を保存
const originalLinkRenderer = renderer.link.bind(renderer);

// ローカルホスト判定
function isLocalhost(url: string) {
  try {
    const parsedUrl = new URL(url);
    return parsedUrl.hostname === "localhost" || parsedUrl.hostname === "127.0.0.1" || parsedUrl.hostname === "[::1]";
  } catch (e) {
    return false;
  }
}

// link関数をオーバーライド
renderer.link = (tokens: Tokens.Link) => {
  // 外部リンクかどうかをチェック
  const isExternal = /^https?:\/\//.test(tokens.href!);
  let isLocal = false;
  let isPDFHref = false;
  if (tokens.href) {
    isLocal = isLocalhost(tokens.href);
    isPDFHref = isPDF(tokens.href);
  }
  const html = originalLinkRenderer(tokens);
  if (isExternal) {
    if (isLocal && isPDFHref) {
      return html.replace(/^<a /, '<a target="_blank" rel="noopener noreferrer" title="PDFリンク" ');
    }
    // 外部リンクの場合、targetとrel属性を追加
    return html.replace(/^<a /, '<a target="_blank" rel="noopener noreferrer" title="外部リンク" ');
  } else {
    // 内部リンクかつPDFの場合
    if (isPDFHref) {
      return html.replace(/^<a /, '<a target="_blank" rel="noopener noreferrer" title="PDFリンク" ');
    }
    // 内部リンクの場合、元の処理を使用
    return originalLinkRenderer(tokens);
  }
};

// mermaidの処理
const originalCodeRenderer = renderer.code.bind(renderer);
renderer.code = (tokens: Tokens.Code) => {
  let html = originalCodeRenderer(tokens);
  if (tokens.lang == "mermaid") {
    return '<pre class="mermaid">' + escapeHtml(tokens.text) + '\n</pre>';
  } else {
    return originalCodeRenderer(tokens);
  }
}

const originalImageRenderer = renderer.image;
renderer.image = (tokens: Tokens.Image) => {
  let width = "";
  let href = tokens.href;
  let text = tokens.text;
  const match = tokens.href.match(/\s*=(\d+)(x)?$/);
  if (match) {
    width = match[1];
    href  = href.replace(/\s*=.*$/, "");
  }
  const widthAttr = width ? ` width="${width}px"` : "";
  return `<img src="${href}" alt="${text}" ${widthAttr}>`;
};

// HTMLエスケープ関数
function escapeHtml(html: string) {
  return html
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

// Markedにカスタムトークンを追加
marked.use(
  {
    extensions: [
      videoToken,
      detailsToken,
      noteToken,
      warningToken,
      mathExtentionToken,
      youtubeToken,
    ],
  }
);


// markedの設定をカスタマイズ
marked.setOptions({
  renderer,
  async: false
});

// XSSフィルタの設定をカスタマイズする
let xssOptions: IFilterXSSOptions = {
  whiteList: {
    ...getDefaultWhiteList(),　// デフォルトの許可リストを維持
    h1: ['id', 'class'], // h1-h6タグのid属性を許可 h1-h2のclass属性を許可
    h2: ['id', 'class'],
    h3: ['id'],
    h4: ['id'],
    h5: ['id'],
    h6: ['id'],
    pre: ['class'],
    a: ['target', 'rel', 'href', 'title'],
    div: ['class'],
    span: ['class', 'aria-hidden', 'style'],
    "app-youtube": ['video-id', 'data-src']
  },
  // iframeの確認（念のため、iframeはここで不許可）
  onTag(tag, html) {
    if (tag === "iframe") return "Not Allow iframe ";
  },
  // Katexでサニタイズされてしまうスタイルを再定義
  css: {
    whiteList: {
      height: true,
      'margin-right': true,
      top: true,
      width: true,
      'margin-left': true,
      left: true, right: true, bottom: true,
    }
  }
};
const myXss = new FilterXSS(xssOptions);

// List.vueへリダイレクト
const router = useRouter();
const listRedirect = (): void => {
  router.push("/wiki/list");
}

// Login.vueへのリダイレクト
const loginRedirect = (): void => {
  router.push("/account/login");
}

// Preview.vueへのリダイレクト
const previewRedirect = (id: string): void => {
  router.push(`/wiki/preview/${id}`);
}

interface Props {
  id: string;
}

const props = defineProps<Props>();
const wikiStore = useWikiStore();
const wiki = computed(
  (): WikiData => {
    return wikiStore.getById(props.id);
  }
);

const deletewikiDataInit: deleteWikiData = {
  id: wiki.value.id,
  title: wiki.value.title,
  body: wiki.value.body,
  is_public: wiki.value.is_public,
};

const deletewikiData = ref(deletewikiDataInit);


// マークダウンへのパース処理
const textTitleData = "# " + wiki.value.title + "\n\n";
const textBodyData = wiki.value.body;
const markdownData = textTitleData + textBodyData;
const options: MarkedOptions = { async: false };
const htmlStr = marked.parse(markdownData, options);
const cleanHtml = myXss.process(htmlStr as string);
const renderHtml = renderIframe(cleanHtml);
const bindHtml = ref(renderHtml);

// 削除処理
const deleteWiki = async (): Promise<void> => {
  const id = deletewikiData.value.id;
  const title = deletewikiData.value.title;
  const body = deletewikiData.value.body;

  try {
    const response = await apiClient.delete(
      deleteWikiUrl + `/${id}`,
    );
    const deleteData = {
      id: id,
      title: title,
      body: body
    }
    const wikiStore = useWikiStore();
    wikiStore.deleteWiki(deleteData.id);
    showContent.value = false;
    isDeleteOkModal.value = true;

  } catch (error: unknown) {
    if (typeof error === "object" && error !== null) {
      const axiosError = error as AxiosError;

      if (axiosError.response) {
        console.log("Status code:", axiosError.response.status);
        console.log("Error data:", axiosError.response.data);
        if (axiosError.response.status === 401) {
          messageModalOpenClose("不正な操作です。\nオーナーでないデータは削除できません。");
          localStorage.setItem("loginUser", "");
          return;
        }
      } else if (axiosError.request) {
        console.log("No response was received", axiosError.request);
      } else {
        console.log("Error", axiosError.message);
      }
    } else {
      console.log("An unknown error occurred.");
    }
  }
}

// 削除画面であることの注意喚起モーダル
const isDeleteModal = ref(true);
const onIsDeleteModal = (res: number): void => {
  if (res === 1) {
    listRedirect();
  } else {
    isDeleteModal.value = false;
  }
}

// 削除確認モーダル
const showContent = ref(false);
const onDeleteCheck = (): void => {
  showContent.value = true;
}

// 削除の実行かキャンセル
const onCloseModal = (res: number): void => {
  if (res === 1) {
    localStorage.setItem("prev-table-data", "");
    deleteWiki();
  } else {
    showContent.value = false;
  }
};

// 削除完了モーダル
const isDeleteOkModal = ref(false);

//** Wikiデータのオーナー取得 */
const wikiOwner = ref("");
const isOwner = ref(false);
const getWikiOwner = async (id: string): Promise<void> => {
  try {
    const response = await apiClient.get(
      wikiOwnerGetUrl + `/${id}`,
    );
    wikiOwner.value = response.data["WikiOwner"];
    // Wikiオーナーとログインユーザーが一致しているか確認
    if (localStorage.getItem("loginUser") === wikiOwner.value) {
      isOwner.value = true;
    }
  } catch (error) {
    console.log("Owner Get Error");
    loginRedirect();
  }
};
getWikiOwner(props.id);

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

// ショートカットキーを追加
const handleKeyDown = (event: KeyboardEvent) => {
  // List.vueへ移動
  if (event.ctrlKey && event.key === "1") {
    event.preventDefault(); // デフォルトのブラウザのショートカットをキャンセル
    listRedirect();
    // Preview.vueへ移動
  } else if (event.ctrlKey && event.key === "2") {
    event.preventDefault();
    previewRedirect(wiki.value.id);
  }
};

// コンポーネントマウント時にイベントリスナーを追加し、mermaid.jsを発動
onMounted(() => {
  window.addEventListener("keydown", handleKeyDown);
  mermaid.init();
});

// コンポーネントがアンマウントされた際にイベントリスナーを削除
onUnmounted(() => {
  window.removeEventListener("keydown", handleKeyDown);
});

// 拡張子でPDFファイルか判定する関数
function isPDF(filename: string) {
  return /\.pdf$/i.test(filename);
}
</script>

<template>
  <button class="btn-head-image" title="Wiki一覧画面へ遷移します。&#10;ショートカット: Ctrl + 1"
    v-on:click="listRedirect()"><img :src="`${assetsUrl}home_24.png`" class="btn-img" alt="home_24.png"></button>
  <button class="btn-head-image" title="プレビュー画面へ戻ります。&#10;ショートカット: Ctrl + 2"
    v-on:click="previewRedirect(props.id)"><img :src="`${assetsUrl}preview_on_24.png`" class="btn-img" alt="preview_on_24.png"></button>

  <!-- 削除操作可能画面であることの警告メッセージ（遷移時に描画） -->
  <transition>
    <div id="overlay-delete-warn" v-show="isDeleteModal">
      <div id="content-delete-warn">
        <h2 class="modal-h2">削除画面</h2>
        <p><strong>ここからの操作はWikiを削除する操作です。<br>操作をやめる場合 「戻る」 を選択してください。</strong></p>
        <div class="btn-zone">
          <button v-on:click="previewRedirect(props.id)">戻る</button>
          <button v-on:click="onIsDeleteModal(0)" class="btn-delete">続ける</button>
        </div>
      </div>
    </div>
  </transition>

  <!-- 各種メッセージモーダル -->
  <transition>
    <div id="overlay-message" v-show="isMessageModal">
      <div id="content-message">
        <h2 class="modal-h2">メッセージ</h2>
        <div class="input-text-zone">
          <p><strong>{{ messageText }}</strong></p>
        </div>
        <div class="btn-zone">
          <button v-on:click="messageModalOpenClose('No Message')" class="btn-modal-yes btn-hover">閉じる</button>
        </div>
      </div>
    </div>
  </transition>

  <!-- 削除の最終確認 -->
  <transition>
    <div id="overlay-delete-confirm" v-show="showContent">
      <div id="content-delete-confirm">
        <h2 class="modal-h2">最終確認</h2>
        <p><strong>本当に削除してもよろしいですか?</strong></p>
        <div class="btn-zone">
          <button v-on:click="onCloseModal(0)">やめる</button>
          <button v-on:click="onCloseModal(1)" class="btn-delete">削除</button>
        </div>
      </div>
    </div>
  </transition>

  <!-- 削除完了モーダル -->
  <transition>
    <div id="overlay-message" v-show="isDeleteOkModal">
      <div id="content-message">
        <h2 class="modal-h2">削除完了</h2>
        <div class="input-text-zone">
          <p><strong>削除しました。</strong></p>
        </div>
        <div class="btn-close">
          <button v-on:click="listRedirect()">閉じる</button>
        </div>
      </div>
    </div>
  </transition>

  <div class="contants-area">
    <div class="markdown-isprint">
      <section v-html="bindHtml"></section>
    </div>
  </div>
  <div class="footer-area">
    <div class="btn-zone">
      <button type="submit" class="btn-delete" v-if="isOwner" v-on:click.prevent="onDeleteCheck">削除</button>
    </div>
  </div>

  <footer>
    <p class="wiki-owner">Wikiオーナー: {{ wikiOwner }}</p>
  </footer>
</template>

<style scoped>
.contants-area {
  margin-top: 10px;
}

.v-enter-active,
.v-leave-active {
  transition: all 0.3s ease-in-out;
}

.v-enter-from,
.v-leave-to {
  opacity: 0;
}

.markdown-isprint {
  background-color: white;
  color: black;
  padding-top: 10px;
  padding-left: 20px;
  padding-right: 20px;
  padding-bottom: 20px;
  margin-bottom: 20px;
  border-collapse: collapse;
}

.btn-delete {
  background: rgb(219, 54, 76);
}

/* 削除画面に遷移したことの警告モーダル */
#overlay-delete-warn {
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

#content-delete-warn {
  z-index: 2;
  width: 30%;
  padding: 1em;
  background: #fff;
  border-radius: 10px;
  text-align: center;
}

/* 削除の最終確認モーダル */
#overlay-delete-confirm {
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

#content-delete-confirm {
  z-index: 2;
  width: 30%;
  padding: 1em;
  background: #fff;
  border-radius: 10px;
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
  ;
  align-items: center;
  justify-content: center;
}

#content-message {
  z-index: 4;
  width: 20%;
  padding: 1em;
  background: whitesmoke;
  border-radius: 10px;
  text-align: center;
}

.btn-zone {
  margin-top: 10px;
  margin-bottom: 5px;
  display: flex;
  justify-content: space-between;
}

.btn-close {
  margin-top: 10px;
  margin-bottom: 5px;
  text-align: center;
  align-items: center;
}

.wiki-owner {
  position: fixed;
  bottom: 1%;
  right: 1%;
  text-align: right;
  font-size: 16px;
  font-weight: bold;
  text-shadow: 1px 1px 2px rgb(202, 202, 202);
}
</style>