<script setup lang="ts">
import type { UpdateWikiData, WikiData, ImageData, LocalStrageItem } from '@/interface';
import { ref, computed, onMounted, onUnmounted, onBeforeUnmount, watch, nextTick } from 'vue';
import type { Ref } from 'vue';
import { useRouter } from 'vue-router';
import {
  updateWikiUrl,
  imageDeleteUrl,
  getUserUrl,
  wikiOwnerGetUrl,
  postEditWikiRequestUrl,
} from '@/router/urls';
import { AxiosError } from 'axios';
import { marked, Renderer } from 'marked';
import type { Tokens, MarkedOptions } from 'marked';
import { useWikiStore } from '@/stores/wikis';
import { useEditRequestWikiStore } from '@/stores/editWikis';
import { useImageStore } from '@/stores/images';
import { baseUrl, assetsUrl } from '@/setting';
import ace from 'ace-builds';
import 'ace-builds/src-noconflict/ext-searchbox'; // Ctrl+Fで検索ボックスを使用するために必要なモジュール
import 'ace-builds/src-noconflict/mode-markdown'; // Aceでマークダウンを使用するためのモジュール
import 'ace-builds/src-noconflict/theme-monokai'; // Aceのテーマのモジュール
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
  isMP4,
  createLinkRenderer,
  createImageRenderer,
  createXssFilter,
} from '@/utils/markedSetup';
import { useMessageModal } from '@/utils/useMessageModal';
import { useProtocolDetection } from '@/utils/useProtocolDetection';
import { useImageUpload } from '@/utils/useImageUpload';
import apiClient from '@/axiosClient';
import katex from 'katex';
import 'katex/dist/katex.min.css';

// データIDを管理するProps
interface Props {
  id: string;
}
const props = defineProps<Props>();

// メッセージ表示モーダル機能
const { isMessageModal, messageText, messageModalOpenClose } = useMessageModal();

// アプリケーションの通信プロトコル
const { isHttpsProtocol } = useProtocolDetection();

// Wikiデータのオーナー取得
const isOwner = ref<boolean | null>(null);
const getWikiOwner = async (id: string): Promise<void> => {
  try {
    const response = await apiClient.get(wikiOwnerGetUrl + `/${id}`);
    const ownerPublicName = response.data['public_name'];
    if (response.data['is_owner'] === 'true' || response.data['is_owner'] === true) {
      isOwner.value = true;
    } else {
      messageModalOpenClose(`${ownerPublicName} さんへの変更申請画面です。`);
    }
  } catch (error) {
    console.error('Owner Get Error');
    loginRedirect();
  }
};
getWikiOwner(props.id);

// KaTeXによる数式描画機能
const formula = ref('');
const renderedFormula = ref('');
const katexPreviewModal = ref(false);
const isGenMath = ref(false);
const onCloseKatexModal = (): void => {
  if (katexPreviewModal.value) {
    katexPreviewModal.value = false;
  } else {
    katexPreviewModal.value = true;
  }
};

watch(formula, () => {
  renderedFormula.value = katex.renderToString(formula.value, {
    throwOnError: false,
  });
  if (formula.value === '') {
    isGenMath.value = false;
  } else {
    isGenMath.value = true;
  }
});

const mathContainer = ref<HTMLElement | null>(null);
const insertMathImage = async () => {
  if (mathContainer.value && formula.value !== '') {
    // キャプチャ前にサイズを固定
    await nextTick();
    const originalStyle = mathContainer.value.style.transform;
    mathContainer.value.style.transform = 'scale(1)';

    const { default: html2canvas } = await import('html2canvas');
    const canvas = await html2canvas(mathContainer.value, {
      scale: 1.2, // スケールを1に固定
      backgroundColor: null, // 背景を透明化
      useCORS: true,
    });

    // 元のスタイルに戻す
    mathContainer.value.style.transform = originalStyle;

    isImageSendNow.value === true;
    canvas.toBlob((blob) => {
      if (blob) {
        selectedImageBlob.value = blob;
        selectedFileName.value = 'math.png';
        imageFileSend();
      } else {
        isImageSendNow.value === false;
        selectedFileName.value = '';
      }
    });
  }
};

const saveMathImage = async () => {
  if (mathContainer.value && formula.value !== '') {
    // キャプチャ前にサイズを固定
    await nextTick();
    const originalStyle = mathContainer.value.style.transform;
    mathContainer.value.style.transform = 'scale(1)';

    const { default: html2canvas } = await import('html2canvas');
    const canvas = await html2canvas(mathContainer.value, {
      scale: 1.2, // スケールを1に固定
      backgroundColor: null, // 背景を透明化
      useCORS: true,
    });

    // 元のスタイルに戻す
    mathContainer.value.style.transform = originalStyle;

    const link = document.createElement('a');
    link.href = canvas.toDataURL('image/png');
    link.download = 'math.png';
    link.click();
  }
};

const mermaid: any = (window as any).mermaid;
// Mermaidの初期読み込みを阻止（MarkedによるHTMLレンダリング後にinitで読み込み）
mermaid.initialize({ startOnLoad: false });

// markedのスラッグ化機能をカスタマイズ
const renderer = new Renderer();
let headingIndex = -1; // 見出しのインデックス
renderer.heading = function (tokens: Tokens.Heading) {
  const id = `heading-${headingIndex++}`; // インデックスに基づいてIDを生成
  return `<h${tokens.depth} class="head${tokens.depth}">${tokens.text}</h${tokens.depth}>\n`; // class属性のCSSはトップレベル（App.vue）で定義
};

// [テキスト](URL)で定義された外部リンクを別タブで開かせるカスタムレンダラ設定
createLinkRenderer(renderer);

// mermaidの処理
const originalCodeRenderer = renderer.code.bind(renderer);
renderer.code = (tokens: Tokens.Code) => {
  let html = originalCodeRenderer(tokens);
  if (tokens.lang == 'mermaid') {
    return '<pre class="mermaid">' + escapeHtml(tokens.text) + '\n</pre>';
  } else {
    return originalCodeRenderer(tokens);
  }
};

createImageRenderer(renderer);

// markedの設定をカスタマイズ
marked.setOptions({
  renderer,
  async: false,
});

// Markedにカスタムトークンを追加
marked.use({
  extensions: [videoToken, detailsToken, noteToken, warningToken, mathExtentionToken, youtubeToken],
});

const myXss = createXssFilter();

// Aceエディタを定義
const editorRef = ref<HTMLDivElement | null>(null);
let editor: any | null = null;

// content（bodyの要素）の変化を監視
const content = ref('');
watch(content, (newContent) => {
  if (editor && editor.getValue() !== newContent) {
    editor.setValue(newContent, 1);
  }
});

// ローカルストレージから最後のアプリケーション設定情報を取得
const localStorageItems = getLocalStrageInfo();
const isShowTools = ref(false); // マークダウン入力ツール表示コントロール
const isPreview = ref(true); // プレビューの表示非表示

if (localStorageItems.isShowToolsFromLocalStrage === null) {
  localStorage.setItem('isShowTools', 'false');
}

if (localStorageItems.isShowToolsFromLocalStrage === 'true') {
  isShowTools.value = true;
}

if (localStorageItems.isPreviewFromLocalStrage === null) {
  localStorage.setItem('isPreview', 'true');
}

if (localStorageItems.isPreviewFromLocalStrage === 'false') {
  isPreview.value = false;
}

// ローカルストレージから前回起動時の状況を取得
function getLocalStrageInfo(): LocalStrageItem {
  const localstrageItem = {
    isShowToolsFromLocalStrage: localStorage.getItem('isShowTools'),
    isPreviewFromLocalStrage: localStorage.getItem('isPreview'),
  };
  return localstrageItem;
}

// マークダウン入力ツールの表示非表示管理
const handleInputTool = (): void => {
  if (isShowTools.value) {
    isShowTools.value = false;
    localStorage.setItem('isShowTools', 'false');
  } else {
    isShowTools.value = true;
    localStorage.setItem('isShowTools', 'true');
  }
};

