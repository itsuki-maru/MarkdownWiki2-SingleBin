<script setup lang="ts">
import { marked, Renderer } from "marked";
import type { Tokens, MarkedOptions } from "marked";
import { computed, ref, onMounted, onUnmounted, nextTick, watch, onBeforeUnmount } from "vue";
import { useRouter, useRoute } from "vue-router";
import type { WikiData } from "@/interface";
import { assetsUrl } from "@/setting";
import { useWikiStore } from "@/stores/wikis";
import { downloadFileUrl, wikiOwnerGetUrl, generateOnetimeWikiUrl, invalidateOntimeWikiUrl } from "@/router/urls";
import '@/assets/github.css';
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
import Prism from "prismjs";
import "prismjs/themes/prism-okaidia.css";
import "prismjs/components/prism-typescript";
import "prismjs/components/prism-javascript";
import "prismjs/components/prism-bash";
import "prismjs/components/prism-python";
import "prismjs/components/prism-rust";
import "prismjs/components/prism-markup";
import "prismjs/components/prism-json";
import "prismjs/components/prism-markdown.js";
import "prismjs/components/prism-powershell.js";
import "prismjs/components/prism-sql.js";
import "prismjs/components/prism-toml.js";
import "prismjs/components/prism-yaml.js";
import "prismjs/components/prism-uri.js";
import "prismjs/components/prism-c.js";
import "prismjs/components/prism-docker.js";
import "katex/dist/katex.min.css";
import FindBar from '@/components/FindBar.vue';

// アプリケーションの通信プロトコル
const isHttpsProtocol = ref(false);
const isDevelopLocalhost = ref(false);
// 現在のURLを取得
const currentUrl = window.location.href;
// URLを解析
const url = new URL(currentUrl);
// プロトコルとホスト名を取得
const protocol = url.protocol;
const hostname = url.hostname;
const port = url.port;
// HTTPSかlocalhost通信の場合の設定
if (protocol === "https:") {
  isHttpsProtocol.value = true;
} if (hostname === "localhost") {
  isHttpsProtocol.value = true;
  // 開発環境の処理
  if (port === "4080") {
    isDevelopLocalhost.value = true;
  }
}

const mermaid: any = (window as any).mermaid;

// Mermaidの初期読み込みを阻止（MarkedによるHTMLレンダリング後にinitで読み込み）
mermaid.initialize({startOnLoad: false});

