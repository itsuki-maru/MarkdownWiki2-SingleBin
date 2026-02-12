<script setup lang="ts">
import type { UpdateWikiData, WikiData, ImageData } from '@/interface';
import { ref, computed, watch, onUnmounted, onMounted, nextTick } from 'vue';
import { useRouter } from 'vue-router';
import { updateWikiUrl } from '@/router/urls';
import { AxiosError } from 'axios';
import { useWikiStore } from '@/stores/wikis';
import { useImageStore } from '@/stores/images';
import { useEditRequestWikiStore } from '@/stores/editWikis';
import {
  imageUploadUrl,
  imageDeleteUrl,
  wikiOwnerGetUrl,
  postEditWikiRequestUrl,
  getUserUrl,
} from '@/router/urls';
import { baseUrl, assetsUrl } from '@/setting';
import { marked, Renderer } from 'marked';
import { videoToken } from '@/utils/markedSetup';
import apiClient from '@/axiosClient';

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
const isHttpsProtocol = ref(false);
// 現在のURLを取得
const currentUrl = window.location.href;
// URLを解析
const url = new URL(currentUrl);
// プロトコルとホスト名を取得
const protocol = url.protocol;
const hostname = url.hostname;
// HTTPSかlocalhost通信の場合の設定
if (protocol === 'https:') {
  isHttpsProtocol.value = true;
}
if (hostname === 'localhost') {
  isHttpsProtocol.value = true;
}

// AccessTokenが無効になった際にログイン画面へ飛ばすための機能
const isAuthToken = ref(true);
const loginModal = ref(false);
watch(isAuthToken, (): void => {
  if (isAuthToken.value === false) {
    loginModal.value = true;
  } else {
    loginModal.value = false;
  }
});

// 画像ファイルのデータ管理
const imageStore = useImageStore();
imageStore.initList();
// ImageStoreから取得したデータをMapオブジェクトとして保持
const imageList = computed((): Map<string, ImageData> => {
  return imageStore.imageList;
});

// Preview.vueへのリダイレクト
const router = useRouter();
const previewRedirect = (id: string): void => {
  router.push(`/wiki/preview/${id}`);
};

// Login.vueへのリダイレクト
const loginRedirect = (): void => {
  router.push('/account/login');
};

// List.vueへリダイレクト
const listRedirect = (): void => {
  router.push('/wiki/list');
};

interface Props {
  id: string;
}

const props = defineProps<Props>();

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

const wikiStore = useWikiStore();
const wiki = computed((): WikiData => {
  return wikiStore.getById(props.id);
});

// 更新対象Wikiデータの初期化
const updateWikiDataInit: UpdateWikiData = {
  id: wiki.value.id,
  title: wiki.value.title,
  body: wiki.value.body,
  is_public: wiki.value.is_public,
};
const updateWikiData = ref(updateWikiDataInit);

// 初期データから変更があるか比較するための定数
const editConfirmationTitle = updateWikiDataInit.title;
const editConfirmationBody = updateWikiDataInit.body;

