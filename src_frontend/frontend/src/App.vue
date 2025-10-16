<script setup lang="ts">
import { reactive, provide, onMounted, onUnmounted, ref, watch } from "vue";
import type { LoginUser } from "./interface";
import { assetsUrl } from "@/setting";
import { useRouter } from "vue-router";
import ace from "ace-builds";
import "ace-builds/src-noconflict/ext-searchbox"; // Ctrl+Fで検索ボックスを使用するために必要なモジュール
import "ace-builds/src-noconflict/mode-markdown"; // Aceでマークダウンを使用するためのモジュール
import "ace-builds/src-noconflict/theme-monokai"; // Aceのテーマのモジュール
import UserPrivacySetting from "@/components/UserPrivacySetting.vue";
import { useApplicationInitStore } from "./stores/appInits";

const appInitStore = useApplicationInitStore();
const appTitle = ref(appInitStore.appInitData.appTitle)

// Login User Status Provide.
const loginUser: LoginUser = {
  isAuthenticated: false,
}
provide("loginUser", reactive(loginUser));

// List.vueへリダイレクト
const router = useRouter();
const listRedirect = (): void => {
  router.push("/wiki/list");
}

listRedirect();

// メモアイコンの表示非表示管理
const isShowMemoIcon = ref(true);
// 他の子コンポーネントで表示・非表示を切り替えられるようにprovide
provide("isShowMemoIcon", isShowMemoIcon);

// メモモーダルの描画
const showMemoContent = ref(false);
const onOpenCloseMemoModal = (): void => {
  if (showMemoContent.value === true) {
    showMemoContent.value = false;
  } else {
    showMemoContent.value = true;
    // カーソルのフォーカスがエディタ描画完了後になるようにsetTimeoutで遅延させる
    setTimeout(() => {
      editor.focus();
    }, 300);
  }
}

// Aceエディタを定義
const editorRef = ref<HTMLDivElement | null>(null);
let editor: any | null = null;

// contentの変化を監視
const content = ref("");
watch(content, (newContent) => {
  if (editor && editor.getValue() !== newContent) {
    editor.setValue(newContent, 1);
  }
});

// HTML描画後にAceエディタを反映
onMounted(() => {
  // Aceの設定
  if (editorRef.value) {
    editor = ace.edit(editorRef.value);
    editor.getSession().setMode("ace/mode/markdown");
    editor.getSession().setUseWrapMode(true);
    editor.setFontSize(18);
    // 80文字の縦ラインを消す
    editor.setShowPrintMargin(false);
  }
  // editorの変更を監視
  editor.on("change", () => {
    const newValue = editor.getValue();
    if (newValue !== content.value) {
      content.value = newValue;
    }
  });
});

onUnmounted(() => {
  if (editor) {
    editor.destroy()
  }
});

// QRコード作成モーダルの描画
const showQRContent = ref(false);
const qrCodeText = ref("");
const isGenerateOk = ref(false);
// TypeScript でグローバル変数を使用する場合、型アサーションが必要
// QRCodeはindex.htmlでCDN経由で読み込み、既にページにグローバルとして存在するため、これを明示
const QRCode: any = (window as any).QRCode;

// HTMLの描画後にqrcodeを設定
let qrcode: any;
onMounted(() => {
  qrcode = new QRCode(document.getElementById("qrcode"), {
    text: qrCodeText.value,
    width: 128,
    height: 128,
    colorDark: "#000000",
    colorLight: "#ffffff",
    correctLevel: QRCode.CorrectLevel.H
  });
});

watch(qrCodeText, () => {
  if (qrCodeText.value === "") {
    let qrElement = document.getElementById("qrcode") as HTMLElement | null;
    if (qrElement !== null) {
      const images = qrElement.querySelectorAll("img");
      images.forEach(img => img.style.display = "none");
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
      document.getElementById("qr-input-text")!.focus();
    }, 300);
  }
}

// QRCode作成関数
function generateQRCode(): void {
  const text = qrCodeText.value;
  if (text === "") {
    return;
  }

  qrcode.clear();
  qrcode.makeCode(text); // make another code.
}

// QRCode保存関数
function saveQRCode(): void {
  const canvas: any = document.querySelector("#qrcode canvas");
  if (canvas) {
    // canvas要素から画像のURLを生成
    const imageUrl = canvas.toDataURL("image/png").replace("image/png", "image/octet-stream");
    // ダウンロードリンクを作成
    const link = document.createElement("a");
    link.download = "qrcode.png";
    link.href = imageUrl;
    link.click();
  }
}