// マークダウンプレビューの表示非表示切替
const handlePreview = (): void => {
  if (isPreview.value) {
    isPreview.value = false;
    localStorage.setItem('isPreview', 'false');
  } else {
    isPreview.value = true;
    localStorage.setItem('isPreview', 'true');
  }
};

// HTML描画後にAceエディタを反映
onMounted(() => {
  // Aceの設定
  if (editorRef.value) {
    editor = ace.edit(editorRef.value);
    editor.getSession().setMode('ace/mode/markdown');
    editor.getSession().setUseWrapMode(true);
    editor.setFontSize(16);
    // 80文字の縦ラインを消す
    editor.setShowPrintMargin(false);
    editor.getSession().setValue(wiki.value.body);
  }

  // editorの変更を監視
  editor.on('change', () => {
    const newValue = editor.getValue();
    // mermaid.jsによるフロー図レンダリング
    drawMermaid();
    if (newValue !== content.value) {
      content.value = newValue;
    }
  });

  let isEditorScrolling = false;
  let isPreviewScrolling = false;

  // editorからpreviewへのスクロールの同期
  // previewからeditorの同期は不可（画像の差分を微調整するため）
  editor.getSession().on('changeScrollTop', function () {
    if (isPreviewScrolling) return;

    const editorScroll = editor.getSession().getScrollTop();
    const editorMaxScroll =
      editor.renderer.layerConfig.maxHeight - editor.renderer.$size.scrollerHeight;
    const preview = document.getElementById('result')!;
    if (!preview) return;

    const previewMaxScroll = preview.scrollHeight - preview.clientHeight;

    isEditorScrolling = true;
    preview.scrollTop = (editorScroll / editorMaxScroll) * previewMaxScroll;
    setTimeout(() => (isEditorScrolling = false), 50);
  });
});

// Mermaid.jsのエラーハンドリング
async function drawMermaid() {
  try {
    await mermaid.init();
  } catch (error) {
    console.error('Mermaid.js Syntax Error.');
  }
}

onUnmounted(() => {
  if (editor) {
    editor.destroy();
  }
});

// imageのデータ管理
const imageStore = useImageStore();

// Preview.vueへのリダイレクト
const router = useRouter();
const previewRedirect = (id: string): void => {
  router.push(`/wiki/preview/${id}`);
};

// List.vueへリダイレクト
const listRedirect = (): void => {
  router.push('/wiki/list');
};

// wikiのデータ管理
const wikiStore = useWikiStore();
const wiki = computed((): WikiData => {
  return wikiStore.getById(props.id);
});

// 更新データの初期化
const updateWikiDataInit: UpdateWikiData = {
  id: wiki.value.id,
  title: wiki.value.title,
  body: wiki.value.body,
  is_public: wiki.value.is_public,
};
const updateWikiData = ref(updateWikiDataInit);
content.value = wiki.value.body;

// 初期データから変更があるか比較するための定数
const editConfirmationTitle = updateWikiDataInit.title;
const editConfirmationBody = updateWikiDataInit.body;
const editConfirmationIsPublic = updateWikiDataInit.is_public;

// 初期データと現在のデータに変更があるか比較する関数
// 変更があればtrueを返却
function checkingEditConfirm(): boolean {
  if (
    editConfirmationTitle === updateWikiData.value.title &&
    editConfirmationBody === content.value &&
    editConfirmationIsPublic === updateWikiData.value.is_public
  ) {
    return false;
  }
  return true;
}

// データに変更がある場合の画面遷移制御
const showYesNoMessageContent = ref(false);
const redirectTargetRef = ref('list');
const onOutCheck = (redirectTarget: string = 'list'): void => {
  redirectTargetRef.value = redirectTarget;
  const isConfirm = checkingEditConfirm();
  if (isConfirm) {
    showYesNoMessageContent.value = true;
  } else {
    if (redirectTarget === 'list') {
      listRedirect();
    } else if (redirectTarget === 'preview') {
      previewRedirect(updateWikiData.value.id);
    } else {
      return;
    }
  }
};

// 作成中のデータ存在時の画面遷移を確認後の処理
const onCloseModal = (res: number): void => {
  if (res === 1) {
    if (redirectTargetRef.value === 'list') {
      listRedirect();
    } else if (redirectTargetRef.value === 'preview') {
      previewRedirect(updateWikiData.value.id);
    }
  } else {
    showYesNoMessageContent.value = false;
  }
};

// 更新完了メッセージモーダル
const isUpdateOkModal = ref(false);

// Wiki更新ボタンクリック連打の抑制とプログレス表示
const isWikiUpdateSendNow = ref(false);
const showProgressModal = ref(false);
watch(isWikiUpdateSendNow, (): void => {
  if (isWikiUpdateSendNow.value) {
    showProgressModal.value = true;
  } else {
    showProgressModal.value = false;
  }
});

// Wikiの更新処理
const updateWiki = async (): Promise<void> => {
  if (!checkingEditConfirm()) {
    messageModalOpenClose('変更はありません。');
    return;
  }

  if (isWikiUpdateSendNow.value === true) {
    return;
  } else {
    isWikiUpdateSendNow.value = true;
  }

  const id = updateWikiData.value.id;
  const title = updateWikiData.value.title;
  const body = content.value;

  // パブリック・プライベートの切り替えUIから取得
  let is_public = false;
  if (updateWikiData.value.is_public) {
    is_public = true;
  } else {
    is_public = false;
  }

  // 入力項目の検証
  if (title === '') {
    messageModalOpenClose('Wikiのタイトルが入力されていません。');
    isWikiUpdateSendNow.value = false;
    return;
  } else if (body === '') {
    messageModalOpenClose('Wikiのコンテンツが入力されていません。');
    isWikiUpdateSendNow.value = false;
    return;
  }

  const data = {
    title: title,
    body: body,
    is_public: is_public,
  };

  // axiosによるPUT
  try {
    const response = await apiClient.put(updateWikiUrl + `/${id}`, data);
    const updateData = {
      id: id,
      title: title,
      body: body,
      is_public: is_public,
    };
    const wikiStore = useWikiStore();
    wikiStore.updateWiki(updateData);
    isUpdateOkModal.value = true;
  } catch (error) {
    if (apiClient.isAxiosError(error)) {
      // エラーオブジェクトがAxiosError型であることが保証
      const axiosError = error as AxiosError<any>;
      if (axiosError.response) {
        console.error('Status code:', axiosError.response.status);
        console.error('Error data:', axiosError.response.data);
        if (axiosError.response.status === 401) {
          messageModalOpenClose('不正な操作です。\nオーナーでないデータは編集できません。');
          localStorage.setItem('loginUser', '');
          localStorage.setItem('isAuthenticate', 'false');
          return;
        }
      } else if (axiosError.request) {
        console.error('No response was received', axiosError.request);
      } else {
        console.error('Error', axiosError.message);
      }
    } else {
      console.error('An unknown error occurred.');
    }
  } finally {
    isWikiUpdateSendNow.value = false;
  }
};

// 更新申請完了メッセージモーダル
const isEditRequestOkModal = ref(false);
// 更新メッセージモーダル
const isRequestMessageModal = ref(false);
// リクエストメッセージの内容
const requestMessage = ref<string | null>(null);

const handleOpenCloseRequestMessageModal = (): void => {
  if (!checkingEditConfirm()) {
    messageModalOpenClose('変更はありません。');
    return;
  }

  if (isRequestMessageModal.value) {
    isRequestMessageModal.value = false;
  } else {
    isRequestMessageModal.value = true;
  }
};

