<script setup lang="ts">
import { marked, Renderer } from 'marked';
import type { Tokens, MarkedOptions } from 'marked';
import type { deleteWikiData, WikiData, TypeWikiOwner } from '@/interface';
import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { deleteWikiUrl, wikiOwnerGetUrl, getUserUrl } from '@/router/urls';
import { AxiosError } from 'axios';
import { useWikiStore } from '@/stores/wikis';
import { assetsUrl } from '@/setting';
import {
  videoToken,
  detailsToken,
  noteToken,
  warningToken,
  mathExtentionToken,
  youtubeToken,
  renderIframe,
  escapeHtml,
  isPDF,
  createLinkRenderer,
  createImageRenderer,
  createXssFilter,
} from '@/utils/markedSetup';
import { useMessageModal } from '@/utils/useMessageModal';
import apiClient from '@/axiosClient';
import 'katex/dist/katex.min.css';
import mermaid from 'mermaid';

// Mermaidの初期読み込みを阻止（MarkedによるHTMLレンダリング後にinitで読み込み）
mermaid.initialize({ startOnLoad: false });

// markedのスラッグ化機能をカスタマイズ
const renderer = new Renderer();
let headingIndex = -1; // 見出しのインデックス
renderer.heading = function (tokens: Tokens.Heading) {
  const id = `heading-${headingIndex++}`; // インデックスに基づいてIDを生成
  return `<h${tokens.depth} class="head${tokens.depth}">${tokens.text}</h${tokens.depth}>\n`; // class属性のCSSはトップレベル（App.vue）で定義
};

createLinkRenderer(renderer);

// mermaidの処理
const originalCodeRenderer = renderer.code.bind(renderer);
renderer.code = (tokens: Tokens.Code) => {
  if (tokens.lang == 'mermaid') {
    return '<pre class="mermaid">' + escapeHtml(tokens.text) + '\n</pre>';
  } else {
    return originalCodeRenderer(tokens);
  }
};

createImageRenderer(renderer);

// Markedにカスタムトークンを追加
marked.use({
  extensions: [videoToken, detailsToken, noteToken, warningToken, mathExtentionToken, youtubeToken],
});

// markedの設定をカスタマイズ
marked.setOptions({
  renderer,
  async: false,
});

const myXss = createXssFilter();

// ListMobile.vueへのリダイレクト
const router = useRouter();
const listRedirect = (): void => {
  router.push('/wiki/list');
};

// LoginMobile.vueへのリダイレクト
const loginRedirect = (): void => {
  router.push('/account/login');
};

// PreviewMobile.vueへのリダイレクト
const previewRedirect = (id: string): void => {
  router.push(`/wiki/preview/${id}`);
};

interface Props {
  id: string;
}

const props = defineProps<Props>();
const wikiStore = useWikiStore();
const wiki = computed((): WikiData => {
  return wikiStore.getById(props.id);
});

const deleteWikiDataInit: deleteWikiData = {
  id: wiki.value.id,
  title: wiki.value.title,
  body: wiki.value.body,
  is_public: wiki.value.is_public,
};

const deleteWikiData = ref(deleteWikiDataInit);

// マークダウンへのパース処理
const textTitleData = '# ' + wiki.value.title + '\n\n';
const textBodyData = wiki.value.body;
const markdownData = textTitleData + textBodyData;
const options: MarkedOptions = { async: false };
const htmlStr = marked.parse(markdownData, options);
const cleanHtml = myXss.process(htmlStr as string);
const renderHtml = renderIframe(cleanHtml);
const bindHtml = ref(renderHtml);

// 削除処理
const deleteWiki = async (): Promise<void> => {
  const id = deleteWikiData.value.id;
  const title = deleteWikiData.value.title;
  const body = deleteWikiData.value.body;

  try {
    const response = await apiClient.delete(deleteWikiUrl + `/${id}`);
    const deleteData = {
      id: id,
      title: title,
      body: body,
    };
    const wikiStore = useWikiStore();
    wikiStore.deleteWiki(deleteData.id);
    showContent.value = false;
    isDeleteOkModal.value = true;
  } catch (error: unknown) {
    if (typeof error === 'object' && error !== null) {
      const axiosError = error as AxiosError;

      if (axiosError.response) {
        console.log('Status code:', axiosError.response.status);
        console.log('Error data:', axiosError.response.data);
        if (axiosError.response.status === 401) {
          messageModalOpenClose('不正な操作です。\nオーナーでないデータは削除できません。');
          localStorage.setItem('loginUser', '');
          return;
        }
      } else if (axiosError.request) {
        console.log('No response was received', axiosError.request);
      } else {
        console.log('Error', axiosError.message);
      }
    } else {
      console.log('An unknown error occurred.');
    }
  }
};