// メモモーダル表示時に灰色の部分のクリック時にもメモモーダルを閉じる処理
// HTMLが描画後に組み込む（onmoutedを利用）
onMounted(() => {
  // オーバレイとメモの内容を取得
  const memoModal = document.getElementById("overlay-memo");
  const memoModalContent = document.getElementById("content-memo");

  // 灰色部分クリック時にクローズ処理がなされるようにイベント設定
  if (memoModal) {
    memoModal.addEventListener("click", function (event) {
      if (showMemoContent.value === true) {
        showMemoContent.value = false
      } else {
        return;
      }
    });
  }

  // 灰色の部分以外（content-memo）をクリックした時にはイベント伝搬を止め、クローズさせない
  if (memoModalContent) {
    memoModalContent.addEventListener("click", function (event) {
      event.stopPropagation();
    });
  }
});

// QRコード生成モーダル
onMounted(() => {
  const genQRCodeModal = document.getElementById("overlay-gen-qrcode");
  const genQRCodeModalContent = document.getElementById("content-gen-qrcode");
  if (genQRCodeModal) {
    genQRCodeModal.addEventListener("click", function (event) {
      if (showQRContent.value === true) {
        showQRContent.value = false
      } else {
        return;
      }
    });
  }
  if (genQRCodeModalContent) {
    genQRCodeModalContent.addEventListener("click", function (event) {
      event.stopPropagation();
    });
  }
});

// メモ機能呼び出しのショートカットキーを追加
const handleKeyDown = (event: KeyboardEvent) => {
  if (event.altKey && event.key === "m") {
    event.preventDefault() // デフォルトのブラウザのショートカットをキャンセル
    onOpenCloseMemoModal();
  } else if (event.altKey && event.key === "q") {
    event.preventDefault() // デフォルトのブラウザのショートカットをキャンセル
    onOpenCloseQRCodeCreateModal();
  }
};

// コンポーネントマウント時にイベントリスナーを追加
onMounted(() => {
  window.addEventListener("keydown", handleKeyDown);
});

// コンポーネントがアンマウントされた際にイベントリスナーを削除
onUnmounted(() => {
  window.removeEventListener("keydown", handleKeyDown);
});

const showSplashScreen = ref(true);
// スプラッシュスクリーンを3秒後に非表示にする
onMounted(() => {
  setTimeout(() => {
    showSplashScreen.value = false;
  }, 1800);
});

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
</script>

<template>
  <div class="container">
    <div v-if="showSplashScreen" id="splash-screen">
      <img :src="`${assetsUrl}icon-512x512.png`" alt="App Logo" class="logo"/>
      <h1 id="splash-title">MarkdownWiki2</h1>
    </div>
    <header class="parent-header">
      <h1 class="app-header" id="application-title">{{ appTitle }}</h1>
      <div v-show="isShowMemoIcon">
        <button class="btn-memo-open-close btn-hover"
          title="メモ機能&#10;記述したメモは各画面にまたがって確認できます。&#10;メモはブラウザを閉じるまで保存されます。&#10;ショートカット: Alt + M"
          v-on:click="onOpenCloseMemoModal"><img :src="`${assetsUrl}fillable_card_line24.png`" class="btn-img"
            alt="fillable_card_line24.png"></button>
        <button class="btn-qrcode-creater-open-close btn-hover"
          title="QRコード生成機能&#10;入力したテキストからQRコードを生成します。&#10;ショートカット: Alt + Q"
          v-on:click="onOpenCloseQRCodeCreateModal"><img :src="`${assetsUrl}code_reader_line24.png`" class="btn-img"
            alt="code_reader_line24.png"></button>
      </div>
    </header>
    <RouterView />

    <!-- メモモーダル -->
    <transition>
      <div id="overlay-memo" v-show="showMemoContent">
        <div id="content-memo">
          <h1 class="memo-title">MEMO</h1>
          <div ref="editorRef" class="editor-div" id="editor"></div>
        </div>
      </div>
    </transition>

    <!-- QR生成モーダル -->
    <transition>
      <div id="overlay-gen-qrcode" v-show="showQRContent">
        <div id="content-gen-qrcode">
          <h2 class="modal-h2">QRコード生成</h2>
          <div class="setting-contents">
            <div id="qrcode" class="qrcode"></div>
            <input type="text" maxlength="150" title="" id="qr-input-text" placeholder="Input Text."
                  class="input-textbox" required v-model="qrCodeText"/>
            <div :class="{ 'btn-zone': isGenerateOk, 'btn-close': !isGenerateOk }">
              <button v-if="isGenerateOk" v-on:click="saveQRCode()">保存</button>
              <button v-on:click="onOpenCloseQRCodeCreateModal">閉じる</button>
            </div>
          </div>
        </div>
      </div>
    </transition>
    
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

  </div>
</template>

<style>
html {
  scroll-behavior: smooth;
}

.container {
  height: 100%;
}

.v-enter-active,
.v-leave-active {
  transition: all 0.3s ease-in-out;
}

.v-enter-from,
.v-leave-to {
  opacity: 0;
}

.parent-header {
  border-bottom: solid 1px gray;
  display: flex;
  justify-content: space-between;
  margin-bottom: 5px;
}