// Wikiの更新リクエスト処理
const editRequestWiki = async (): Promise<void> => {
  if (!checkingEditConfirm()) {
    messageModalOpenClose('変更はありません。');
    return;
  }

  if (isWikiUpdateSendNow.value === true) {
    return;
  } else {
    isWikiUpdateSendNow.value = true;
  }

  const id = updateWikiData.value.id;
  const title = updateWikiData.value.title;
  const body = content.value;
  const message = requestMessage.value;

  // 入力項目の検証
  if (title === '') {
    messageModalOpenClose('Wikiのタイトルが入力されていません。');
    isWikiUpdateSendNow.value = false;
    return;
  } else if (body === '') {
    messageModalOpenClose('Wikiのコンテンツが入力されていません。');
    isWikiUpdateSendNow.value = false;
    return;
  }

  const data = {
    edit_request_title: title,
    edit_request_body: body,
    request_message: message,
    status: 'REQUESTNOW',
  };

  // axiosによるPUT
  try {
    const response = await apiClient.put(postEditWikiRequestUrl + `${id}`, data);
    const editRequestWikiStore = useEditRequestWikiStore();
    editRequestWikiStore.initList();
    isRequestMessageModal.value = false;
    isEditRequestOkModal.value = true;
  } catch (error) {
    if (apiClient.isAxiosError(error)) {
      // エラーオブジェクトがAxiosError型であることが保証
      const axiosError = error as AxiosError<any>;
      const errorStatusCode = axiosError.response?.status;
      if (errorStatusCode === 409) {
        messageModalOpenClose(
          '現在、更新リクエストを申請中のWikiであるため、新たに申請することができません。',
        );
        return;
      }
    } else {
      console.error('An unknown error occurred.');
    }
  } finally {
    isWikiUpdateSendNow.value = false;
  }
};

// Login.vueへリダイレクト
const loginRedirect = (): void => {
  router.push('/account/login');
};

// 現在ユーザーの取得
const currentUser = ref('');
const getCurrentUser = async (): Promise<void> => {
  try {
    const response = await apiClient.get(getUserUrl);
    currentUser.value = response.data['public_name'];
  } catch (error) {
    loginRedirect();
  }
};
getCurrentUser();

// マークダウンからHTMLへのコンバート処理
const markDownConv = computed((): String => {
  let plainTextTitle = '';
  if (updateWikiData.value.title) {
    plainTextTitle = '# ' + updateWikiData.value.title + '\n\n';
  }
  const plainTextBody = content.value;
  const plainText = plainTextTitle + plainTextBody;
  const options: MarkedOptions = { async: false };
  const htmlStr = marked.parse(plainText, options);
  const cleanHtml = myXss.process(htmlStr as string);
  const renderHtml = renderIframe(cleanHtml);
  return renderHtml;
});

// 画像送信のモーダル表示・非表示を管理
const showFileUploadContent = ref(false);
const openFileUpModal = (): void => {
  if (showFileUploadContent.value) {
    showFileUploadContent.value = false;
  } else {
    showFileUploadContent.value = true;
  }
};

// 画像の送信処理
const {
  selectedImageBlob,
  selectedFileName,
  isImageSendNow,
  onImageSelect,
  imageFileSend,
  imageCrear,
} = useImageUpload(showProgressModal, messageModalOpenClose, (markdownStr) =>
  insertMarkdown(markdownStr),
);

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

// 画像一覧の取得
const imageList = computed((): Map<string, ImageData> => {
  return imageStore.imageList;
});

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
const showImageListContent = ref(false);
const openImageListModal = (): void => {
  if (showImageListContent.value) {
    showImageListContent.value = false;
  } else {
    showImageListContent.value = true;
  }
};

// 画像とPDF、動画のプレビュー
const imagePreviewModal = ref(false);
const imageFileSrc = ref('');
const pdfFileSrc = ref('');
const previewSelectedImageId = ref('');
const onOpenImagePreviewModal = (filename: string, imageId: string): void => {
  // PDFファイルの場合
  if (isPDF(filename)) {
    if (pdfPreviewModal.value) {
      pdfFileSrc.value = '';
      pdfPreviewModal.value = false;
    } else {
      pdfFileSrc.value = `${baseUrl}/static/images/${filename}`;
      pdfPreviewModal.value = true;
      loadPdf(pdfFileSrc.value);
    }
    previewSelectedImageId.value = imageId;
    return;

    // 画像ファイルか動画の場合
  } else {
    imagePreviewModal.value = true;
    previewSelectedImageId.value = imageId;
    // 動画ファイルか否かを判定してimgタグかvideoタグか切り替え
    if (isMP4(filename)) {
      imageFileSrc.value = `<video controls="" src="${baseUrl}/static/images/${filename}" id="img-preview"></video><br>`;
    } else {
      imageFileSrc.value = `<img src="${baseUrl}/static/images/${filename}" id="img-preview" v-on:click="onImageCopyPath(previewSelectedImageId)"><br>`;
    }
  }
};

// 画像プレビューモーダルのクローズ
const onCloseImagePreviewModal = (): void => {
  imagePreviewModal.value = false;
};

const pdfPreviewModal = ref(false);
const pdfCanvases: Ref<{ ref: string }[]> = ref([]);
const loadPdf = async (url: string) => {
  clearCanvas();
  const { getDocument, GlobalWorkerOptions } = await import('pdfjs-dist');
  GlobalWorkerOptions.workerSrc = `${assetsUrl}pdf.worker.mjs`;
  const pdf = await getDocument({
    url,
    cMapUrl: assetsUrl,
    cMapPacked: true,
  }).promise;
  const numPages = pdf.numPages;

  // 必要な数のキャンバスを用意
  pdfCanvases.value = Array.from({ length: numPages }, (_, i) => ({
    ref: `pdfCanvas${i}`,
  }));

  for (let i = 1; i <= numPages; i++) {
    const page = await pdf.getPage(i);
    const viewport = page.getViewport({ scale: 1.0 });

    const canvasRef = pdfCanvases.value[i - 1]!.ref;
    const canvas = (document.getElementById(canvasRef) as HTMLCanvasElement) || null;

    if (canvas) {
      const context = canvas.getContext('2d');
      if (context) {
        canvas.height = viewport.height;
        canvas.width = viewport.width;

        const renderContext = {
          canvasContext: context,
          viewport: viewport,
        };
        await page.render(renderContext).promise;
      }
    }
  }
};

const clearCanvas = () => {
  pdfCanvases.value.forEach((canvasRefObj) => {
    const canvas = (document.getElementById(canvasRefObj.ref) as HTMLCanvasElement) || null;
    if (canvas) {
      const context = canvas.getContext('2d');
      if (context) {
        context.clearRect(0, 0, canvas.width, canvas.height);
      }
    }
  });
};

// PDFプレビューモーダルのクローズ
const onClosePDFPreviewModal = (): void => {
  clearCanvas();
  pdfPreviewModal.value = false;
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
    imageDeleteCheckModal.value = false;
    imagePreviewModal.value = false;
    pdfPreviewModal.value = false;
    previewSelectedImageId.value = '';
  } else {
    imageDeleteCheckModal.value = false;
  }
  return;
};

// 画像削除処理
const onImageDelete = async (id: string): Promise<void> => {
  try {
    const response = await apiClient.delete(imageDeleteUrl + `/${id}`);
    console.log(response);
    imageStore.deleteImage(id);
    messageModalOpenClose('削除しました。');
  } catch (error) {
    console.log(error);
    messageModalOpenClose('削除に失敗しました。');
  }
};

// 画像検索処理
const queryFormData = ref('');
const onSearch = (isClear: boolean = false): void => {
  try {
    if (isClear) {
      imageStore.queryImage('');
    } else {
      imageStore.queryImage(queryFormData.value);
    }
  } catch (error) {
    console.log(error);
  }
};

