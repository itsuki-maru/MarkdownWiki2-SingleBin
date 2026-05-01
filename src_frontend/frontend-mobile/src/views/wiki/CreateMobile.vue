<script setup lang="ts">
import type { WikiData, CreateWikiData, ImageData, UploadProgressState } from '@/interface';
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { useWikiStore } from '@/stores/wikis';
import { useImageStore } from '@/stores/images';
import { AxiosError } from 'axios';
import { createWikiUrl, disableTokenUrl, imageDeleteUrl, getUserUrl } from '@/router/urls';
import { baseUrl, assetsUrl } from '@/setting';
import { marked, Renderer } from 'marked';
import { videoToken, isPDF, isMP4 } from '@/utils/markedSetup';
import { useMessageModal } from '@/utils/useMessageModal';
import { useProtocolDetection } from '@/utils/useProtocolDetection';
import apiClient from '@/axiosClient';
import QRCode from 'qrcodejs2-fix';
import ImageUploadModal from '@/components/ImageUploadModal.vue';
import ProgressSpinner from '@/components/ProgressSpinner.vue';
import UploadProgressModal from '@/components/UploadProgressModal.vue';

// 現在ユーザーの取得
const getCurrentUser = async (): Promise<void> => {
  try {
    await apiClient.get(getUserUrl);
  } catch (error) {
    loginRedirect();
  }
};
getCurrentUser();

// markedのスラッグ化機能をカスタマイズ
const renderer = new Renderer();

// markedの設定をカスタマイズ
marked.setOptions({
  renderer,
  async: false,
});

// Markedにカスタムトークンを追加
marked.use({
  extensions: [videoToken],
});

// アプリケーションの通信プロトコル
const { isHttpsProtocol } = useProtocolDetection();

// メッセージ表示モーダル機能
const { isMessageModal, messageText, messageModalOpenClose } = useMessageModal();

// storeの定義
const wikiStore = useWikiStore();

// 画像ファイルのデータ管理
const imageStore = useImageStore();
// ImageStoreから取得したデータをMapオブジェクトとして保持
const imageList = computed((): Map<string, ImageData> => {
  return imageStore.imageList;
});

// "create" Redirect
const router = useRouter();
const createRedirect = (): void => {
  router.push('/wiki/create');
};

// "list" Redirect
const listRedirect = (): void => {
  localStorage.setItem('prev-table-data', '');
  router.push('/wiki/list');
};

// Login.vueへリダイレクト（無効トークンで上書き）
async function loginRedirect(): Promise<void> {
  try {
    await apiClient.get(disableTokenUrl);
  } catch (error) {
    console.error(error);
  }
  router.push('/account/login');
}

// Wiki作成ボタンクリック連打の抑制とプログレス表示
const isNewWikiSendNow = ref(false);
const showProgressModal = ref(false);
watch(isNewWikiSendNow, (): void => {
  if (isNewWikiSendNow.value) {
    showProgressModal.value = true;
  } else {
    showProgressModal.value = false;
  }
});

const emptyUploadProgressState = (): UploadProgressState => ({
  isOpen: false,
  phase: 'preparing',
  percent: null,
  fileName: '',
  message: '',
});
const uploadProgress = ref<UploadProgressState>(emptyUploadProgressState());

const handleImageUploaded = (markdownStr: string): void => {
  createWikiData.value.body = createWikiData.value.body + markdownStr + '\n\n';
};

// 新規Wikiデータの初期化
const crateWikiDataInit: CreateWikiData = {
  title: '',
  body: '',
  is_public: false,
};
const createWikiData = ref(crateWikiDataInit);