// 削除画面であることの注意喚起モーダル
const isDeleteModal = ref(true);
const onIsDeleteModal = (res: number): void => {
  if (res === 1) {
    listRedirect();
  } else {
    isDeleteModal.value = false;
  }
};

// 削除確認モーダル
const showContent = ref(false);
const onDeleteCheck = (): void => {
  showContent.value = true;
};

// 削除の実行かキャンセル
const onCloseModal = (res: number): void => {
  if (res === 1) {
    localStorage.setItem('prev-table-data', '');
    deleteWiki();
  } else {
    showContent.value = false;
  }
};

// 削除完了モーダル
const isDeleteOkModal = ref(false);

// Wikiデータのオーナー取得
const wikiOwnerInit: TypeWikiOwner = {
  wikiOwner: '',
  publicName: '',
  isOwner: false,
};
const wikiOwner = ref(wikiOwnerInit);
const isOwner = ref(false);
const getWikiOwner = async (id: string): Promise<void> => {
  try {
    const response = await apiClient.get(wikiOwnerGetUrl + `/${id}`);
    wikiOwner.value.wikiOwner = response.data['WikiOwner'];
    wikiOwner.value.publicName = response.data['public_name'];
    if (response.data['is_owner'] === 'true' || response.data['is_owner'] === true) {
      wikiOwner.value.isOwner = true;
      isOwner.value = true;
    }
  } catch (error) {
    console.error('Owner Get Error');
    loginRedirect();
  }
};
getWikiOwner(props.id);

// メッセージ表示モーダル機能
const { isMessageModal, messageText, messageModalOpenClose } = useMessageModal();

// コンポーネントマウント時にmermaid.jsを発動
onMounted(() => {
  mermaid.init();
});
</script>

<template>
  <div class="head-btn-zone">
    <button class="btn-head-img" v-on:click="listRedirect()">
      <img :src="`${assetsUrl}home_24.png`" class="btn-img" alt="home_24.png" />
    </button>
    <button class="btn-head-img" v-on:click="previewRedirect(props.id)">
      <img :src="`${assetsUrl}preview_24.png`" class="btn-img" alt="preview_24.png" />
    </button>
  </div>
  <div class="overlay" v-show="showContent">
    <div class="content">
      <h2 class="modal-h2">最終確認</h2>
      <p><strong>本当に削除してもよろしいですか？</strong></p>
      <div class="btn-zone">
        <button v-on:click="onCloseModal(0)">キャンセル</button>
        <button v-on:click="onCloseModal(1)" class="btn-delete">削除</button>
      </div>
    </div>
  </div>

  <div class="overlay" v-show="isDeleteModal">
    <div class="content">
      <h2 class="modal-h2">警告</h2>
      <p>
        <strong
          >ここからの操作はWikiデータを削除することが可能です。誤ってこの画面に移動した場合は
          「戻る」を選択してください。</strong
        >
      </p>
      <div class="btn-zone">
        <button v-on:click="previewRedirect(props.id)">戻る</button>
        <button v-on:click="onIsDeleteModal(0)" class="btn-delete">続ける</button>
      </div>
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
        <button v-on:click="messageModalOpenClose('No Message')">閉じる</button>
      </div>
    </div>
  </div>

  <!-- 削除完了モーダル -->
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

  <div class="contants-area">
    <div class="markdown-isprint">
      <section v-html="bindHtml"></section>
    </div>
  </div>
  <div class="footer-area">
    <div class="btn-zone">
      <button type="submit" class="btn-delete" v-if="isOwner" v-on:click.prevent="onDeleteCheck">
        削除
      </button>
    </div>
    <div class="owner-zone">
      <p class="wiki-owner">Wikiオーナー：{{ wikiOwner.publicName }}</p>
    </div>
  </div>
</template>

<style scoped>
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

/* Modal Window */
.overlay {
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

.content {
  z-index: 2;
  width: 80%;
  padding: 1em;
  background: #fff;
  border-radius: 10px;
  text-align: center;
}

.footer-area {
  display: flex;
  justify-content: space-between;
}

.wiki-owner {
  text-align: right;
  font-size: 16px;
  font-weight: bold;
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
  width: 70%;
  padding: 1em;
  background: whitesmoke;
  border-radius: 10px;
  text-align: center;
}

.btn-zone {
  display: flex;
  justify-content: space-between;
}

.btn-delete-complate {
  text-align: center;
}
</style>