// Diff表示モーダルの表示非表示
const showDiffPreviewModal = ref(false);
const originalAreaRef = ref<HTMLElement | null>(null);
const modifiedAreaRef = ref<HTMLElement | null>(null);
const onOpenCloseDiffModal = async (isClose: boolean = false) => {
  if (isClose) {
    showDiffPreviewModal.value = false;
    return;
  }

  // 比較するテキスト
  const text1 = editConfirmationTitle + '\n\n' + editConfirmationBody;
  const text2 = updateWikiData.value.title + '\n\n' + content.value;

  if (text1 === text2) {
    messageModalOpenClose('変更はありません。');
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

// スクロールの同期処理
const formArea: Ref<HTMLElement | null> = ref(null);
const previewArea: Ref<HTMLElement | null> = ref(null);
onMounted(() => {
  formArea.value?.addEventListener('scroll', function (this: HTMLElement) {
    if (previewArea.value && this.scrollTop !== undefined) {
      previewArea.value.scrollTop = this.scrollTop;
    }
  });

  previewArea.value?.addEventListener('scroll', function (this: HTMLElement) {
    if (formArea.value && this.scrollTop !== undefined) {
      formArea.value.scrollTop = this.scrollTop;
    }
  });
});

// プライベート・パブリックが変更された際にメッセージボックスを出現
const isPublicWatch = ref(updateWikiData.value.is_public);
watch(isPublicWatch, (): void => {
  if (isPublicWatch.value) {
    messageModalOpenClose(
      'パブリックに変更されました。この設定で作成したWikiは全ユーザーに共有され、閲覧可能となります。',
    );
  } else {
    messageModalOpenClose(
      'プライベートに変更されました。この設定で作成したWikiは投稿したユーザーだけが閲覧可能です。',
    );
  }
});
const isPublicChanged = (): void => {
  if (isPublicWatch.value) {
    isPublicWatch.value = false;
  } else {
    isPublicWatch.value = true;
  }
};

// ウィンドウサイズでエディタのサイズを自動調整
function useWindowSize() {
  const width = ref(window.innerWidth);
  const height = ref(window.innerHeight);

  const updateSize = () => {
    width.value = window.innerWidth;
    height.value = window.innerHeight;
  };
  onMounted(() => {
    window.addEventListener('resize', updateSize);
  });

  onBeforeUnmount(() => {
    window.removeEventListener('resize', updateSize);
  });
  return { width, height };
}

const { width, height } = useWindowSize();
const divHeight = ref(0);
if (height.value > 850) {
  divHeight.value = height.value * 0.68;
} else if (height.value > 820) {
  divHeight.value = height.value * 0.66;
} else if (height.value > 400) {
  divHeight.value = height.value * 0.66;
}

if (width.value < 770) {
  isPreview.value = false;
}

watch(height, (newHeight) => {
  if (newHeight > 800) {
    divHeight.value = newHeight * 0.68;
  } else {
    divHeight.value = newHeight * 0.68;
  }
});

watch(width, (newWidth) => {
  if (newWidth > 770) {
    const localStrageItem = getLocalStrageInfo();
    if (localStrageItem.isPreviewFromLocalStrage === 'true') {
      isPreview.value = true;
    }
  } else {
    isPreview.value = false;
  }
});

// 画像追加モーダル表示時に灰色の部分のクリック時にも画像追加モーダルを閉じる処理
// HTMLが描画後に組み込む（onmoutedを利用）
onMounted(() => {
  // オーバレイとヘルプの内容を取得
  const imgAddModal = document.getElementById('overlay-fileup');
  const imgAddModalContent = document.getElementById('content-fileup');
  // 灰色部分クリック時にクローズ処理がなされるようにイベント設定
  if (imgAddModal) {
    imgAddModal.addEventListener('click', function (event) {
      if (showFileUploadContent.value === true) {
        showFileUploadContent.value = false;
      } else {
        return;
      }
    });
  }
  // 灰色の部分以外（content-fileup）をクリックした時にはイベント伝搬を止め、クローズさせない
  if (imgAddModalContent) {
    imgAddModalContent.addEventListener('click', function (event) {
      event.stopPropagation();
    });
  }
});

// 画像一覧モーダルクローズ処理（オーバーレイをクリック時）
onMounted(() => {
  const imgListModal = document.getElementById('overlay-imagelist');
  const imgListModalContent = document.getElementById('content-image');
  if (imgListModal) {
    imgListModal.addEventListener('click', function (event) {
      if (showImageListContent.value === true) {
        showImageListContent.value = false;
      } else {
        return;
      }
    });
  }
  if (imgListModalContent) {
    imgListModalContent.addEventListener('click', function (event) {
      event.stopPropagation();
    });
  }
});

// 画像一覧モーダル（https or localhost）
onMounted(() => {
  const imageListHttpsModal = document.getElementById('overlay-image-https-list');
  const imageListHttpsModalContent = document.getElementById('content-image-https-list');
  if (imageListHttpsModal) {
    imageListHttpsModal.addEventListener('click', function (event) {
      if (showImageListHttpsModal.value === true) {
        showImageListHttpsModal.value = false;
      } else {
        return;
      }
    });
  }
  if (imageListHttpsModalContent) {
    imageListHttpsModalContent.addEventListener('click', function (event) {
      event.stopPropagation();
    });
  }
});

// 画像プレビューモーダルクローズ処理（オーバーレイをクリック時）
onMounted(() => {
  const imgPreviewModal = document.getElementById('overlay-image-preview');
  const imgPreviewModalContent = document.getElementById('content-image-view');
  if (imgPreviewModal) {
    imgPreviewModal.addEventListener('click', function (event) {
      if (imagePreviewModal.value === true) {
        imagePreviewModal.value = false;
      } else {
        return;
      }
    });
  }
  if (imgPreviewModalContent) {
    imgPreviewModalContent.addEventListener('click', function (event) {
      event.stopPropagation();
    });
  }
});

// メッセージモーダルクローズ処理（オーバーレイをクリック時）
onMounted(() => {
  const messageModal = document.getElementById('overlay-message');
  const messageModalContent = document.getElementById('content-message');
  if (messageModal) {
    messageModal.addEventListener('click', function (event) {
      if (isMessageModal.value === true) {
        isMessageModal.value = false;
      } else {
        return;
      }
    });
  }
  if (messageModalContent) {
    messageModalContent.addEventListener('click', function (event) {
      event.stopPropagation();
    });
  }
});

// Diffプレビューモーダルクローズ処理（オーバーレイをクリック時）
onMounted(() => {
  const diffViewModal = document.getElementById('overlay-diff');
  const diffViewModalContent = document.getElementById('content-diff');
  if (diffViewModal) {
    diffViewModal.addEventListener('click', function (event) {
      if (showDiffPreviewModal.value === true) {
        showDiffPreviewModal.value = false;
      } else {
        return;
      }
    });
  }
  if (diffViewModalContent) {
    diffViewModalContent.addEventListener('click', function (event) {
      event.stopPropagation();
    });
  }
});

// PDFプレビューモーダルクローズ処理（オーバーレイをクリック時）
onMounted(() => {
  const pdfModal = document.getElementById('overlay-pdf-preview');
  const pdfModalContent = document.getElementById('content-pdf-view');
  if (pdfModal) {
    pdfModal.addEventListener('click', function (event) {
      if (pdfPreviewModal.value === true) {
        onClosePDFPreviewModal();
      } else {
        return;
      }
    });
  }
  if (pdfModalContent) {
    pdfModalContent.addEventListener('click', function (event) {
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
  console.log('Clipboard Copied.');
  messageModalOpenClose('クリップボードにコピーしました。');
};

// ショートカットキーを追加
const handleKeyDown = (event: KeyboardEvent) => {
  // Preview.vueへ移動
  if (event.ctrlKey && event.key === '1') {
    event.preventDefault(); // デフォルトのブラウザのショートカットをキャンセル
    onOutCheck('preview');

    // List.vueへ移動
  } else if (event.ctrlKey && event.key === '2') {
    event.preventDefault();
    onOutCheck();

    // 画像挿入モーダルを表示
  } else if (event.ctrlKey && event.key === '3') {
    event.preventDefault();
    openFileUpModal();

    // 画像一覧モーダルを表示
  } else if (event.ctrlKey && event.key === '4') {
    event.preventDefault();
    if (isHttpsProtocol) {
      openCloseImageListHttpsModal();
    } else {
      openImageListModal();
    }

    // 計算式作成
  } else if (event.ctrlKey && event.key === '5') {
    event.preventDefault();
    onCloseKatexModal();

    // プレビューの表示非表示
  } else if (event.ctrlKey && event.key === '6') {
    event.preventDefault();
    handlePreview();

    // マークダウン入力ツールの表示非表示
  } else if (event.ctrlKey && event.key === '7') {
    event.preventDefault();
    handleInputTool();

    // プレビューの表示非表示
  } else if (event.ctrlKey && event.key === '8') {
    event.preventDefault();
    onOpenCloseDiffModal();

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
    if (isOwner.value) {
      updateWiki();
    } else {
      if (isRequestMessageModal.value) {
        editRequestWiki();
      }
      handleOpenCloseRequestMessageModal();
    }

    // Escapeキーでモーダルウィンドウをクローズ
  } else if (event.key === 'Escape') {
    event.preventDefault();
    if (isMessageModal.value) {
      isMessageModal.value = false;
    }
    if (showFileUploadContent.value) {
      showFileUploadContent.value = false;
    }
    if (imagePreviewModal.value) {
      imagePreviewModal.value = false;
      return;
    }
    if (showImageListContent.value) {
      showImageListContent.value = false;
    }
    if (showYesNoMessageContent.value) {
      showYesNoMessageContent.value = false;
    }
    if (isUpdateOkModal.value) {
      previewRedirect(wiki.value.id);
    }
    if (pdfPreviewModal.value) {
      onClosePDFPreviewModal();
    }
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

// マークダウン記号をエディタに挿入
function insertMarkdown(text: string) {
  const corsorPosition = editor.getCursorPosition();
  editor.session.insert(corsorPosition, text);
  editor.focus();

  if (isHttpsProtocol.value) {
    navigator.clipboard.writeText(text);
  }
}
</script>

<template>
  <div id="btn-head-zone">
    <div class="btn-head-left">
      <button
        class="btn-head-image"
        title="更新作業を終了してプレビュー画面へ遷移します。&#10;ショートカット: Ctrl + 1"
        v-on:click="onOutCheck('preview')"
      >
        <img :src="`${assetsUrl}preview_on_24.png`" class="btn-img" alt="preview_on_24.png" />
      </button>
      <button
        class="btn-head-image"
        title="Wiki一覧画面へ遷移します。&#10;ショートカット: Ctrl + 2"
        v-on:click="onOutCheck()"
      >
        <img :src="`${assetsUrl}home_24.png`" class="btn-img" alt="home_24.png" />
      </button>
      <button
        v-show="isOwner"
        class="btn-head-image"
        title="画像・PDFの挿入画面を表示&#10;ショートカット: Ctrl + 3"
        v-on:click="openFileUpModal"
      >
        <img
          :src="`${assetsUrl}smartphone_line24.png`"
          class="btn-img"
          alt="smartphone_line24.png"
        />
      </button>
      <button
        v-if="isHttpsProtocol || isOwner"
        class="btn-head-image"
        title="画像・PDFの一覧画面を表示&#10;ショートカット: Ctrl + 4"
        v-on:click="openCloseImageListHttpsModal"
      >
        <img :src="`${assetsUrl}documents_line24.png`" class="btn-img" alt="documents_line24.png" />
      </button>
      <button
        v-else="isHttpsProtocol || isOwner"
        class="btn-head-image"
        title="画像・PDFの一覧画面を表示&#10;ショートカット: Ctrl + 4"
        v-on:click="openImageListModal"
      >
        <img :src="`${assetsUrl}documents_line24.png`" class="btn-img" alt="documents_line24.png" />
      </button>
      <button
        class="btn-head-image"
        title="計算式作成&#10;ショートカット: Ctrl + 5"
        v-on:click="onCloseKatexModal()"
      >
        <img :src="`${assetsUrl}math24.png`" class="btn-img" alt="math24.png" />
      </button>
      <button
        v-if="isPreview"
        class="btn-head-image"
        title="プレビュー切り替え&#10;ショートカット: Ctrl + 6"
        v-on:click="handlePreview()"
      >
        <img :src="`${assetsUrl}preview_off_24.png`" class="btn-img" alt="preview_off_24.png" />
      </button>
      <button
        v-else
        class="btn-head-image"
        title="プレビュー切り替え&#10;ショートカット: Ctrl + 6"
        v-on:click="handlePreview()"
      >
        <img :src="`${assetsUrl}preview_on_24.png`" class="btn-img" alt="preview_on_24.png" />
      </button>
      <button
        class="btn-head-image"
        title="マークダウン入力ツール&#10;ショートカット: Ctrl + 7"
        v-on:click="handleInputTool()"
      >
        <img :src="`${assetsUrl}markdown_24.png`" class="btn-img" alt="markdown_24.png" />
      </button>
    </div>
    <div id="btn-head-right">
      <button
        class="btn-head-image"
        title="編集前と編集後の差分を比較&#10;ショートカット: Ctrl + 8"
        v-on:click="onOpenCloseDiffModal()"
      >
        <img :src="`${assetsUrl}difference_24.png`" class="btn-img" alt="difference_24.png" />
      </button>
    </div>
  </div>

  <!-- 入力フォームエリア -->
  <div class="contants-area" :style="{ height: divHeight + 'px' }">
    <div
      class="left-area-isprev"
      :style="{ width: isPreview ? '50%' : '100%', marginRight: isPreview ? '10px' : '0px' }"
    >
      <div class="left-h3">
        <h3 v-if="isOwner" class="editor-and-preview-title" id="title_h3_1">Editor</h3>
        <h3 v-else class="editor-and-preview-title" id="title_h3_1_request">Editor</h3>
      </div>
      <div class="edit-area" :style="{ height: divHeight + 'px' }">
        <div class="title-input">
          <input
            type="text"
            id="title-input-text"
            class="title"
            placeholder="タイトル入力欄"
            required
            v-model="updateWikiData.title"
            title="Wikiのタイトル。&#10;タイトルは自動的に見出しレベル1「#」として反映されます。&#10;ショートカット: Ctrl + i"
          />
          <div
            v-if="isOwner"
            class="switch-btn-container"
            title="Wikiを閲覧可能なユーザーの範囲を切り替えます。&#10;プライベート：作成者アカウントのみ閲覧可能&#10;パブリック：全アカウントから閲覧可能（編集は作成者のみ）"
          >
            <label for="switch" class="switch-label">
              <div class="switch">
                <input
                  type="checkbox"
                  id="switch"
                  v-model="updateWikiData.is_public"
                  v-on:click="isPublicChanged"
                />
                <div class="base"></div>
                <div class="circle"></div>
                <div class="slider"></div>
              </div>
            </label>
          </div>
        </div>
        <div
          ref="editorRef"
          class="editor-div"
          id="editor"
          title="マークダウンエディター&#10;Wikiとして作成したい文書をマークダウンで記述します。&#10;作成はリアルタイムで左側のプレビューエリアに反映されます。"
        ></div>
      </div>
    </div>

    <div class="post-btn-and-switch">
      <button
        v-show="isOwner"
        type="submit"
        title="Wikiを更新&#10;ショートカット: Ctrl + m"
        class="btn-post"
        v-on:click.prevent="updateWiki"
      >
        + 更新
      </button>
      <button
        v-show="!isOwner"
        type="submit"
        title="Wikiの更新をリクエスト&#10;ショートカット: Ctrl + m"
        class="btn-post wide"
        v-on:click.prevent="handleOpenCloseRequestMessageModal"
      >
        + 変更をリクエスト
      </button>
    </div>

    <div class="right-area-preview" v-if="isPreview">
      <div class="right-h3">
        <h3 v-show="isOwner" class="editor-and-preview-title" id="title_h3_2">Preview</h3>
        <h3 v-show="!isOwner" class="editor-and-preview-title" id="title_h3_2_request">Preview</h3>
      </div>
      <div
        class="preview-area"
        id="result"
        ref="previewArea"
        :style="{ height: divHeight + 37 + 'px' }"
      >
        <section v-html="markDownConv"></section>
      </div>
    </div>
  </div>

  <!-- マークダウン入力支援ボタン -->
  <div class="input-tools" v-show="isShowTools" :style="{ right: isPreview ? '53%' : '5%' }">
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
      title="[Title](URL)を挿入"
      v-on:click="insertMarkdown('[Title](URL)')"
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

    <button
      class="btn-input-tools"
      title="チェックボックスを挿入"
      v-on:click="insertMarkdown('- [ ] ')"
    >
      <img
        :src="`${assetsUrl}check_box_24.png`"
        class="btn-input-tools-img"
        alt="check_box_24.png"
      />
    </button>
  </div>

  <!-- 画像アップロードモーダル -->
  <div id="overlay-fileup" v-show="showFileUploadContent">
    <div id="content-fileup">
      <h2 class="modal-h2">画像・PDF・動画アップロード</h2>
      <div class="table_sticky_file_upload">
        <table>
          <thead>
            <tr>
              <th>選択</th>
              <th>送信</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>
                <input
                  type="file"
                  accept="image/jpeg,image/png,image/webp,image/gif,video/mp4,application/pdf"
                  id="image1"
                  v-on:change="onImageSelect"
                />
              </td>
              <td>
                <button type="submit" class="btn-file-upload" v-on:click.prevent="imageFileSend">
                  アップロード
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="btn-zone">
        <button v-on:click="openFileUpModal">閉じる</button>
        <button v-on:click.prevent="imageCrear">選択解除</button>
      </div>
    </div>
  </div>

  <!-- 画像一覧モーダル（http）-->
  <div id="overlay-imagelist" v-show="showImageListContent">
    <div id="content-image" :style="{ width: imageList.size === 0 ? '40%' : '73%' }">
      <h2 class="modal-h2">画像・PDF・動画</h2>
      <div v-if="imageList.size !== 0" class="search-form">
        <div class="form-text">
          <input type="text" class="query1" placeholder="検索ワード" v-model="queryFormData" />
        </div>
        <div class="btn-img-search-clear-zone">
          <button class="btn-search-start-reset" type="submit" v-on:click.prevent="onSearch(false)">
            <img :src="`${assetsUrl}search_fill24.png`" class="btn-img" alt="search_fill24.png" />
          </button>
          <button class="btn-search-start-reset" type="submit" v-on:click.prevent="onSearch(true)">
            <img :src="`${assetsUrl}update_fill24.png`" class="btn-img" alt="update_fill24.png" />
          </button>
        </div>
      </div>
      <div v-if="imageList.size === 0" style="text-align: center">
        <p>画像コンテンツがありません。</p>
      </div>
      <div v-else class="table_sticky">
        <table>
          <thead>
            <tr>
              <th>Link</th>
              <th>Preview</th>
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
                v-if="isPDF(image.uuid_filename)"
                v-on:click.prevent="onOpenImagePreviewModal(image.uuid_filename, image.id)"
                class="td-img"
              >
                <img
                  :src="`${assetsUrl}picture_as_pdf.png`"
                  class="btn-img-table"
                  alt="picture_as_pdf.png"
                />
              </td>
              <td
                v-else="isPDF(image.uuid_filename)"
                v-on:click.prevent="onOpenImagePreviewModal(image.uuid_filename, image.id)"
                class="td-img"
              >
                <img :src="`${assetsUrl}camera24.png`" class="btn-img-table" alt="camera24.png" />
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="btn-close">
        <button v-on:click="openImageListModal">閉じる</button>
      </div>
    </div>
  </div>

  <!-- 画像一覧モーダル（https or localhost） -->
  <div id="overlay-image-https-list" v-show="showImageListHttpsModal">
    <div id="content-image-https-list" :style="{ width: imageList.size === 0 ? '40%' : '73%' }">
      <h2 class="modal-h2">画像・PDF・動画</h2>
      <div v-if="imageList.size !== 0" class="search-form">
        <div class="form-text">
          <input type="text" class="query1" placeholder="検索ワード" v-model="queryFormData" />
        </div>
        <div class="btn-img-search-clear-zone">
          <button class="btn-search-start-reset" type="submit" v-on:click.prevent="onSearch(false)">
            <img :src="`${assetsUrl}search_fill24.png`" class="btn-img" alt="search_fill24.png" />
          </button>
          <button class="btn-search-start-reset" type="submit" v-on:click.prevent="onSearch(true)">
            <img :src="`${assetsUrl}update_fill24.png`" class="btn-img" alt="update_fill24.png" />
          </button>
        </div>
      </div>
      <div v-if="imageList.size === 0" style="text-align: center">
        <p>画像コンテンツがありません。</p>
      </div>
      <div v-else class="table_sticky">
        <table>
          <thead>
            <tr>
              <th>Link</th>
              <th>Preview</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="[id, image] in imageList" v-bind:key="id">
              <td v-on:click="onImageCopyPath(image.id)">{{ image.filename }}</td>
              <td
                v-if="isPDF(image.filename)"
                v-on:click.prevent="onOpenImagePreviewModal(image.uuid_filename, image.id)"
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
                v-on:click.prevent="onOpenImagePreviewModal(image.uuid_filename, image.id)"
                class="td-img"
              >
                <img :src="`${assetsUrl}camera24.png`" class="btn-img-table" alt="camera24.png" />
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="btn-close">
        <button v-on:click="openCloseImageListHttpsModal()">閉じる</button>
      </div>
    </div>
  </div>

  <!-- 画像プレビュー -->
  <div id="overlay-image-preview" v-show="imagePreviewModal">
    <div id="content-image-view">
      <section v-html="imageFileSrc"></section>
      <div class="btn-zone">
        <button v-on:click.prevent="onOpenImageDeleteModal()">削除</button>
        <button v-on:click.prevent="onCloseImagePreviewModal()">閉じる</button>
      </div>
    </div>
  </div>

  <!-- PDFプレビュー -->
  <div id="overlay-pdf-preview" v-show="pdfPreviewModal">
    <div id="content-pdf-view">
      <div v-for="(canvas, index) in pdfCanvases" :key="index">
        <canvas :id="canvas.ref"></canvas>
      </div>
      <div class="btn-zone">
        <button v-on:click.prevent="onOpenImageDeleteModal()">削除</button>
        <button v-on:click.prevent="onClosePDFPreviewModal()">閉じる</button>
      </div>
    </div>
  </div>

  <!-- 画像削除確認モーダル -->
  <transition>
    <div id="overlay-delete-image" v-if="imageDeleteCheckModal">
      <div id="content-delete-image">
        <h2 class="modal-h2">最終確認</h2>
        <p><strong>本当に削除してもよろしいですか?</strong></p>
        <div class="btn-zone">
          <button v-on:click="onCloseImageDeleteModal(1)">削除</button>
          <button v-on:click="onCloseImageDeleteModal(0)">やめる</button>
        </div>
      </div>
    </div>
  </transition>

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

  <!-- アップロード完了モーダル -->
  <div id="overlay-uploaded-message" v-show="isUploadedMessageModal">
    <div id="content-uploaded-message">
      <h2 class="modal-h2">メッセージ</h2>
      <div class="input-text-zone">
        <p><strong>アップロード完了。次のテキストリンクをコピーして使用してください。</strong></p>
        <pre><code :id=uploadedUniqueFileName v-on:click="selectTextOrClipboardCopy(`${uploadedUniqueFileName}`)">{{ uploadedUrl }}</code></pre>
      </div>
      <div class="btn-zone">
        <button id="message-close-btn" v-on:click="uploadMessageModalOpenClose('', '')">
          閉じる
        </button>
      </div>
    </div>
  </div>

  <!-- データ変更時の画面遷移 YES or NO メッセージモーダル -->
  <transition>
    <div id="overlay-warn-message" v-show="showYesNoMessageContent">
      <div id="content-warn-message">
        <h2 class="modal-h2">メッセージ</h2>
        <p>
          <strong
            >変更されたデータがあります。この操作を継続（画面の移動）した場合、変更は失われます。よろしいですか?</strong
          >
        </p>
        <div class="btn-zone">
          <button v-on:click="onCloseModal(1)">はい</button>
          <button v-on:click="onOpenCloseDiffModal()">変更を確認</button>
          <button v-on:click="onCloseModal(0)">いいえ</button>
        </div>
      </div>
    </div>
  </transition>

  <!-- 変更リクエストメッセージ -->
  <div id="overlay-request-message" v-show="isRequestMessageModal">
    <div id="content-request-message">
      <h2 class="modal-h2">変更リクエストメッセージ</h2>
      <textarea id="message-textarea" v-model="requestMessage"></textarea>
      <div class="btn-zone">
        <button v-on:click.prevent="handleOpenCloseRequestMessageModal">キャンセル</button>
        <button v-on:click.prevent="editRequestWiki">送信</button>
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
    @click.self="onOpenCloseDiffModal()"
  >
    <div id="content-diff">
      <header class="diff-header">
        <div class="diff-header__title">
          <h2 id="diff-title">差分比較</h2>
          <p class="diff-header__sub">変更前 / 変更後</p>
        </div>
        <button type="button" v-on:click="onOpenCloseDiffModal(true)">閉じる</button>
      </header>

      <div class="diff-grid">
        <section class="diff-panel">
          <div class="diff-panel__head">
            <h3>変更前</h3>
          </div>
          <div class="diff-panel__body" ref="originalAreaRef"></div>
        </section>

        <section class="diff-panel">
          <div class="diff-panel__head">
            <h3>変更後</h3>
          </div>
          <div class="diff-panel__body" ref="modifiedAreaRef"></div>
        </section>
      </div>
    </div>
  </div>

  <!-- 更新完了モーダル -->
  <div id="overlay-updated-message" v-show="isUpdateOkModal">
    <div id="content-updated-message">
      <h2 class="modal-h2">更新完了</h2>
      <div class="input-text-zone">
        <p><strong>Wikiの内容を更新しました。</strong></p>
      </div>
      <div class="btn-close">
        <button v-on:click="previewRedirect(wiki.id)">閉じる</button>
      </div>
    </div>
  </div>

  <!-- 更新申請完了モーダル -->
  <div id="overlay-updated-message" v-show="isEditRequestOkModal">
    <div id="content-updated-message">
      <h2 class="modal-h2">申請完了</h2>
      <div class="input-text-zone">
        <p><strong>Wikiの更新を申請しました。</strong></p>
      </div>
      <div class="btn-close">
        <button v-on:click="previewRedirect(wiki.id)">閉じる</button>
      </div>
    </div>
  </div>

  <!-- プログレスモーダル -->
  <div id="overlay-progress-bar" v-show="showProgressModal">
    <svg class="spinner" width="50" height="50" view-box="0 0 50 50" aria-hidden="true">
      <g transform="rotate(-90 25 25)">
        <circle
          cx="25"
          cy="25"
          r="20"
          fill="none"
          stroke="#76c7c0"
          stroke-width="5"
          stroke-linecap="round"
          stroke-dasharray="31.4 31.4"
        />
      </g>
    </svg>
  </div>

  <!-- 数式作成モーダル -->
  <div id="overlay-katex-preview" v-show="katexPreviewModal">
    <div id="content-katex-view">
      <h2 class="modal-h2">数式作成</h2>
      <div ref="mathContainer" v-html="renderedFormula" id="content-katex"></div>
      <textarea
        class="input-katex"
        id="input-katex"
        name="input-katex"
        v-model="formula"
      ></textarea>
      <div :class="{ 'btn-zone': isGenMath, 'btn-close': !isGenMath }">
        <button v-on:click.prevent="onCloseKatexModal()">閉じる</button>
        <button v-on:click.prevent="saveMathImage()" v-if="isGenMath">保存</button>
        <button v-on:click.prevent="insertMathImage()" v-if="isGenMath">挿入</button>
      </div>
    </div>
  </div>

  <footer>
    <p class="login-user">ログインユーザー：{{ currentUser }}</p>
  </footer>
</template>

<style scoped>
h3 {
  /* h3タグのテキストを左寄せにする */
  text-align: left;
  margin-bottom: 0;
}

#btn-head-zone {
  display: flex;
  justify-content: space-between;
}

.v-enter-active,
.v-leave-active {
  transition: all 0.3s ease-in-out;
}

.v-enter-from,
.v-leave-to {
  opacity: 0;
}

.btn-img-table {
  border: none;
  box-shadow: none;
  max-width: 100%;
  scale: 0.9;
}
.btn-img-table:hover {
  opacity: 0.5;
}
.td-img {
  text-align: center;
}

.contants-area {
  display: flex;
}

/* 画面左側エリア */
.left-area-isprev {
  height: 100%;
}

.left-h3 {
  width: 100%;
}

h3#title_h3_1 {
  position: relative;
  padding-left: 25px;
  margin-bottom: 10px;
  border-bottom: 0;
  text-shadow: 2px 1px 2px rgb(165, 165, 165);
  margin: 10px 0 10px;
}

h3#title_h3_1:before {
  position: absolute;
  content: '';
  bottom: -1px;
  left: 0;
  width: 0;
  height: 0;
  border: none;
  border-left: solid 15px transparent;
  border-bottom: solid 15px rgb(36, 128, 36);
}

h3#title_h3_1:after {
  position: absolute;
  content: '';
  bottom: -3px;
  left: 0px;
  width: 100%;
  border-bottom: solid 3px rgb(36, 128, 36);
}

h3#title_h3_1_request {
  position: relative;
  padding-left: 25px;
  margin-bottom: 10px;
  border-bottom: 0;
  text-shadow: 2px 1px 2px rgb(165, 165, 165);
  margin: 10px 0 10px;
}

h3#title_h3_1_request:before {
  position: absolute;
  content: '';
  bottom: -1px;
  left: 0;
  width: 0;
  height: 0;
  border: none;
  border-left: solid 15px transparent;
  border-bottom: solid 15px rgb(187, 84, 16);
}

h3#title_h3_1_request:after {
  position: absolute;
  content: '';
  bottom: -3px;
  left: 0px;
  width: 100%;
  border-bottom: solid 3px rgb(187, 84, 16);
}

.title-input {
  display: flex;
  margin-bottom: 5px;
}

/* Aceエディタの上にモーダルを出した際の崩れ（スクロールバーが前面に現れる）を解消 */
.ace_editor {
  z-index: 0;
  height: 100%;
  isolation: isolate;
}

#editor {
  border: solid 1px rgb(184, 184, 184);
}