.app-header {
  color: #4183C4;
  text-shadow: 2px 1px 2px rgb(165, 165, 165);
  letter-spacing: 1px;
  font-size: 36px;
  margin-bottom: 0;
}

.app-header a {
  color: #4183C4;
}

.other-function-btn-zone {
  display: flex;
  z-index: 5;
  position: fixed;
  top: 1%;
  right: 1%;
}

a {
  text-decoration: none;
}

/* 目次のリンク（github.cssを上書き） */
.toc-content a {
  text-decoration: none;
  color: black;
}

/* ホバー */
.toc-content a:hover {
  background-color: #c1d1d6;
  display: inline-block;
  text-decoration: underline;
  /* transform を適用するために必要 */
}

.app-header a:hover {
  transform: scale(1.03);
  display: inline-block;
  text-decoration: underline;
  /* transform を適用するために必要 */
}

/**
* マークダウンプレビューのh1、h2タイトル用
*/
.head1 {
  font-size: 32px;
  margin-bottom: -1%;
  border-bottom: solid 3px #d7d7d7;
}

.head2 {
  font-size: 20px;
  color: black;
  margin-top: 2%;
  padding: 0.1em 0.3em;
  background: #f4f4f4;
  border-left: solid 5px #daac9e;
  border-bottom: solid 3px #d7d7d7;
}

/* メモモーダル */
#overlay-memo {
  z-index: 4;
  position: fixed;
  top: 0;
  left: 0;
  height: 100%;
  width: 100%;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
}

/* メモモーダルのコンテンツ */
#content-memo {
  z-index: 5;
  height: 100%;
  width: 80%;
  padding: 1em;
  margin-left: 70%;
  background: whitesmoke;
  overflow-y: auto;
}

.btn-memo-open-close {
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

.btn-memo-open-close:hover {
  background: rgb(235, 235, 235);
}

.btn-qrcode-creater-open-close {
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

.btn-qrcode-creater-open-close:hover {
  background: rgb(235, 235, 235);
}

.memo-title {
  text-align: center;
}

/* Aceエディタの上にモーダルを出した際の崩れ（スクロールバーが前面に現れる）を解消 */
.ace_editor {
  z-index: 0;
  height: 85%;
  isolation: isolate;
}

.editor-div {
  border-radius: 5px;
  border: solid 0.5px;
}

/* QRコード生成モーダル */
#overlay-gen-qrcode {
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
  text-align: center;
}

#content-gen-qrcode {
  z-index: 2;
  width: 30%;
  padding: 1em;
  background: whitesmoke;
  border-radius: 10px;
}

.qrcode {
  margin-bottom: 5%;
  display: grid;
  place-items: center;
}

.input-textbox {
  font-size: 24px;
  width: 90%;
  height: 40px;
  text-align: center;
  border-radius: 5px;
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

.setting-contents {
  text-align: center;
}

/* QR生成ボタンの間隔 */
.btn-zone {
  margin-top: 5%;
  display: flex;
  justify-content: space-between;
}

/* 最後のボタンの右側のマージンを0にする（オプショナル） */
.btn-zone button:last-child {
  margin-right: 0;
}

/**
* Update.vueのDiffモーダル用
*/
/* 削除されたテキスト */
.delete {
  display: inline-block;
  margin-top: 1px;
  text-decoration: none;
  background-color: #ffb6ba;
  border-radius: .2em;
}

/* 追加されたテキスト */
.added {
  display: inline-block;
  margin-top: -1px;
  text-decoration: none;
  background-color: #97f295;
  border-radius: .2em;
}

#splash-screen {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  background: linear-gradient(135deg, #cecece, #ffffff);
  color: white;
  text-align: center;
  z-index: 1000;
  z-index: 10;
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
}

#splash-title {
  color: #4183C4;
  font-size: 32px;
  text-shadow: 2px 1px 2px rgb(165, 165, 165);
  letter-spacing: 1px;
  letter-spacing: 1px;
  text-align: center;
  letter-spacing: 2px;
  margin: 0;
  padding: 20px 10px;
  animation: fade-in 1.5s ease-in-out;
}

.logo {
  width: 100px;
  height: auto;
  animation: fade-in 1.5s ease-in-out;
  z-index: 1000;
  border: none;
  box-shadow: none;
}

.message {
  margin-top: 20px;
  font-size: 18px;
  animation: fade-in 2s ease-in-out;
}

@keyframes fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

/* メッセージモーダル */
#overlay-message {
  z-index: 19;
  position:fixed;
  top:0;
  left:0;
  width:100%;
  height:100%;
  background-color:rgba(0,0,0,0.5);
  display: flex;;
  align-items: center;
  justify-content: center;
}

#content-message {
  z-index:20;
  width:20%;
  padding: 1em;
  background:whitesmoke;
  border-radius: 10px;
}
</style>