// markedのスラッグ化機能をカスタマイズ
const renderer = new Renderer();
let headingIndex = -1; // 見出しのインデックス（h1タグは無視したいため-1から開始）
renderer.heading = function (tokens: Tokens.Heading) {
  const id = `heading-${headingIndex++}`; // インデックスに基づいてIDを生成
  return `<h${tokens.depth} id="${id}" class="head${tokens.depth}">${tokens.text}</h${tokens.depth}>\n`; // class属性のCSSはトップレベル（App.vue）で定義
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

// codeタグの処理
const originalCodeRenderer = renderer.code.bind(renderer);
renderer.code = (tokens: Tokens.Code) => {
  let html = originalCodeRenderer(tokens);
  // mermaidの処理
  if (tokens.lang == "mermaid") {
    return '<pre class="mermaid">' + escapeHtml(tokens.text) + '\n</pre>';

  // 通常のコードブロック + コピー機能
  } else {
    const id = `code-${Math.random().toString(36).substr(2, 9)}`;
    const lang = tokens.lang;
    const escapedCode = escapeHtml(tokens.text);
    return `
    <div class="code-container" style="position: relative;">
      <button class="copy-btn" data-target="${id}" style="position: absolute; top: 5px; right: 5px; z-index: 1;">コピー</button>
      <pre><code id="${id}" class="language-${lang}">${escapedCode}</code></pre>
    </div>
    `
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

// codeタグにコピー機能を実装
onMounted(() => {
  document.addEventListener("click", (e) => {
    const target = e.target as HTMLElement;
    if (target.classList.contains("copy-btn")) {
      const codeId = target.dataset.target;
      const codeElem = document.getElementById(codeId || "")
      if (codeElem && isHttpsProtocol.value) {
        navigator.clipboard.writeText(codeElem.textContent || "");

        // すでにメッセージがあれば削除
        const existingTooltip = target.parentElement?.querySelector(".copy-tooltip");
        if (existingTooltip) existingTooltip.remove();

        // メッセージを作成
        const tooltip = document.createElement("div");
        tooltip.textContent = "コピーしました";
        tooltip.className = "copy-tooltip";

        // ボタンの親要素（code-container）に追加
        target.parentElement?.appendChild(tooltip);

        // 一定時間後に非表示
        setTimeout(() => {
          tooltip.style.opacity = "0";
          setTimeout(() => tooltip.remove(), 300);
        }, 1000);
      }
    }
  })
});

// Markedにカスタムトークンを追加
marked.use(
  {
    extensions: [
      videoToken,
      detailsToken,
      noteToken,
      warningToken,
      mathExtentionToken,
      youtubeToken
    ],
  }
);

// HTMLエスケープ関数
function escapeHtml(html: string) {
  return html
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

// markedの設定をカスタマイズ
marked.setOptions({
  renderer,
  async: false
});

// XSSフィルタの設定をカスタマイズする
let xssOptions: IFilterXSSOptions = {
  whiteList: {
    ...getDefaultWhiteList(), // デフォルトの許可リストを維持
    h1: ['id', 'class'], // h1タグのid属性を許可 class属性を許可
    h2: ['id', 'class'], // h2タグのid属性を許可 class属性を許可
    h3: ['id'], // h3タグのid属性を許可
    h4: ['id'], // h4タグのid属性を許可
    h5: ['id'], // h5タグのid属性を許可
    h6: ['id'], // h6タグのid属性を許可
    pre: ['class'],
    a: ['target', 'rel', 'href', 'title'],
    button: ['class', 'data-target'],
    code: ['id', 'class'],
    div: ['class'],
    p: ['class'],
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

// updateへのリダイレクト設定
const router = useRouter();
const updateViewRedirect = (id: string): void => {
  router.push(`/wiki/update/${id}`);
}

// Login.vueへのリダイレクト
const loginRedirect = (): void => {
  router.push("/account/login");
}

// List.vueへリダイレクト
const listRedirect = (): void => {
  router.push("/wiki/list");
}

// Delete.vueへのリダイレクト
const deleteRedirect = (id: string): void => {
  router.push(`/wiki/delete/${id}`);
}

// 画面遷移時に確実にハイライトを実行
const route = useRoute();
const highlight = async () => {
  await nextTick();
  Prism.highlightAll();
}

onMounted(highlight);
watch(() => route.fullPath, highlight);

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

// マークダウンへのパース処理
const textTitleData = "# " + wiki.value.title + "\n\n";
const textBodyData = wiki.value.body;
const markdownData = textTitleData + textBodyData;
const options: MarkedOptions = { async: false };
const htmlStr = marked.parse(markdownData, options);
const cleanHtml = myXss.process(htmlStr as string);
const renderHtml = renderIframe(cleanHtml);
const bindHtml = ref(renderHtml);
Prism.highlightAll();

// Wikiデータのオーナー取得
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
    console.error("Owner Get Error");
    loginRedirect();
  }
};
getWikiOwner(props.id);


/** Wikiデータの更新処理 */
const updateWikiData = (id: string): void => {
  getWikiOwner(id);
  if (localStorage.getItem("loginUser") !== wikiOwner.value) {
    messageModalOpenClose("オーナーではないため、編集の権限がありません。");
    return;
  } else {
    updateViewRedirect(id);
  }
};

/** ダウンロード処理 */
const downloadFile = async (id: string): Promise<void> => {
  try {
    const response = await apiClient.get(
      downloadFileUrl + `/${id}`,
    );
    // FILE DOWNLOAD
    let blob = new Blob([response.data], { "type": "text/markdown" });
    const downloadUrl = window.URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = downloadUrl;
    const fileName = `${id}.md`;
    link.setAttribute("download", fileName);
    document.body.appendChild(link);
    link.click();
    link.remove();
  } catch (error) {
    console.error(error);
  }
};

// プレビュー描画の印刷モード・通常モードの切り替え管理
const isPrintMode = ref(true);
const changePrintMode = (): void => {
  if (isPrintMode.value === false) {
    isPrintMode.value = true;
  } else {
    isPrintMode.value = false;
  }
};

// 目次の表示非表示の切り替えを管理
const showTocContent = ref(true);
const changeShowToc = (): void => {
  if (showTocContent.value === false) {
    showTocContent.value = true;
  } else {
    showTocContent.value = false;
  }
};

// 見出しのテキストをIDに変換
const createId = (index: number): string => {
  return `heading-${index}`;
};

// 1行の中に三重引用符が出現した回数を返す
const countBackticksInLine = (line: string): number => {
  const matches = line.match(/```/g);
  return matches ? matches.length : 0;
};

// マークダウンデータから見出しの行を抽出する
const getToc = (): string[] => {
  // コードブロック内かどうかを示すフラグ
  let inCodeBlock = false;
  // 入力文字列を行に分解
  const lines = textBodyData.split('\n');
  // 各行をループして、`#`で始まる行で、かつバッククォートの数が2個以下の行だけをフィルタリング
  const headings = lines.filter(line => {
    // 三重引用符が1行内に2つ以上あった場合
    const lenBackQoute = countBackticksInLine(line);
    if (lenBackQoute >= 2) {
      return false;
    }
    if (line.trim().startsWith('```')) {
      // コードブロックの開始または終了を切り替える
      inCodeBlock = !inCodeBlock;
      return false;
    }
    if (inCodeBlock) {
      // コードブロック内の行は無視する
      return false;
    }
    const match = line.match(/^#+/);
    return match;
  });
  return headings;
};

// 見出しの配列からマークダウンのリストを作成
const createNestedList = (headings: string[]): string => {
  // 見出しの配列に要素が一つもない場合
  if (headings.length === 0) {
    return "No Table Of Contents.";
  }

  // 最終的なマークダウンテキスト
  let result = "";

  // 見出しの最初のレベルが2であることを確認するフラグ
  let isFirstTocLebelOk = false;

  // 見出しマークダウンの作成処理
  headings.forEach((heading, index) => {
    // '#'の数を取得
    const level = heading.split(" ").shift()!.length;

    // レベル1なら見出しから除外
    if (level === 1) {
      return;
    }

    // 最初の見出しを2から始めるための処理
    if (!isFirstTocLebelOk) {
      if (level === 2) {
        isFirstTocLebelOk = true;
      } else {
        return;
      }
    }

    // '#'を除去
    let text = heading.replace(/#/g, "").trim();
    // '`'を除去
    text = text.replace(/`/g, '').trim();
    // インデントを付ける
    const indents = "  ".repeat(level - 1);

    const id = createId(index);
    // 目次のエントリにリンクを追加
    result += `${indents}- [${text}](#${id})\n`;
  });
  return result;
};

// 目次（マークダウン）からHTMLへ変換
const tocToHtml = computed(
  (): string => {
    const tocListString = getToc();
    const mdTocStr = createNestedList(tocListString);
    const tocHtml = marked.parse(mdTocStr);
    return tocHtml as string;
  }
);

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

// ワンタイム設定モーダル
const isOnetimeSettingModal = ref(false);
const openCloseOnetimeSetting = (): void => {
  if (!isOnetimeSettingModal.value) {
    isOnetimeSettingModal.value = true;
  } else {
    isOnetimeSettingModal.value = false;
  }
};

// ワンタイムURLの表示モーダル
const isOnetimeUrlModal = ref(false);
const openCloseOnetimeUrl = (): void => {
  if (!isOnetimeUrlModal.value) {
    isOnetimeUrlModal.value = true;
  } else {
    isOnetimeUrlModal.value = false;
  }
};

// 与えられたelement idのテキストに次の処理
// HTTPS（localhost）プロトコル下ではクリップボードコピー HTTPではテキスト選択（IEは非対応）
function selectTextOrClipboardCopy(elementId: string) {
  let element = document.getElementById(elementId);
  if (!element || !element.textContent) {
    console.log(`Element with ID ${elementId} not found`);
    return;
  }

  if (isHttpsProtocol.value) {
    navigator.clipboard.writeText(element.textContent);
    messageModalOpenClose("クリップボードにコピーしました。");
  } else {
    if (window.getSelection) {
      let selection = window.getSelection();
      let range = document.createRange();
    try {
      range.selectNodeContents(element);
    } catch (e) {
      console.log(`Error selecting contents of element: ${e}`);
    }
    if (selection) {
      selection.removeAllRanges();  // 現在の選択をクリア
      selection.addRange(range);  // 新しい範囲を選択
    }
  }
}
}

const oneTimeUrl = ref("");
const oneTimeUuid = ref("");
const onetimeDurationMinits = ref(60);
const genOnetimeWikiUrl = async (): Promise<void> => {
  try {
    if (Number.isInteger(onetimeDurationMinits.value) === false) {
      messageModalOpenClose("数値を入力してください。");
      return;
    }
    if (onetimeDurationMinits.value < 10) {
      messageModalOpenClose("10分以上の設定が必要です。");
      return;
    }
    const payload = {
      minutes: onetimeDurationMinits.value,
    }
    const url = generateOnetimeWikiUrl + `${props.id}`;
    const response = await apiClient.post(
      url,
      payload,
    );

    if (isDevelopLocalhost.value) {
      oneTimeUrl.value = `${protocol}//${hostname}:4080${response.data["url"]}`;
    } else {
      if (port === "") {
        // 本番環境（HTTPS + ドメイン時）
        oneTimeUrl.value = `${protocol}//${hostname}${port}${response.data["url"]}`;
      } else {
        // 開発環境（HTTP or LOCALHOST時）
        oneTimeUrl.value = `${protocol}//${hostname}:${port}${response.data["url"]}`;
      }
    }
    
    oneTimeUuid.value = response.data["id"];
    openCloseOnetimeUrl();
  } catch (error) {
    console.error("Error");
  }
}

const invalidateOneTimeWiki = async (): Promise<void> => {
  try {
    const url = invalidateOntimeWikiUrl + `${oneTimeUuid.value}`;
    const response = await apiClient.delete(
      url,
    );
    openCloseOnetimeUrl();
    messageModalOpenClose("共有を停止しました。");
  } catch (error) {
    console.error("Error");
  }
}

// 画面上部（id=application-title）へスクロール
const scrollAppTitle = (): void => {
  const element = document.getElementById("application-title");
  if (element) {
    element.scrollIntoView({ behavior: "smooth" });
  }
}

// id="heading--1"までスクロールされたらページ上部までスクロールさせるボタンを出現
const showScrollBtn = ref(false);
document.addEventListener("scroll", function () {
  let scrollPosition = window.scrollY;
  let targetElement = document.getElementById("heading--1");
  if (targetElement) {
    // 特定の位置に達したかどうかをチェック
    if (scrollPosition >= targetElement.offsetTop) {
      showScrollBtn.value = true;
    } else {
      showScrollBtn.value = false;
    }
  }
});

// 削除画面であることの注意喚起モーダル
const isDeleteModal = ref(false);
const onOpenDeleteViewModal = (): void => {
  if (isDeleteModal.value === true) {
    isDeleteModal.value = false;
  } else {
    isDeleteModal.value = true;
  }
}

// 削除ビュー遷移確認モーダル表示時に灰色の部分のクリック時にも削除ビュー遷移確認モーダルを閉じる処理
// HTMLが描画後に組み込む（onmoutedを利用）
onMounted(() => {
  // オーバレイとヘルプの内容を取得
  const deleteViewCheckModal = document.getElementById("overlay-delete-check");
  const deleteViewCheckModalContent = document.getElementById("content-delete-check");

  // 灰色部分クリック時にクローズ処理がなされるようにイベント設定
  if (deleteViewCheckModal) {
    deleteViewCheckModal.addEventListener("click", function (event) {
      if (isDeleteModal.value === true) {
        isDeleteModal.value = false
      } else {
        return;
      }
    });
  }

  // 灰色の部分以外（content-delete-check）をクリックした時にはイベント伝搬を止め、クローズさせない
  if (deleteViewCheckModalContent) {
    deleteViewCheckModalContent.addEventListener("click", function (event) {
      event.stopPropagation();
    });
  }
});

// 灰色の部分をクリックし共有モーダルの非表示処理
onMounted(() => {
  const onetimeSettingModal = document.getElementById("overlay-onetime-setting");
  const onetimeSettingModalContent = document.getElementById("content-onetime-setting");

  if (onetimeSettingModal) {
    onetimeSettingModal.addEventListener("click", function (event) {
      if (isOnetimeSettingModal.value === true) {
        openCloseOnetimeSetting();
      } else {
        return;
      }
    });
  }

  // 灰色の部分以外（content-delete-check）をクリックした時にはイベント伝搬を止め、クローズさせない
  if (onetimeSettingModalContent) {
    onetimeSettingModalContent.addEventListener("click", function (event) {
      event.stopPropagation();
    });
  }
});

// ショートカットキーを追加
const handleKeyDown = (event: KeyboardEvent) => {
  // List.vueへ移動
  if (event.ctrlKey && event.key === "1") {
    event.preventDefault(); // デフォルトのブラウザのショートカットをキャンセル
    listRedirect();
    // Update.vueへ移動
  } else if (event.ctrlKey && event.key === "2") {
    event.preventDefault();
    updateViewRedirect(wiki.value.id);
    // ファイルダウンロード
  } else if (event.ctrlKey && event.key === "3") {
    event.preventDefault();
    downloadFile(wiki.value.id);
    // 表示切替
  } else if (event.ctrlKey && event.key === "4") {
    event.preventDefault();
    changePrintMode();
    // 目次の表示切替
  } else if (event.ctrlKey && event.key === "5") {
    event.preventDefault();
    changeShowToc();
  
  } else if (event.ctrlKey && event.key === "6") {
    event.preventDefault();
    openCloseOnetimeSetting();

  } else if (event.ctrlKey && event.key === "7") {
    event.preventDefault();
    openCloseSearchBar(); 
  
    // 上部へスクロール
  } else if (event.altKey && event.key === "u") {
    event.preventDefault();
    scrollAppTitle();
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

// ウィンドウサイズで目次の表示非表示を変更
function useWindowSize() {
  const width = ref(window.innerWidth);
  const height = ref(window.innerHeight);

  const updateSize = () => {
    width.value = window.innerWidth;
    height.value = window.innerHeight;
  };
  onMounted(() => {
    window.addEventListener("resize", updateSize);
  });

  onBeforeUnmount(() => {
    window.removeEventListener("resize", updateSize);
  });
  return { width, height };
}

const { width } = useWindowSize();
if (width.value < 770 ) {
  showTocContent.value = false;
}

watch(width, (newWidth) => {
  if (newWidth > 770) {
    showTocContent.value = true;
  } else {
    showTocContent.value = false;
  }
});

// 検索機能用
const contentEl = ref<HTMLElement | null>(null);

// 検索バー
const isShowSearchBar = ref(false);
const openCloseSearchBar = (): void => {
  if (!isShowSearchBar.value) {
    isShowSearchBar.value = true;
  } else {
    isShowSearchBar.value = false;
  }
};
</script>

<template>
  <div class="btn-head-left">
    <button class="btn-head-image" title="Wiki一覧画面へ遷移します。&#10;ショートカット: Ctrl + 1" v-on:click="listRedirect()"><img :src="`${assetsUrl}home_24.png`" class="btn-img" alt="home_24.png"></button>
    <button class="btn-head-image" title="更新画面へ遷移します。&#10;ショートカット: Ctrl + 2" v-if="isOwner" v-on:click="updateWikiData(wiki.id)"><img :src="`${assetsUrl}edit_24.png`" class="btn-img" alt="edit_24.png"></button>
    <button class="btn-head-image" title="マークダウンファイルをダウンロード&#10;ショートカット: Ctrl + 3" v-on:click="downloadFile(wiki.id)"><img :src="`${assetsUrl}download_fill24.png`" class="btn-img" alt="download_fill24.png"></button>
    <button class="btn-head-image" title="スクロール表示の切替&#10;ショートカット: Ctrl + 4" v-if="isPrintMode" v-on:click="changePrintMode()"><img :src="`${assetsUrl}close_fullscreen_24.png`" class="btn-img" alt="close_fullscreen_24.png"></button>
    <button class="btn-head-image" title="スクロール表示の切替&#10;ショートカット: Ctrl + 4" v-else="isPrintMode" v-on:click="changePrintMode()"><img :src="`${assetsUrl}fullscreen_24.png`" class="btn-img" alt="fullscreen_24.png"></button>
    <button class="btn-head-image" title="目次の表示・非表示&#10;ショートカット: Ctrl + 5" v-on:click="changeShowToc()"><img :src="`${assetsUrl}toc_24.png`" class="btn-img" alt="toc_24.png"></button>
    <button class="btn-head-image" title="Wikiの共有URLを作成&#10;ショートカット: Ctrl + 6" v-if="isOwner" v-on:click="openCloseOnetimeSetting()"><img :src="`${assetsUrl}family_line24.png`" class="btn-img" alt="family_line24.png"></button>
    <button class="btn-head-image" title="ページ内検索&#10;ショートカット: Ctrl + 7" v-on:click="openCloseSearchBar()"><img :src="`${assetsUrl}search_fill24.png`" class="btn-img" alt="search_fill24.png"></button>
  </div>
  <transition>
    <div class="btn-scrolled-show" v-show="showScrollBtn">
      <button class="btn-scroll-to-list scroll-btn-hover" title="Wiki一覧" v-on:click="listRedirect()"><img
          :src="`${assetsUrl}home_24.png`" class="btn-img" alt="home_24.png"></button>
      <button class="btn-scroll-to-top scroll-btn-hover" title="画面上部へ移動&#10;ショートカット: Alt + U" v-on:click="scrollAppTitle()"><img
          :src="`${assetsUrl}arrow_upward_24.png`" class="btn-img" alt="arrow_upward_24.png"></button>
      <button class="btn-scroll-to-update scroll-btn-hover" title="更新" v-if="isOwner" v-on:click="updateWikiData(wiki.id)"><img
          :src="`${assetsUrl}edit_24.png`" class="btn-img" alt="edit_24.png"></button>
      <button class="btn-scroll-to-update scroll-btn-hover" title="ページ内検索&#10;ショートカット: Ctrl + 7" v-on:click="openCloseSearchBar()"><img 
          :src="`${assetsUrl}search_fill24.png`" class="btn-img" alt="search_fill24.png"></button>
    </div>
  </transition>
  <div class="contants-area">
    <div :class="{ 'istoc': showTocContent, 'notoc': !showTocContent }">
      <div :class="{ 'markdown-isprint': isPrintMode, 'markdown-isnomal': !isPrintMode }">
        <section v-html="bindHtml" ref="contentEl"></section>
      </div>
      <div class="footer-zone" v-if="isOwner">
        <button class="btn-delete" v-on:click="onOpenDeleteViewModal()">削除</button>
        <p class="wiki-owner">Wikiオーナー: {{ wikiOwner }}</p>
      </div>
    </div>
    <div v-if="showTocContent" class="toc">
      <h3 class="toc-title">目次</h3>
      <div class="toc-content" v-html="tocToHtml"></div>
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

  <!-- ワンタイムWikiの有効化設定モーダル -->
  <div id="overlay-onetime-setting" v-show="isOnetimeSettingModal">
    <div id="content-onetime-setting">
      <h2 class="modal-h2">Wikiの共有リンクを作成</h2>
      <div class="input-area-duration">
        <label for="minits" style="font-size: 16px;">有効期限（分）</label>
        <input v-model="onetimeDurationMinits" type="number" step="10" class="input-minits" id="minits">
      </div>
      <div class="btn-zone">
        <button v-on:click="openCloseOnetimeSetting()">閉じる</button>
        <button v-on:click="genOnetimeWikiUrl()">共有リンクを作成</button>
      </div>
    </div>
  </div>

  <!-- ワンタイムWikiのURL表示モーダル -->
  <div id="overlay-onetime-message" v-show="isOnetimeUrlModal">
    <div id="content-onetime-message">
      <h2 class="modal-h2">メッセージ</h2>
      <div class="input-text-zone" v-if="isHttpsProtocol">
        <p><strong>共有リンクを作成しました。</strong></p>
        <pre :id=oneTimeUuid class="hidden-code-text"><code :id=oneTimeUuid>{{ oneTimeUrl }}</code></pre>
        <button id="link-copy-btn" v-on:click="selectTextOrClipboardCopy(`${oneTimeUuid}`)">リンクを取得</button>
      </div>
      <div class="input-text-zone" v-else="isHttpsProtocol">
        <p><strong>共有リンクを作成しました。</strong></p>
        <pre><code :id=oneTimeUuid v-on:click="selectTextOrClipboardCopy(`${oneTimeUuid}`)">{{ oneTimeUrl }}</code></pre>
      </div>
      <div class="btn-zone">
        <button v-on:click="openCloseOnetimeUrl()">閉じる</button>
        <button v-on:click="invalidateOneTimeWiki()">共有停止</button>
      </div>
    </div>
  </div>

  <!-- 削除ビューへの遷移確認モーダル -->
  <transition>
    <div id="overlay-delete-check" v-show="isDeleteModal">
      <div id="content-delete-check">
        <h2 class="modal-h2">警告</h2>
        <p><strong>削除画面へ移動します。削除しない場合は、「戻る」ボタンを選択してください。</strong></p>
        <div class="btn-zone">
          <button v-on:click="onOpenDeleteViewModal()">戻る</button>
          <button v-on:click="deleteRedirect(wiki.id)" class="btn-delete">移動</button>
        </div>
      </div>
    </div>
  </transition>

  <transition>
    <div id="search-word-box" v-show="isShowSearchBar">
      <FindBar :container="contentEl" :showOpenInBrowser="true" />
    </div>
  </transition>

  <footer>
    <p class="wiki-owner" v-if="!isOwner">Wikiオーナー: {{ wikiOwner }}</p>
  </footer>
</template>

<style scoped>
.contants-area {
  display: flex;
  height: 100%;
  margin-top: 10px;
}

.notoc {
  width: 100%;
}

.istoc {
  width: 70%;
}

.toc-title {
  color: black;
  text-align: center;
  font-size: 20px;
  margin-top: -1%;
}

.toc {
  width: 25%;
  height: 70%;
  margin-left: 70%;
  position: fixed;
  overflow: auto;
  color: whitesmoke;
  font-size: 14px;
}

.markdown-isnomal {
  background-color: white;
  color: black;
  overflow-y: auto;
  scroll-behavior: smooth;
  height: 70vh;
  border: 1px solid;
  border-color: gray;
  padding-top: 10px;
  padding-left: 20px;
  padding-right: 20px;
  border-collapse: collapse;
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

/* メッセージモーダル */
#overlay-message {
  z-index: 9;
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
  z-index: 10;
  width: 20%;
  padding: 1em;
  background: whitesmoke;
  border-radius: 10px;
  text-align: center;
}

#overlay-onetime-setting {
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

#content-onetime-setting {
  z-index: 4;
  width: 30%;
  padding: 1em;
  background: whitesmoke;
  border-radius: 10px;
}

.input-area-duration {
  margin: 20px;
  display: flex;
  justify-content: center;
}

.input-minits {
  font-size: 18px;
  text-align: center;
}

/* Wikiの共有リンク作成完了モーダル */
#overlay-onetime-message {
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

#content-onetime-message {
  z-index: 4;
  width: 35%;
  padding: 1em;
  background: whitesmoke;
  border-radius: 10px;
  text-align: center;
}

.btn-scrolled-show {
  z-index: 3;
  position: fixed;
  top: 35%;
  right: 30.3%;
  display: flex;
  flex-direction: column;
}

.btn-scroll-to-top,
.btn-scroll-to-list,
.btn-scroll-to-update {
  width: 40px;
  box-shadow: 3px 3px 5px 0 rgba(75, 75, 75, 0.5);
  background: white;
  color: #fff;
  padding: 5px 5px;
  text-decoration: none;
  border: 1px solid rgb(207, 207, 207);
  border-radius: 20px;
  transition-property: opacity;
  -webkit-transition-property: opacity;
  transition-duration: .5s;
  -webkit-transition-duration: .5s;
  margin: 20px 5px 5px 5px;
}

.scroll-btn-hover:hover {
  background: rgb(235, 235, 235);
  transition: background-color 0.3s;
}

#overlay-delete-check {
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

#content-delete-check {
  z-index: 4;
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

.footer-zone {
  display: flex;
  justify-content: space-between;
  margin-top: 1%;
}

.btn-delete {
  background: rgb(219, 54, 76);
}

#link-copy-btn {
  width: 250px;
  background: rgb(41, 15, 187);
  height: 45px;
  font-size: 16px;
  color: #ffffff;
  padding: 10px 7px;
  text-decoration: none;
  border: 1px;
  border-radius: 12px;
  transition-property: opacity;
  -webkit-transition-property: opacity;
  transition-duration: .5s;
  -webkit-transition-duration: .5s;
  margin: 0px 5px 10px 5px;
}

#link-copy-btn:hover {
  background: rgb(22, 7, 109);
  transition: background-color 0.3s;
}

.input-text-zone {
  text-align: center;
}

.hidden-code-text {
  display: none;
}

.wiki-owner {
  position: fixed;
  bottom: 1px;
  right: 1%;
  text-align: right;
  font-size: 14px;
  font-weight: bold;
  text-shadow: 1px 1px 2px rgb(202, 202, 202);
}

#search-word-box {
  z-index: 3;
  position: fixed;
  bottom: 15%;
  left: 50%;
}
</style>