.title {
  height: 28px;
  width: 100%;
  border-radius: 5px;
  border: solid 1px rgb(184, 184, 184);
  font-size: 16px;
  margin-right: 5px;
  padding-left: 12px;
}

.title:focus {
  border-color: #007bff;
  box-shadow: 0 0 5px rgba(0, 123, 255, 0.5);
  outline: none;
}

.editor-div {
  border-radius: 5px;
  border: solid 0.5px;
}

.post-btn-and-switch {
  position: absolute;
  z-index: 1;
  bottom: 9%;
  right: 3%;
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
  width: 56px;
  border-radius: 16px;
  height: 32px;
  background-color: #ddd;
}

.switch-title {
  margin-left: 4px;
  font-size: 16px;
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
  top: 4px;
  left: 4px;
  width: 24px;
  height: 24px;
  border-radius: 12px;
  background-color: white;
  transition: 0.5s;
}

.switch {
  position: relative;
}

.switch-btn-container {
  display: flex;
  justify-content: space-between;
}

.private-public-label {
  font-size: 16px;
  margin-left: 5px;
  margin-top: 6px;
  font-weight: bold;
}

.private-public-label p {
  text-shadow: 1px 1px 2px rgb(202, 202, 202);
}

.btn-post {
  width: 100%;
  height: 50px;
  background: rgb(23, 155, 126);
  box-shadow: 3px 3px 5px 0 rgba(75, 75, 75, 0.5);
  color: #fff;
  font-size: 16px;
  padding: 9px 7px;
  text-decoration: none;
  border: 1px;
  border-radius: 20px;
  transition-property: opacity;
  -webkit-transition-property: opacity;
  transition-duration: 0.5s;
  transition: background-color 0.3s;
  -webkit-transition-duration: 0.5s;
  margin: 5px 5px 5px 5px;
}