// 初期データと現在のデータに変更があるか比較する関数
// 変更があればtrueを返却
function checkingEditConfirm(): boolean {
  if (
    editConfirmationTitle === updateWikiData.value.title &&
    editConfirmationBody === updateWikiData.value.body
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
  if (isWikiUpdateSendNow.value === true) {
    return;
  } else {
    isWikiUpdateSendNow.value = true;
  }

  const id = updateWikiData.value.id;
  const title = updateWikiData.value.title;
  const body = updateWikiData.value.body;

  // パブリック・プライベートの切り替えUIから取得
  let is_public = false;
  if (updateWikiData.value.is_public) {
    is_public = true;
  } else {
    is_public = false;
  }

  if (title == '' || body == '') {
    messageModalOpenClose('入力データがありません。');
    isWikiUpdateSendNow.value = false;
    return;
  }

  const data = {
    title: title,
    body: body,
    is_public: is_public,
  };

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
  } catch (error: unknown) {
    if (typeof error === 'object' && error !== null) {
      const axiosError = error as AxiosError;

      if (axiosError.response) {
        console.error('Status code:', axiosError.response.status);
        console.error('Error data:', axiosError.response.data);
        if (axiosError.response.status === 401) {
          window.alert('不正な操作です。\nオーナーでないデータは編集できません。');
          localStorage.setItem('loginUser', '');
          localStorage.setItem('isAuthenticate', 'false');
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
  const body = updateWikiData.value.body;
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

// 画像アップロード
// 画像アップロードのモーダル表示・非表示を管理
const showImageUploadModal = ref(false);
const openCloseImageUpModal = (): void => {
  if (showImageUploadModal.value === true) {
    showImageUploadModal.value = false;
  } else {
    showImageUploadModal.value = true;
  }
};

const isImageSendNow = ref(false); // クリック連打の抑制とプログレス表示
watch(isImageSendNow, (): void => {
  if (isImageSendNow.value) {
    showProgressModal.value = true;
  } else {
    showProgressModal.value = false;
  }
});

const selectedImageBlob = ref<Blob | null>(null); // リサイズ後のBlobを保持
const selectedFileName = ref<string>('');

// 画像選択時にリサイズ処理
const onImageSelect = async (): Promise<void> => {
  const element = document.getElementById('image1')! as HTMLInputElement;
  if (element.value === '' || element.value === null) {
    messageModalOpenClose('画像ファイルを選択してください。');
    return;
  }

  // ファイルオブジェクトを取得してペイロードに追加
  const file = element.files!;
  const fileObj = file[0]!;
  const fileName = fileObj.name;

  // mime-typeで許可ファイルをフィルタリング
  const arrowMimeTypes = [
    'image/jpeg',
    'image/png',
    'image/webp',
    'image/gif',
    'video/mp4',
    'application/pdf',
  ];
  if (!arrowMimeTypes.includes(fileObj.type)) {
    messageModalOpenClose('許可されていない形式のファイルです。');
    imageCrear();
    return;
  }

  // 画像ファイルの場合
  if (fileObj.type.startsWith('image/')) {
    try {
      showProgressModal.value = true;
      // ブラウザネイティブでリサイズ
      selectedImageBlob.value = await resizeImageWithCanvas(fileObj);
    } catch (error) {
      console.error('リサイズエラー: ', error);
      selectedImageBlob.value = null;
    } finally {
      showProgressModal.value = false;
    }
    // 画像ファイル以外の場合
  } else {
    selectedImageBlob.value = fileObj;
  }
  selectedFileName.value = fileName;
};

// リサイズ処理機構
const resizeImageWithCanvas = (file: File): Promise<Blob> => {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => {
      // リサイズ対象の画像の最大幅と高さを定義（2K）
      const maxWidth = 1280;
      const maxHeight = 720;

      // リサイズ後のサイズを計算
      const { width, height } = caluculateDimensions(img.width, img.height, maxWidth, maxHeight);

      // Canvas作成
      const canvas = document.createElement('canvas');
      canvas.width = width;
      canvas.height = height;

      // Canvasに描画
      const ctx = canvas.getContext('2d');
      if (!ctx) {
        reject(new Error('Canvas contextの取得に失敗しました。'));
        return;
      }
      ctx.drawImage(img, 0, 0, width, height);

      // Blobとして出力
      canvas.toBlob(
        (blob) => {
          if (blob) {
            resolve(blob);
          } else {
            reject(new Error('Blobの生成に失敗しました。'));
          }
        },
        file.type,
        0.8,
      ); // 画質80%
    };
    img.onerror = () => reject(new Error('画像の読み込みに失敗しました。'));
    img.src = URL.createObjectURL(file); // ローカルファイルのURL
  });
};

// リサイズ後の幅と高さを計算
const caluculateDimensions = (
  width: number,
  height: number,
  maxWidth: number,
  maxHeight: number,
) => {
  // 横長画像の場合
  if (height < width) {
    if (width > maxWidth || height > maxHeight) {
      const widthRatio = maxWidth / width;
      const heightRatio = maxHeight / height;
      const ratio = Math.min(widthRatio, heightRatio);
      return {
        width: Math.floor(width * ratio),
        height: Math.floor(height * ratio),
      };
    }
    // サイズがすでに範囲内の場合
    return { width, height };

    // 縦長画像の場合
  } else {
    if (height > maxWidth || width > maxHeight) {
      const widthRatio = maxWidth / height;
      const heightRatio = maxHeight / width;
      const ratio = Math.min(widthRatio, heightRatio);
      return {
        width: Math.floor(width * ratio),
        height: Math.floor(height * ratio),
      };
    }
    // サイズがすでに範囲内の場合
    return { width, height };
  }
};

// 画像アップロード処理
const uploadImage = async (): Promise<void> => {
  if (isImageSendNow.value === true) {
    return;
  } else {
    isImageSendNow.value = true;
  }

  if (!selectedImageBlob.value) {
    messageModalOpenClose('ファイルを選択してください。');
    isImageSendNow.value = false;
    return;
  }

  // FormDataを初期化作成
  const payload = new FormData();

  if (selectedImageBlob.value) {
    payload.append('upload_file', selectedImageBlob.value, selectedFileName.value);
  }

  // axiosによる送信処理
  try {
    const response = await apiClient.post(imageUploadUrl, payload);

    const newImageData: ImageData = {
      id: response.data['new_image_id'],
      user_id: response.data['user_id'],
      filename: response.data['filename'],
      uuid_filename: response.data['uuid_filename'],
    };
    imageStore.addImage(newImageData);

    const uniqueFileName = response.data['uuid_filename'];

    let imageUrlMarkdown = '';
    if (isMP4(uniqueFileName)) {
      imageUrlMarkdown = `?[${selectedFileName.value}](${baseUrl}/static/images/${uniqueFileName})`;
    } else {
      if (isPDF(uniqueFileName)) {
        imageUrlMarkdown = `[${selectedFileName.value}](${baseUrl}/static/images/${uniqueFileName})`;
      } else {
        imageUrlMarkdown = `![${selectedFileName.value}](${baseUrl}/static/images/${uniqueFileName})`;
      }
    }

    const textarea = document.getElementById('wiki-detail')! as HTMLTextAreaElement;
    if (textarea) {
      updateWikiData.value.body = updateWikiData.value.body + imageUrlMarkdown + '\n\n';
      messageModalOpenClose('アップロード完了。画像を挿入しました。');
    }

    imageStore.initList();
    imageCrear();
    return;
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
            console.error('Please remove spaces from the file name.', axiosError.response.data);
            break;
          case 401:
            console.error('No token provided.', axiosError.response.data);
            break;
          case 500:
            messageModalOpenClose(`${axiosError.response.data}`);
            console.error('Server error, please try again later', axiosError.response.data);
            break;
          default:
            messageModalOpenClose(`${axiosError.response.data}`);
            console.error(`An error occurred: ${status}`, axiosError.response.data);
            break;
        }
      }
    }
  } finally {
    selectedImageBlob.value = null;
    selectedFileName.value = '';
    isImageSendNow.value = false;
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

/** 選択した画像ファイルをクリア */
const imageCrear = (): void => {
  selectedFileName.value = '';
  selectedImageBlob.value = null;
  let imageContent = document.getElementById('image1')! as HTMLInputElement;
  if (imageContent.value === null) {
    return;
  } else {
    imageContent.value = '';
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
            isAuthToken.value = false;
            break;
          case 500:
            messageModalOpenClose(`${axiosError.response.data}`);
            console.error('Server error, please try again later', axiosError.response.data);
            break;
          default:
            messageModalOpenClose(`${axiosError.response.data}`);
            console.error(`An error occurred: ${status}`, axiosError.response.data);
            break;
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

// メッセージ表示モーダル機能
const isMessageModal = ref(false);
const messageText = ref('');
const messageModalOpenClose = (message: string): void => {
  if (!isMessageModal.value) {
    messageText.value = message;
    isMessageModal.value = true;
  } else {
    isMessageModal.value = false;
    messageText.value = '';
  }
};

// QRコード作成モーダルの描画
const showQRContent = ref(false);
const qrCodeText = ref('');
const isGenerateOk = ref(false);
// TypeScript でグローバル変数を使用する場合、型アサーションが必要
// QRCodeはindex.htmlでCDN経由で読み込み、既にページにグローバルとして存在するため、これを明示
const QRCode: any = (window as any).QRCode;

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
  qrcode.makeCode(text);
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

// メッセージモーダル表示時に灰色の部分のクリック時にもメッセージモーダルを閉じる処理
// HTMLが描画後に組み込む（onmoutedを利用）
onMounted(() => {
  // オーバレイとヘルプの内容を取得
  const imgUploadModal = document.getElementById('overlay-fileup');
  const imgUploadModalContent = document.getElementById('content-fileup');

  // 灰色部分クリック時にクローズ処理がなされるようにイベント設定
  if (imgUploadModal) {
    imgUploadModal.addEventListener('click', function (event) {
      if (showImageUploadModal.value === true) {
        showImageUploadModal.value = false;
      } else {
        return;
      }
    });
  }

  // 灰色の部分以外（content-fileup）をクリックした時にはイベント伝搬を止め、クローズさせない
  if (imgUploadModalContent) {
    imgUploadModalContent.addEventListener('click', function (event) {
      event.stopPropagation();
    });
  }
});

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

// 拡張子で動画ファイルか判定する関数
function isMP4(filename: string) {
  return /\.mp4$/i.test(filename);
}

// 拡張子でPDFファイルか判定する関数
function isPDF(filename: string) {
  return /\.pdf$/i.test(filename);
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
  // Preview.vueへ移動
  if (event.ctrlKey && event.key === '1') {
    event.preventDefault(); // デフォルトのブラウザのショートカットをキャンセル
    onOutCheck('preview');

    // Preview.vueへ移動
  } else if (event.ctrlKey && event.key === '2') {
    event.preventDefault();
    onOutCheck();

    // 画像挿入モーダル
  } else if (event.ctrlKey && event.key === '3') {
    event.preventDefault();
    openCloseImageUpModal();

    // 画像一覧モーダル
  } else if (event.ctrlKey && event.key === '4') {
    event.preventDefault();
    if (isHttpsProtocol) {
      openCloseImageListHttpsModal();
    } else {
      openCloseImageListModal();
    }

    // QRコード生成モーダル
  } else if (event.ctrlKey && event.key === '5') {
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

    if (isUpdateOkModal.value) {
      previewRedirect(props.id);
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
    <button class="btn-head-img" v-on:click="onOutCheck('preview')">
      <img :src="`${assetsUrl}preview_24.png`" class="btn-img" alt="preview_24.png" />
    </button>
    <button class="btn-head-img" v-on:click="onOutCheck()">
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
  </div>

  <!-- 入力フォーム -->
  <div class="main-container">
    <h3 id="title_h3_1">Editor</h3>
    <div class="title-row">
      <input
        class="title"
        id="title-input-text"
        type="text"
        required
        v-model="updateWikiData.title"
      />
    </div>
    <textarea
      class="textarea"
      ref="formArea"
      rows="45"
      required
      v-model="updateWikiData.body"
      id="wiki-detail"
    ></textarea>
    <div class="form-footer"></div>
    <div class="form-footer">
      <p class="switch-btn-container">
        <label for="switch" class="switch-label">
          <div class="switch">
            <input type="checkbox" id="switch" v-model="updateWikiData.is_public" />
            <div class="base"></div>
            <div class="circle"></div>
            <div class="slider"></div>
          </div>
          <span v-if="updateWikiData.is_public" class="switch-title">パブリック</span>
          <span v-else class="switch-title">プライベート</span>
        </label>
      </p>
      <button v-show="isOwner" type="submit" class="btn-post" v-on:click.prevent="updateWiki">
        + 更新
      </button>
      <button
        v-show="!isOwner"
        type="submit"
        class="btn-post"
        v-on:click.prevent="handleOpenCloseRequestMessageModal"
      >
        + 変更をリクエスト
      </button>
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

  <!-- 画像ファイルのアップロードモーダル -->
  <div id="overlay-fileup" v-show="showImageUploadModal">
    <div id="content-fileup">
      <h2 class="modal-h2">画像アップロード</h2>
      <div>
        <table class="file-select-table">
          <thead>
            <tr>
              <th>選択</th>
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
            </tr>
          </tbody>
        </table>
        <button type="submit" class="btn-file-upload" v-on:click.prevent="uploadImage()">
          アップロード
        </button>
      </div>
      <div class="btn-zone">
        <button v-on:click.prevent="openCloseImageUpModal()">閉じる</button>
        <button v-on:click.prevent="imageCrear()">解除</button>
      </div>
    </div>
  </div>

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
      <div class="btn-close">
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

  <!-- データ変更時の画面遷移 YES or NO メッセージモーダル -->
  <div id="overlay-warn-message" v-show="showYesNoMessageContent">
    <div id="content-warn-message">
      <h2 class="modal-h2">メッセージ</h2>
      <p>
        <strong
          >変更されたデータがあります。<br />画面を移動した場合、変更は失われますがよろしいですか？</strong
        >
      </p>
      <div class="btn-zone">
        <button v-on:click="onCloseModal(1)">はい</button>
        <button v-on:click="onCloseModal(0)">いいえ</button>
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
</template>

<style scoped>
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
  width: 70%;
  padding: 1em;
  background: #fff;
  border-radius: 10px;
  text-align: center;
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
  width: 80%;
  padding: 1em;
  background: whitesmoke;
  border-radius: 10px;
  text-align: center;
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
  width: 91%;
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
</style>