/** Wikiの作成処理 */
const createWiki = async (): Promise<void> => {
  if (isNewWikiSendNow.value === true) {
    return;
  } else {
    isNewWikiSendNow.value = true;
  }
  const title = createWikiData.value.title;
  const body = createWikiData.value.body;
  const is_public = createWikiData.value.is_public;

  if (title == '' || body == '') {
    messageModalOpenClose('入力データがありません。');
    isNewWikiSendNow.value = false;
    return;
  }

  const data = {
    title: title,
    body: body,
    is_public: is_public,
  };

  try {
    const response = await apiClient.post(createWikiUrl, data);

    if (createWikiData.value.is_public) {
      messageModalOpenClose(
        'Wiki（パブリック）を作成しました。このWikiは全ユーザーが閲覧可能です。',
      );
    } else {
      messageModalOpenClose(
        'Wiki（プライベート）を作成しました。このWikiは作成したユーザーだけが閲覧可能です。',
      );
    }

    const newWikiData: WikiData = {
      id: response.data['new_wiki_id'],
      user_id: response.data['user_id'],
      date: response.data['date'],
      title: title,
      body: body,
      update_at: response.data['date'],
      is_public: is_public,
    };
    wikiStore.addWiki(newWikiData);

    createWikiData.value.title = '';
    createWikiData.value.body = '';
    createWikiData.value.is_public = false;
    createRedirect();
  } catch (error) {
    console.error(error);
    messageModalOpenClose(`エラーが発生しました: ${error}`);
  } finally {
    isNewWikiSendNow.value = false;
  }
};

// LOGOUT
const onLogout = (): void => {
  localStorage.setItem('isAuthenticate', 'false');
  localStorage.setItem('loginUser', '');
  loginRedirect();
};

// MODAL OUT CHECK
const showContent = ref(false);
const onOutCheck = (): void => {
  if (createWikiData.value.title != '' || createWikiData.value.body != '') {
    showContent.value = true;
  } else {
    listRedirect();
  }
};

const onCloseModal = (res: number): void => {
  if (res === 1) {
    listRedirect();
  } else {
    showContent.value = false;
  }
};

// 画像アップロード
/** 画像アップロードのモーダル表示・非表示を管理 */
const showImageUploadModal = ref(false);
const openCloseImageUpModal = (): void => {
  if (showImageUploadModal.value === true) {
    showImageUploadModal.value = false;
  } else {
    showImageUploadModal.value = true;
  }
};

// アップロード完了モーダル機能
const isUploadedMessageModal = ref(false);
const uploadedUrl = ref('');
const uploadedUniqueFileName = ref('');
const uploadMessageModalOpenClose = (url: string, uniqueFileName: string): void => {
  if (!isUploadedMessageModal.value) {
    uploadedUrl.value = url;
    uploadedUniqueFileName.value = `${uniqueFileName}-uploaded`;
    isUploadedMessageModal.value = true;
  } else {
    isUploadedMessageModal.value = false;
    uploadedUrl.value = '';
    uploadedUniqueFileName.value = '';
  }
};

// 画像一覧モーダルの表示・非表示管理（HTTPS or Localhost）
const showImageListHttpsModal = ref(false);
const openCloseImageListHttpsModal = (): void => {
  if (showImageListHttpsModal.value === true) {
    showImageListHttpsModal.value = false;
  } else {
    showImageListHttpsModal.value = true;
  }
};
// 画像一覧モーダルの表示・非表示管理（HTTP）
const showImageListModal = ref(false);
const openCloseImageListModal = (): void => {
  if (showImageListModal.value === true) {
    showImageListModal.value = false;
  } else {
    showImageListModal.value = true;
  }
};

/** 画像とPDF、動画のプレビュー */
const imagePreviewModal = ref(false);
const imageFileSrc = ref('');
const previewSelectedImageId = ref('');
const openImagePreviewModal = (filename: string = 'notpreview', imageId: string = ''): void => {
  // PDFファイルの場合は別タブで開く
  if (isPDF(filename)) {
    window.open(`${baseUrl}/static/images/${filename}`, '_blank', 'noopener noreferrer');
    return;
  }
  if (imagePreviewModal.value === true) {
    imagePreviewModal.value = false;
  } else {
    imagePreviewModal.value = true;
    previewSelectedImageId.value = imageId;

    // 動画ファイルか否かを判定してimgタグかvideoタグか切り替え
    if (isMP4(filename)) {
      imageFileSrc.value = `<video controls="" src="${baseUrl}/static/images/${filename}" id="img-preview"></video><br>`;
    } else {
      imageFileSrc.value = `<img src="${baseUrl}/static/images/${filename}" width="90%" height="90%"><br>`;
    }
  }
};

// 画像の削除確認モーダル
const imageDeleteCheckModal = ref(false);
const onOpenImageDeleteModal = (): void => {
  imageDeleteCheckModal.value = true;
};

// 画像削除の最終確認
const onCloseImageDeleteModal = (res: number): void => {
  if (previewSelectedImageId.value === '') {
    imageDeleteCheckModal.value = false;
    return;
  }
  if (res === 1) {
    onImageDelete(previewSelectedImageId.value);
  }
  imageDeleteCheckModal.value = false;
  imagePreviewModal.value = false;
  previewSelectedImageId.value = '';
};