.btn-post:hover {
  background: rgb(9, 78, 62);
}

/* 画面右側エリア */
.editor-and-preview-title {
  font-size: 18px;
}

.right-h3 {
  width: 100%;
}

h3#title_h3_2 {
  position: relative;
  padding-left: 25px;
  margin-bottom: 2%;
  border-bottom: 0;
  text-shadow: 2px 1px 2px rgb(165, 165, 165);
  margin: 10px 0 10px;
}

h3#title_h3_2:before {
  position: absolute;
  content: '';
  bottom: -1px;
  left: 0;
  width: 0;
  height: 0;
  border: none;
  border-left: solid 15px transparent;
  border-bottom: solid 15px rgb(36, 128, 36);
}

h3#title_h3_2:after {
  position: absolute;
  content: '';
  bottom: -3px;
  left: 0px;
  width: 100%;
  border-bottom: solid 3px rgb(36, 128, 36);
}

h3#title_h3_2_request {
  position: relative;
  padding-left: 25px;
  margin-bottom: 2%;
  border-bottom: 0;
  text-shadow: 2px 1px 2px rgb(165, 165, 165);
  margin: 10px 0 10px;
}

h3#title_h3_2_request:before {
  position: absolute;
  content: '';
  bottom: -1px;
  left: 0;
  width: 0;
  height: 0;
  border: none;
  border-left: solid 15px transparent;
  border-bottom: solid 15px rgb(187, 84, 16);
}

h3#title_h3_2_request:after {
  position: absolute;
  content: '';
  bottom: -3px;
  left: 0px;
  width: 100%;
  border-bottom: solid 3px rgb(187, 84, 16);
}

.right-area-preview {
  width: 52%;
  height: 100%;
}

.preview-area {
  overflow-y: auto;
  border-radius: 5px;
  padding: 0 20px;
  background-color: #ffffff;
}

/* データ変更時遷移制御モーダル */
#overlay-warn-message {
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

#content-warn-message {
  z-index: 2;
  width: 40%;
  padding: 1em;
  background: #fff;
  border-radius: 10px;
  text-align: center;
}

/* 画像一覧モーダル */
#overlay-imagelist,
#overlay-image-https-list {
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

/* 画像一覧モーダルのコンテンツ */
#content-image,
#content-image-https-list {
  z-index: 2;
  width: 70%;
  padding: 1em;
  background: #fff;
  border-radius: 10px;
}

/* 画像プレビューモーダル */
#overlay-image-preview {
  z-index: 2;
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

/* 画像プレビューモーダルのコンテンツ（実際の画像） */
#content-image-view {
  z-index: 3;
  max-width: 50%;
  height: auto;
  width: auto;
  padding: 1em;
  background: #fff;
  text-align: center;
  border-radius: 10px;
}