// テーブルから削除する際の画像IDの記録処理（ミドルウェアとして機能）
function selectedIdFromTable(selectId: string) {
  previewSelectedImageId.value = selectId;
  imageDeleteCheckModal.value = true;
}

/** 画像削除 */
const onImageDelete = async (id: string): Promise<void> => {
  try {
    const response = await apiClient.delete(imageDeleteUrl + `/${id}`);
    imageStore.deleteImage(id);
    messageModalOpenClose('削除しました');
  } catch (error) {
    if (apiClient.isAxiosError(error)) {
      // エラーオブジェクトがAxiosError型であることが保証
      const axiosError = error as AxiosError;
      if (axiosError.response) {
        const status = axiosError.response.status;
        // ステータスコード毎に処理の切り分け
        switch (status) {
          case 400:
            messageModalOpenClose(`${axiosError.response.data}`);
            console.error('Image Delete Error.', axiosError.response.data);
            break;
          case 401:
            console.error('No token provided.', axiosError.response.data);
            break;
          case 500:
            console.error('Server error, please try again later', axiosError.response.data);
            break;
          default:
            console.error(`An error occurred: ${status}`, axiosError.response.data);
        }
      }
    }
  }
};

/** 画像の検索 */
const queryFormData = ref('');
// 検索実行関数
const onSearch = (reset: boolean = false): void => {
  try {
    if (reset) {
      imageStore.queryImage('');
    } else {
      imageStore.queryImage(queryFormData.value);
    }
  } catch (error) {
    console.error(error);
  }
};

// QRコード作成モーダルの描画
const showQRContent = ref(false);
const qrCodeText = ref('');
const isGenerateOk = ref(false);
// HTMLの描画後にqrcodeを設定
let qrcode: any;
onMounted(() => {
  qrcode = new QRCode(document.getElementById('qrcode'), {
    text: qrCodeText.value,
    width: 128,
    height: 128,
    colorDark: '#000000',
    colorLight: '#ffffff',
    correctLevel: QRCode.CorrectLevel.H,
  });
});

watch(qrCodeText, () => {
  if (qrCodeText.value === '') {
    let qrElement = document.getElementById('qrcode') as HTMLElement | null;
    if (qrElement !== null) {
      const images = qrElement.querySelectorAll('img');
      images.forEach((img) => (img.style.display = 'none'));
    }
    isGenerateOk.value = false;
  } else {
    isGenerateOk.value = true;
    generateQRCode();
  }
});

const onOpenCloseQRCodeCreateModal = (): void => {
  if (showQRContent.value === true) {
    showQRContent.value = false;
  } else {
    showQRContent.value = true;
    // カーソルのフォーカスがエディタ描画完了後になるようにsetTimeoutで遅延させる
    setTimeout(() => {
      document.getElementById('qr-input-text')!.focus();
    }, 300);
  }
};

// QRCode作成関数
function generateQRCode(): void {
  const text = qrCodeText.value;
  if (text === '') {
    return;
  }

  qrcode.clear();
  qrcode.makeCode(text); // make another code.
}

// QRCode保存関数
function saveQRCode(): void {
  const canvas: any = document.querySelector('#qrcode canvas');
  if (canvas) {
    // canvas要素から画像のURLを生成
    const imageUrl = canvas.toDataURL('image/png').replace('image/png', 'image/octet-stream');
    // ダウンロードリンクを作成
    const link = document.createElement('a');
    link.download = 'qrcode.png';
    link.href = imageUrl;
    link.click();
  }
}

// 画像一覧モーダル表示時に灰色の部分のクリック時にも画像一覧モーダルを閉じる処理
// HTMLが描画後に組み込む（onmoutedを利用）
onMounted(() => {
  // オーバレイとヘルプの内容を取得
  const imgListModal = document.getElementById('overlay-imagelist');
  const imgListModalContent = document.getElementById('content-image');

  // 灰色部分クリック時にクローズ処理がなされるようにイベント設定
  if (imgListModal) {
    imgListModal.addEventListener('click', function (event) {
      if (showImageListModal.value === true) {
        showImageListModal.value = false;
      } else {
        return;
      }
    });
  }

  // 灰色の部分以外（content-image）をクリックした時にはイベント伝搬を止め、クローズさせない
  if (imgListModalContent) {
    imgListModalContent.addEventListener('click', function (event) {
      event.stopPropagation();
    });
  }
});