/* PDFプレビューモーダル */
#overlay-pdf-preview {
  z-index: 2;
  overflow-y: auto;
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

/* PDFプレビューモーダルのコンテンツ（実際の画像） */
#content-pdf-view {
  z-index: 3;
  overflow: auto;
  height: auto;
  max-height: 90%;
  /* 縦横比を維持 */
  width: auto;
  /* 縦横比を維持 */
  padding: 1em;
  background: #fff;
  text-align: center;
  border-radius: 10px;
}

canvas {
  border: 1px solid black;
  margin-bottom: 10px;
}

/* テーブルのホバー：ボディ部分の行のみホバー時のスタイルを適用 */
.table_sticky table tbody tr:hover {
  background-color: #c1d1d6;
}

/* 画像一覧テーブル */
.table_sticky table {
  margin-top: 0;
}

.table_sticky {
  display: block;
  overflow-y: auto;
  height: 40vh;
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

/* 画像削除モーダル */
#overlay-delete-image {
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

#content-delete-image {
  z-index: 3;
  width: 30%;
  padding: 1em;
  background: #fff;
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

.search-form {
  display: flex;
  margin-top: 1%;
}

.btn-img-search-clear-zone {
  display: flex;
}

.btn-search-start-reset {
  display: flex;
  justify-content: center;
  align-items: center;
  cursor: pointer;
  background: #ffffff;
  width: 50px;
  height: 45px;
  box-shadow: 3px 3px 5px 0 rgba(75, 75, 75, 0.5);
  text-decoration: none;
  border: 1px solid rgb(207, 207, 207);
  border-radius: 50%;
  padding: 0;
  transition-duration: 0.5s;
  -webkit-transition-duration: 0.5s;
  transition: background-color 0.3s;
  margin-left: 8px;
  margin-right: 2%;
}

.btn-search-start-reset:hover {
  background: rgb(235, 235, 235);
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

/* 画像アップロードモーダル */
#overlay-fileup {
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

/* 画像アップロードモーダルのコンテンツ */
#content-fileup {
  z-index: 2;
  width: 40%;
  padding: 1em;
  background: #fff;
  border-radius: 10px;
}

/* 画像アップロードモーダルのテーブル */
.table_sticky_file_upload table {
  margin-top: 0;
}

.table_sticky_file_upload {
  display: block;
  overflow-y: auto;
  height: 100%;
  margin-top: 1%;
}

.table_sticky_file_upload thead th {
  position: sticky;
  top: 0;
  width: 100%;
  z-index: 1;
  background: rgb(44, 52, 78);
  color: whitesmoke;
}

/* アップロードボタン */
.btn-file-upload {
  width: 110px;
  background: rgb(28, 58, 190);
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

.btn-file-upload:hover {
  background: rgb(16, 34, 112);
}

.search-form {
  display: flex;
  margin-top: 1%;
}

.form-text {
  margin-top: 1px;
  margin-right: 1%;
}

.query1 {
  margin-top: 1px;
  margin-right: 1%;
  font-size: 17px;
  width: 100%;
  padding: 0.6em 1.2em;
  display: flex;
  text-align: center;
  margin-bottom: 2%;
  border-radius: 5px;
  box-sizing: border-box;
}

.query1:focus {
  outline: none;
  border-color: #007bff;
  box-shadow: 0 0 5px rgba(0, 123, 255, 0.5);
}

#overlay-request-message {
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

#content-request-message {
  z-index: 2;
  width: 35%;
  padding: 1em;
  background: #fff;
  border-radius: 10px;
}

#message-textarea {
  width: 100%;
  min-height: 150px;
  padding: 1em;
  margin: 1em 0;
  font-family: 'Consolas', 'Menlo', monospace;
  font-size: 0.95rem;
  line-height: 1.6;
  color: #222;
  background-color: #fdfdfd;
  border: 1px solid #ccc;
  border-radius: 6px;
  resize: vertical;
  box-sizing: border-box;
}

#message-textarea:focus {
  outline: none;
  border-color: #666;
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

#text-original-area,
#text-modified-area {
  flex-grow: 1;
  width: 50%;
  height: 100%;
  padding: 10px;
  white-space: pre-wrap;
  text-align: left;
  font-size: 16px;
}

#text-original-area {
  border-right: solid 1px rgb(160, 160, 160);
}

/* 更新完了メッセージモーダル */
#overlay-updated-message {
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

#content-updated-message {
  z-index: 4;
  width: 20%;
  padding: 1em;
  background: whitesmoke;
  border-radius: 10px;
  text-align: center;
}

.btn-post-complate {
  text-align: center;
}

/* 画像アップロードメッセージモーダル */
#overlay-uploaded-message {
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

#content-uploaded-message {
  z-index: 4;
  width: 35%;
  padding: 1em;
  background: whitesmoke;
  border-radius: 10px;
  text-align: center;
}

/* 数式作成モーダル */
#overlay-katex-preview {
  z-index: 2;
  overflow-y: auto;
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

/* 数式作成モーダル */
#content-katex-view {
  z-index: 3;
  width: 40%;
  height: auto;
  padding: 1em;
  background: #fff;
  text-align: center;
  border-radius: 10px;
}

#content-katex {
  font-size: 24px; /* 固定サイズにする */
  display: inline-block;
  padding: 10px;
  margin: 50px;
  background: white; /* 背景を白にすることで透明化を防ぐ */
}

.input-katex {
  font-size: 20px;
  width: 100%;
  border-radius: 8px;
  border: 1px solid #999999;
  padding: 0.6em 1.2em;
  font-family: inherit;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  box-sizing: border-box;
  height: 15vh;
}

.input-katex:focus {
  outline: none;
  border-color: #007bff;
  box-shadow: 0 0 5px rgba(0, 123, 255, 0.5);
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
</style>