// 画像プレビューモーダル表示時に灰色の部分のクリック時にも画像プレビューモーダルを閉じる処理
// HTMLが描画後に組み込む（onmoutedを利用）
onMounted(() => {
  // オーバレイとヘルプの内容を取得
  const imgPreviewModal = document.getElementById('overlay-image-preview');
  const imgPreviewModalContent = document.getElementById('content-image-view');
  // 灰色部分クリック時にクローズ処理がなされるようにイベント設定
  if (imgPreviewModal) {
    imgPreviewModal.addEventListener('click', function (event) {
      if (imagePreviewModal.value === true) {
        imagePreviewModal.value = false;
      } else {
        return;
      }
    });
  }
  // 灰色の部分以外（content-image-view）をクリックした時にはイベント伝搬を止め、クローズさせない
  if (imgPreviewModalContent) {
    imgPreviewModalContent.addEventListener('click', function (event) {
      event.stopPropagation();
    });
  }
});

// QRコード生成モーダル
onMounted(() => {
  const genQRCodeModal = document.getElementById('overlay-gen-qrcode');
  const genQRCodeModalContent = document.getElementById('content-gen-qrcode');
  if (genQRCodeModal) {
    genQRCodeModal.addEventListener('click', function (event) {
      if (showQRContent.value === true) {
        showQRContent.value = false;
      } else {
        return;
      }
    });
  }
  if (genQRCodeModalContent) {
    genQRCodeModalContent.addEventListener('click', function (event) {
      event.stopPropagation();
    });
  }
});

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

// 画像のパスをクリップボードコピー
// なお、HTTPSによる通信かlocalhostでの通信のみ実現可能であり
// HTTP通信時には使用不可
const onImageCopyPath = (id: string) => {
  const imageData = imageStore.getById(id);
  const imageName = imageData.filename;
  const uuidName = imageData.uuid_filename;
  // 静止画か動画か判定してマークダウンを切り替え
  let imageUrlMarkdown = '';
  if (isMP4(imageData.filename)) {
    imageUrlMarkdown = `?[${imageName}](${baseUrl}/static/images/${uuidName})`;
  } else {
    if (isPDF(imageData.filename)) {
      imageUrlMarkdown = `[${imageName}](${baseUrl}/static/images/${uuidName})`;
    } else {
      imageUrlMarkdown = `![${imageName}](${baseUrl}/static/images/${uuidName})`;
    }
  }
  navigator.clipboard.writeText(imageUrlMarkdown);
  messageModalOpenClose('クリップボードにコピーしました。');
};

// キーボードショートカットを追加
const handleKeyDown = (event: KeyboardEvent) => {
  // List.vueへ移動
  if (event.ctrlKey && event.key === '1') {
    event.preventDefault(); // デフォルトのブラウザのショートカットをキャンセル
    listRedirect();

    // 画像挿入モーダル
  } else if (event.ctrlKey && event.key === '2') {
    event.preventDefault();
    openCloseImageUpModal();

    // 画像一覧モーダル
  } else if (event.ctrlKey && event.key === '3') {
    event.preventDefault();
    if (isHttpsProtocol) {
      openCloseImageListHttpsModal();
    } else {
      openCloseImageListModal();
    }

    // QRコード生成モーダル
  } else if (event.ctrlKey && event.key === '4') {
    event.preventDefault();
    onOpenCloseQRCodeCreateModal();

    // title入力欄にフォーカス
  } else if (event.ctrlKey && event.key === 'i') {
    event.preventDefault();
    const inputTitleElement = document.getElementById('title-input-text');
    if (inputTitleElement) {
      inputTitleElement.focus();
    }

    // 作成
  } else if (event.ctrlKey && event.key === 'm') {
    event.preventDefault();
    createWiki();

    // Escapeキーでモーダルウィンドウをクローズ
  } else if (event.key === 'Escape') {
    event.preventDefault();
    if (isMessageModal.value) {
      isMessageModal.value = false;
    }
  }
};

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});
onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

function insertMarkdown(text: string) {
  // textareaを取得
  const textarea = document.getElementById('wiki-detail')! as HTMLTextAreaElement;

  // 現在のカーソル位置を取得
  const startPos = textarea.selectionStart;
  const endPos = textarea.selectionEnd;

  // 元のテキストを分割して新しいテキストを挿入
  const beforeText = textarea.value.substring(0, startPos);
  const afterText = textarea.value.substring(endPos);
  textarea.value = beforeText + text + afterText;

  // カーソル位置を更新
  const newCursorPos = startPos + text.length;
  textarea.setSelectionRange(newCursorPos, newCursorPos);

  // textareaにフォーカスを戻す
  textarea.focus();

  if (isHttpsProtocol.value) {
    navigator.clipboard.writeText(text);
  }
}

// マークダウン入力支援ボタンの表示・非表示
const isShowMarkdownInputButton = ref(true);
function handleMarkdownInputButtons() {
  if (isShowMarkdownInputButton.value) {
    isShowMarkdownInputButton.value = false;
  } else {
    isShowMarkdownInputButton.value = true;
  }
}
</script>

<template>
  <div class="head-btn-zone">
    <button class="btn-head-img" v-on:click="onOutCheck">
      <img :src="`${assetsUrl}home_24.png`" class="btn-img" alt="home_24.png" />
    </button>
    <button class="btn-head-img" v-on:click="openCloseImageUpModal">
      <img :src="`${assetsUrl}smartphone_line24.png`" class="btn-img" alt="smartphone_line24.png" />
    </button>
    <button v-if="isHttpsProtocol" class="btn-head-img" v-on:click="openCloseImageListHttpsModal">
      <img :src="`${assetsUrl}documents_line24.png`" class="btn-img" alt="documents_line24.png" />
    </button>
    <button v-else="isHttpsProtocol" class="btn-head-img" v-on:click="openCloseImageListModal">
      <img :src="`${assetsUrl}documents_line24.png`" class="btn-img" alt="documents_line24.png" />
    </button>
    <button class="btn-head-img" v-on:click="onOpenCloseQRCodeCreateModal">
      <img
        :src="`${assetsUrl}code_reader_line24.png`"
        class="btn-img"
        alt="code_reader_line24.png"
      />
    </button>
    <button class="btn-head-img" v-on:click="handleMarkdownInputButtons">
      <img :src="`${assetsUrl}markdown_24.png`" class="btn-img" alt="markdown_24.png" />
    </button>
    <button class="btn-head-img" v-on:click="onLogout">
      <img :src="`${assetsUrl}exit_24.png`" class="btn-img" alt="exit_24.png" />
    </button>
  </div>

  <!-- 入力フォーム -->
  <div class="main-container">
    <h3 id="title_h3_1">Editor</h3>
    <div class="title-row">
      <input
        type="text"
        id="title-input-text"
        class="title"
        required
        v-model="createWikiData.title"
        placeholder="タイトル"
      />
    </div>
    <textarea
      class="textarea"
      ref="formArea"
      required
      v-model="createWikiData.body"
      placeholder="## 内容（マークダウン記法で記述）&#13;&#10;&#13;&#10;本文を記述。"
      id="wiki-detail"
    ></textarea>
    <div class="form-footer">
      <p class="switch-btn-container">
        <label for="switch" class="switch-label">
          <div class="switch">
            <input type="checkbox" id="switch" v-model="createWikiData.is_public" />
            <div class="base"></div>
            <div class="circle"></div>
            <div class="slider"></div>
          </div>
          <span v-if="createWikiData.is_public" class="switch-title">パブリック</span>
          <span v-else class="switch-title">プライベート</span>
        </label>
      </p>
      <button type="submit" class="btn-post" v-on:click.prevent="createWiki">+ 作成</button>
    </div>
    <div class="input-tools" v-if="isShowMarkdownInputButton">
      <button class="btn-input-tools" title="## を挿入" v-on:click="insertMarkdown('## ')">
        <img
          :src="`${assetsUrl}format_h2_24.png`"
          class="btn-input-tools-img"
          alt="format_h2_24.png"
        />
      </button>
      <button class="btn-input-tools" title="### を挿入" v-on:click="insertMarkdown('### ')">
        <img
          :src="`${assetsUrl}format_h3_24.png`"
          class="btn-input-tools-img"
          alt="format_h3_24.png"
        />
      </button>
      <button class="btn-input-tools" title="** を挿入" v-on:click="insertMarkdown('**')">
        <img
          :src="`${assetsUrl}format_bold_24.png`"
          class="btn-input-tools-img"
          alt="format_bold_24.png"
        />
      </button>
      <button
        class="btn-input-tools"
        title="チェックボックスを挿入"
        v-on:click="insertMarkdown('- [ ] \n- [ ] \n- [ ] ')"
      >
        <img
          :src="`${assetsUrl}check_box_24.png`"
          class="btn-input-tools-img"
          alt="check_box_24.png"
        />
      </button>
      <button class="btn-input-tools" title="- を挿入" v-on:click="insertMarkdown('- ')">
        <img
          :src="`${assetsUrl}format_list_bulleted_24.png`"
          class="btn-input-tools-img"
          alt="format_list_bulleted_24.png"
        />
      </button>
      <button class="btn-input-tools" title="1. を挿入" v-on:click="insertMarkdown('1. ')">
        <img
          :src="`${assetsUrl}format_list_numbered_24.png`"
          class="btn-input-tools-img"
          alt="format_list_numbered_24.png"
        />
      </button>
      <button class="btn-input-tools" title="|を挿入" v-on:click="insertMarkdown('|')">
        <img :src="`${assetsUrl}table_24.png`" class="btn-input-tools-img" alt="table_24.png" />
      </button>
      <button class="btn-input-tools" title="---を挿入" v-on:click="insertMarkdown('---')">
        <img
          :src="`${assetsUrl}more_horiz_24.png`"
          class="btn-input-tools-img"
          alt="more_horiz_24.png"
        />
      </button>
      <button class="btn-input-tools" title="~を挿入" v-on:click="insertMarkdown('~')">
        <img
          :src="`${assetsUrl}strikethrough_24.png`"
          class="btn-input-tools-img"
          alt="strikethrough_24.png"
        />
      </button>
      <button class="btn-input-tools" title="```を挿入" v-on:click="insertMarkdown('```')">
        <img :src="`${assetsUrl}code_24.png`" class="btn-input-tools-img" alt="code_24.png" />
      </button>
      <button class="btn-input-tools" title="`を挿入" v-on:click="insertMarkdown('`')">
        <img
          :src="`${assetsUrl}ink_highlighter_24.png`"
          class="btn-input-tools-img"
          alt="ink_highlighter_24.png"
        />
      </button>
      <button class="btn-input-tools" title=">を挿入" v-on:click="insertMarkdown('>')">
        <img :src="`${assetsUrl}chat_24.png`" class="btn-input-tools-img" alt="chat_24.png" />
      </button>
      <button
        class="btn-input-tools"
        title="[ Title ]( URL )を挿入"
        v-on:click="insertMarkdown('[ Title ]( URL )')"
      >
        <img :src="`${assetsUrl}link_24.png`" class="btn-input-tools-img" alt="link_24.png" />
      </button>
      <button
        class="btn-input-tools"
        title=":::detailsを挿入"
        v-on:click="insertMarkdown(':::details タイトル\n非表示にする内容\n:::')"
      >
        <img :src="`${assetsUrl}more_24.png`" class="btn-input-tools-img" alt="more_24.png" />
      </button>
      <button
        class="btn-input-tools"
        title=":::noteを挿入"
        v-on:click="insertMarkdown(':::note タイトル\n内容\n:::')"
      >
        <img :src="`${assetsUrl}info_24.png`" class="btn-input-tools-img" alt="info_24.png" />
      </button>
      <button
        class="btn-input-tools"
        title=":::warningを挿入"
        v-on:click="insertMarkdown(':::warning タイトル\n内容\n:::')"
      >
        <img :src="`${assetsUrl}warning_24.png`" class="btn-input-tools-img" alt="warning_24.png" />
      </button>
      <button class="btn-input-tools" title="$$を挿入" v-on:click="insertMarkdown('$$\n数式\n$$')">
        <img :src="`${assetsUrl}math24.png`" class="btn-input-tools-img" alt="math24.png" />
      </button>
    </div>
  </div>

  <!-- 作成中のデータがある場合の画面遷移確認制御 -->
  <transition>
    <div id="overlay" v-show="showContent">
      <div id="content">
        <h2 class="modal-h2">メッセージ</h2>
        <p><strong>保存せずに移動しますか？</strong></p>
        <div class="btn-zone">
          <button v-on:click="onCloseModal(1)">はい</button>
          <button v-on:click="onCloseModal(0)">いいえ</button>
        </div>
      </div>
    </div>
  </transition>

  <ImageUploadModal
    :is-open="showImageUploadModal"
    :is-editing-marker="true"
    :is-https-protocol="isHttpsProtocol"
    @close="showImageUploadModal = false"
    @uploaded="handleImageUploaded"
    @message="messageModalOpenClose"
    @show-uploaded-url="uploadMessageModalOpenClose"
    @upload-progress-change="uploadProgress = $event"
  />

  <!-- アップロード完了モーダル -->
  <div id="overlay-uploaded-message" v-show="isUploadedMessageModal">
    <div id="content-uploaded-message">
      <h2 class="modal-h2">メッセージ</h2>
      <div class="input-text-zone" v-if="isHttpsProtocol">
        <p><strong>アップロード完了。</strong></p>
        <pre
          :id="uploadedUniqueFileName"
          class="hidden-code-text"
        ><code :id=uploadedUniqueFileName>{{ uploadedUrl }}</code></pre>
        <button
          id="link-copy-btn"
          v-on:click="selectTextOrClipboardCopy(`${uploadedUniqueFileName}`)"
        >
          画像のリンクを取得
        </button>
      </div>
      <div class="input-text-zone" v-else="isHttpsProtocol">
        <p>
          <strong>アップロード完了。<br />次のテキストリンクをコピーして使用してください。</strong>
        </p>
        <pre><code :id=uploadedUniqueFileName v-on:click="selectTextOrClipboardCopy(`${uploadedUniqueFileName}`)">{{ uploadedUrl }}</code></pre>
      </div>
      <div class="btn-zone">
        <button id="message-close-btn" v-on:click="uploadMessageModalOpenClose('', '')">
          閉じる
        </button>
      </div>
    </div>
  </div>

  <!-- 画像一覧モーダルウィンドウ（http） -->
  <div id="overlay-imagelist" v-show="showImageListModal">
    <div id="content-image">
      <h2 style="text-align: center">画像・PDF・動画</h2>
      <div class="search-tool-area">
        <input type="text" class="query-input" placeholder="検索ワード" v-model="queryFormData" />
        <button class="btn-search-start" type="submit" v-on:click.prevent="onSearch(false)">
          <img :src="`${assetsUrl}search_fill24.png`" class="btn-img" alt="search_fill24.png" />
        </button>
        <button class="btn-search-start" type="submit" v-on:click.prevent="onSearch(true)">
          <img :src="`${assetsUrl}update_fill24.png`" class="btn-img" alt="update_fill24.png" />
        </button>
      </div>
      <div class="table_sticky_imagelist">
        <table>
          <thead>
            <tr>
              <th>URL</th>
              <th>Prv</th>
              <th>Del</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="[id, image] in imageList" v-bind:key="id">
              <td
                v-if="isPDF(image.uuid_filename)"
                :id="image.uuid_filename"
                v-on:click="selectTextOrClipboardCopy(image.uuid_filename)"
              >
                [{{ image.filename }}]({{ baseUrl }}/static/images/{{ image.uuid_filename }})
              </td>
              <td
                v-else-if="isMP4(image.uuid_filename)"
                :id="image.uuid_filename"
                v-on:click="selectTextOrClipboardCopy(image.uuid_filename)"
              >
                ?[{{ image.filename }}]({{ baseUrl }}/static/images/{{ image.uuid_filename }})
              </td>
              <td
                v-else
                :id="image.uuid_filename"
                v-on:click="selectTextOrClipboardCopy(image.uuid_filename)"
              >
                ![{{ image.filename }}]({{ baseUrl }}/static/images/{{ image.uuid_filename }})
              </td>

              <td
                v-if="isPDF(image.filename)"
                v-on:click.prevent="openImagePreviewModal(image.uuid_filename, image.id)"
                class="td-img"
              >
                <img
                  :src="`${assetsUrl}picture_as_pdf.png`"
                  class="btn-img-table"
                  alt="picture_as_pdf.png"
                />
              </td>
              <td
                v-else="isPDF(image.filename)"
                v-on:click.prevent="openImagePreviewModal(image.uuid_filename, image.id)"
                class="td-img"
              >
                <img :src="`${assetsUrl}camera24.png`" class="btn-img-table" alt="camera24.png" />
              </td>

              <td v-on:click.prevent="selectedIdFromTable(image.id)" class="td-img">
                <img :src="`${assetsUrl}delete.png`" class="btn-img-table" alt="delete.png" />
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="btn-zone">
        <button v-on:click="openCloseImageListModal()">閉じる</button>
      </div>
    </div>
  </div>

  <!-- 画像一覧モーダルウィンドウ（https or localhost） -->
  <div id="overlay-image-https-list" v-show="showImageListHttpsModal">
    <div id="content-image-https-list">
      <h2 style="text-align: center">画像・PDF・動画</h2>
      <div class="search-tool-area">
        <input type="text" class="query-input" placeholder="検索ワード" v-model="queryFormData" />
        <button class="btn-search-start" type="submit" v-on:click.prevent="onSearch(false)">
          <img :src="`${assetsUrl}search_fill24.png`" class="btn-img" alt="search_fill24.png" />
        </button>
        <button class="btn-search-start" type="submit" v-on:click.prevent="onSearch(true)">
          <img :src="`${assetsUrl}update_fill24.png`" class="btn-img" alt="update_fill24.png" />
        </button>
      </div>
      <div class="table_sticky_imagelist">
        <table>
          <thead>
            <tr>
              <th>URL</th>
              <th>Prv</th>
              <th>Del</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="[id, image] in imageList" v-bind:key="id">
              <td v-on:click="onImageCopyPath(image.id)">{{ image.filename }}</td>
              <td
                v-if="isPDF(image.filename)"
                v-on:click.prevent="openImagePreviewModal(image.uuid_filename, image.id)"
                class="td-img"
              >
                <img
                  :src="`${assetsUrl}picture_as_pdf.png`"
                  class="btn-img-table"
                  alt="picture_as_pdf.png"
                />
              </td>
              <td
                v-else="isPDF(image.filename)"
                v-on:click.prevent="openImagePreviewModal(image.uuid_filename, image.id)"
                class="td-img"
              >
                <img :src="`${assetsUrl}camera24.png`" class="btn-img-table" alt="camera24.png" />
              </td>
              <td v-on:click.prevent="selectedIdFromTable(image.id)" class="td-img">
                <img :src="`${assetsUrl}delete.png`" class="btn-img-table" alt="delete.png" />
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="btn-zone">
        <button v-on:click="openCloseImageListHttpsModal()">閉じる</button>
      </div>
    </div>
  </div>

  <!-- 画像プレビュー -->
  <div id="overlay-image-preview" v-show="imagePreviewModal">
    <div id="content-image-view">
      <section v-html="imageFileSrc"></section>
      <div class="btn-zone">
        <button v-on:click.prevent="openImagePreviewModal()">閉じる</button>
        <button v-on:click.prevent="onOpenImageDeleteModal()" class="btn-delete">削除</button>
      </div>
    </div>
  </div>

  <div class="overlay-delete-image" v-if="imageDeleteCheckModal">
    <div class="content-delete-image">
      <h2 class="modal-h2">最終確認</h2>
      <p><strong>本当に削除しますか？</strong></p>
      <div class="btn-zone">
        <button v-on:click="onCloseImageDeleteModal(0)">やめる</button>
        <button v-on:click="onCloseImageDeleteModal(1)" class="btn-delete">削除</button>
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

  <!-- QR生成モーダル -->
  <div id="overlay-gen-qrcode" v-show="showQRContent">
    <div id="content-gen-qrcode">
      <h2 class="modal-h2">QRコード生成</h2>
      <div class="setting-contents">
        <div id="qrcode" class="qrcode"></div>
        <div class="init-latlng-zone">
          <div class="latitude-zone">
            <input
              type="text"
              maxlength="150"
              title=""
              id="qr-input-text"
              placeholder="Input Text."
              class="input-textbox"
              required
              v-model="qrCodeText"
            />
          </div>
          <div :class="{ 'btn-zone': isGenerateOk, 'btn-close': !isGenerateOk }">
            <button v-if="isGenerateOk" v-on:click="saveQRCode()">保存</button>
            <button v-on:click="onOpenCloseQRCodeCreateModal">閉じる</button>
          </div>
        </div>
      </div>
    </div>
  </div>

  <ProgressSpinner :is-open="showProgressModal" />
  <UploadProgressModal :is-open="uploadProgress.isOpen" :progress="uploadProgress" />
</template>

<style scoped></style>
