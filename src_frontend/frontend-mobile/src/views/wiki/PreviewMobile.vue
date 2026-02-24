<script setup lang="ts">
import { marked, Renderer } from 'marked';
import type { Tokens } from 'marked';
import { computed, ref, onMounted, onUnmounted, nextTick, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import type { WikiData, TypeWikiOwner } from '@/interface';
import { useWikiStore } from '@/stores/wikis';
import {
  wikiOwnerGetUrl,
  generateOnetimeWikiUrl,
  invalidateOntimeWikiUrl,
  patchWikiBodyUrl,
} from '@/router/urls';
import { assetsUrl } from '@/setting';
import '@/assets/github.css';
import {
  videoToken,
  detailsToken,
  noteToken,
  warningToken,
  mathExtentionToken,
  renderIframe,
  youtubeToken,
  escapeHtml,
  isPDF,
  createLinkRenderer,
  createImageRenderer,
  createXssFilter,
} from '@/utils/markedSetup';
import { useMessageModal } from '@/utils/useMessageModal';
import { useProtocolDetection } from '@/utils/useProtocolDetection';
import apiClient from '@/axiosClient';
import Prism from 'prismjs';
import 'prismjs/themes/prism-okaidia.css';
import 'prismjs/components/prism-typescript';
import 'prismjs/components/prism-javascript';
import 'prismjs/components/prism-bash';
import 'prismjs/components/prism-python';
import 'prismjs/components/prism-rust';
import 'prismjs/components/prism-markup';
import 'prismjs/components/prism-json';
import 'prismjs/components/prism-markdown.js';
import 'prismjs/components/prism-powershell.js';
import 'prismjs/components/prism-sql.js';
import 'prismjs/components/prism-toml.js';
import 'prismjs/components/prism-yaml.js';
import 'prismjs/components/prism-uri.js';
import 'prismjs/components/prism-c.js';
import 'prismjs/components/prism-docker.js';
import 'katex/dist/katex.min.css';
import FindBar from '@/components/FindBar.vue';

// アプリケーションの通信プロトコル
const { isHttpsProtocol, isDevelopLocalhost } = useProtocolDetection();
// URL構築用にローカルで保持
const { protocol, hostname, port } = new URL(window.location.href);

// メッセージ表示モーダル機能
const { isMessageModal, messageText, messageModalOpenClose } = useMessageModal();

const mermaid: any = (window as any).mermaid;

// Mermaidの初期読み込みを阻止（MarkedによるHTMLレンダリング後にinitで読み込み）
mermaid.initialize({ startOnLoad: false });

// タスクチェックボックスのMarkdown内オフセット管理
interface TaskOffset {
  start: number; // '[' の位置
  end: number; // ']' の次（3文字分）
}
let taskOffsets: TaskOffset[] = [];
let taskOffsetIdx = 0;

// markedのスラッグ化機能をカスタマイズ
const renderer = new Renderer();
let headingIndex = -1; // 見出しのインデックス（h1タグは無視したいため-1から開始）
renderer.heading = function (tokens: Tokens.Heading) {
  const id = `heading-${headingIndex++}`; // インデックスに基づいてIDを生成
  return `<h${tokens.depth} id="${id}" class="head${tokens.depth}">${tokens.text}</h${tokens.depth}>\n`; // class属性のCSSはトップレベル（App.vue）で定義
};

// [テキスト](URL)で定義された外部リンクを別タブで開かせるカスタムレンダラ設定
createLinkRenderer(renderer);

// tableにclassを付する処理（div要素内にテーブルを含包させる）
const originalTableRenderer = renderer.table.bind(renderer);
renderer.table = (tokens: Tokens.Table) => {
  let html = originalTableRenderer(tokens);
  let customHtml = html.replace(
    /^<table>/,
    '<div class="scrollable-table"><table class="md-mobile-table">',
  );
  customHtml = customHtml.replace(/<\/table>/g, '</table></div>');
  return customHtml;
};

// codeタグの処理
const originalCodeRenderer = renderer.code.bind(renderer);
renderer.code = (tokens: Tokens.Code) => {
  let html = originalCodeRenderer(tokens);
  // mermaidの処理
  if (tokens.lang == 'mermaid') {
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
    `;
  }
};

createImageRenderer(renderer);

// タスクチェックボックスのレンダラをオーバーライド
// data-start / data-end を付与し、disabled を外してインタラクティブにする
renderer.checkbox = ({ checked }: Tokens.Checkbox) => {
  const offset = taskOffsets[taskOffsetIdx];
  if (offset) {
    taskOffsetIdx++;
    const checkedAttr = checked ? ' checked' : '';
    return `<input type="checkbox"${checkedAttr} data-start="${offset.start}" data-end="${offset.end}"> `;
  }
  // オフセットが取得できなかった場合はデフォルトと同等（disabled）にフォールバック
  return `<input type="checkbox"${checked ? ' checked' : ''} disabled> `;
};

// codeタグにコピー機能を実装
onMounted(() => {
  document.addEventListener('click', (e) => {
    const target = e.target as HTMLElement;
    if (target.classList.contains('copy-btn')) {
      const codeId = target.dataset.target;
      const codeElem = document.getElementById(codeId || '');
      if (codeElem && isHttpsProtocol.value) {
        navigator.clipboard.writeText(codeElem.textContent || '');

        // すでにメッセージがあれば削除
        const existingTooltip = target.parentElement?.querySelector('.copy-tooltip');
        if (existingTooltip) existingTooltip.remove();

        // メッセージを作成
        const tooltip = document.createElement('div');
        tooltip.textContent = 'コピーしました';
        tooltip.className = 'copy-tooltip';

        // ボタンの親要素（code-container）に追加
        target.parentElement?.appendChild(tooltip);

        // 一定時間後に非表示
        setTimeout(() => {
          tooltip.style.opacity = '0';
          setTimeout(() => tooltip.remove(), 300);
        }, 1000);
      }
    }
  });
});

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

/**
 * Markdown文字列からタスクチェックボックスの位置（start, end）を順番に収集する。
 * コードブロック（```）内のチェックボックス記法は除外する。
 * start: '[' の絶対オフセット、end: start + 3（'[ ]' または '[x]' の末尾）
 */
function buildTaskOffsets(markdown: string): TaskOffset[] {
  const offsets: TaskOffset[] = [];

  // フェンスコードブロックの範囲を収集して除外対象にする
  const codeRanges: Array<[number, number]> = [];
  const codeRegex = /^```[\s\S]*?^```[ \t]*$/gm;
  let cm: RegExpExecArray | null;
  while ((cm = codeRegex.exec(markdown)) !== null) {
    codeRanges.push([cm.index, cm.index + cm[0].length]);
  }

  // タスクリスト行を検出: 行頭の任意インデント + リストマーカー + [ ] or [x]
  const taskRegex = /^[ \t]*[-*+] \[([ x])\] /gm;
  let tm: RegExpExecArray | null;
  while ((tm = taskRegex.exec(markdown)) !== null) {
    const openBracket = tm.index + tm[0].indexOf('[');
    const inCode = codeRanges.some(([s, e]) => openBracket >= s && openBracket < e);
    if (!inCode) {
      offsets.push({ start: openBracket, end: openBracket + 3 });
    }
  }

  return offsets;
}

/**
 * Markdownをパースして表示用HTMLを生成する。
 * 毎回 headingIndex / taskOffsets / taskOffsetIdx をリセットして一貫性を保つ。
 */
function renderMarkdown(markdown: string): string {
  headingIndex = -1;
  taskOffsets = buildTaskOffsets(markdown);
  taskOffsetIdx = 0;
  const parsed = marked.parse(markdown, { async: false }) as string;
  const clean = myXss.process(parsed);
  return renderIframe(clean);
}

// Login.vueへのリダイレクト
const router = useRouter();
const loginRedirect = (): void => {
  router.push('/account/login');
};

// List.vueへリダイレクト
const listRedirect = (): void => {
  router.push('/wiki/list');
};

// Update.vueへのリダイレクト設定
const updateViewRedirect = (id: string): void => {
  router.push(`/wiki/update/${id}`);
};

// Delete.vueへのリダイレクト
const deleteRedirect = (id: string): void => {
  router.push(`/wiki/delete/${id}`);
};

// 画面遷移時に確実にハイライトを実行
const route = useRoute();
const highlight = async () => {
  await nextTick();
  Prism.highlightAll();
};

onMounted(highlight);
watch(() => route.fullPath, highlight);

interface Props {
  id: string;
}

interface Props {
  id: string;
}

const props = defineProps<Props>();
const wikiStore = useWikiStore();
const wiki = computed((): WikiData => {
  return wikiStore.getById(props.id);
});

// マークダウンへのパース処理
const textTitleData = '# ' + wiki.value.title + '\n\n';
const textBodyData = wiki.value.body;
const markdownData = textTitleData + textBodyData;
// チェックボックス更新の正（ソース・オブ・トゥルース）: Markdown全文
const currentMarkdown = ref(markdownData);
const bindHtml = ref(renderMarkdown(markdownData));
Prism.highlightAll();

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

// オーナーチェック
onMounted(() => {});
const checkIsOwner = (): boolean => {
  if (isOwner.value) {
    return true;
  } else {
    return false;
  }
};

getWikiOwner(props.id);

// Wikiデータの更新処理
const updateWikiData = (id: string): void => {
  updateViewRedirect(id);
};

// 目次モーダルの描画
const showTocContent = ref(false);
const openCloseTocModal = (): void => {
  if (showTocContent.value) {
    showTocContent.value = false;
  } else {
    showTocContent.value = true;
  }
};

// 目次モーダル表示時に灰色の部分のクリック時にも目次モーダルを閉じる処理
// HTMLが描画後に組み込む（onmoutedを利用）
onMounted(() => {
  // オーバレイとヘルプの内容を取得
  const tocModal = document.getElementById('overlay-toc');
  const tocModalContent = document.getElementById('content-toc');

  // 灰色部分クリック時にクローズ処理がなされるようにイベント設定
  if (tocModal) {
    tocModal.addEventListener('click', function (event) {
      if (showTocContent.value === true) {
        showTocContent.value = false;
      } else {
        return;
      }
    });
  }

  // 灰色の部分以外（content-toc）をクリックした時にはイベント伝搬を止め、クローズさせない
  if (tocModalContent) {
    tocModalContent.addEventListener('click', function (event) {
      event.stopPropagation();
    });
  }
});

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
  const headings = lines.filter((line) => {
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
    return 'No Table Of Contents.';
  }

  // 最終的なマークダウンテキスト
  let result = '';

  // 見出しの最初のレベルが2であることを確認するフラグ
  let isFirstTocLebelOk = false;

  // 見出しマークダウンの作成処理
  headings.forEach((heading, index) => {
    // '#'の数を取得
    const level = heading.split(' ').shift()!.length;

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
    let text = heading.replace(/#/g, '').trim();
    // '`'を除去
    text = text.replace(/`/g, '').trim();
    // インデントを付ける
    const indents = '  '.repeat(level - 1);

    const id = createId(index);
    // 目次のエントリにリンクを追加
    result += `${indents}- [${text}](#${id})\n`;
  });
  return result;
};

// 目次（マークダウン）からHTMLへ変換
const tocToHtml = computed((): string => {
  const tocListString = getToc();
  const mdTocStr = createNestedList(tocListString);
  const tocHtml = marked.parse(mdTocStr);
  return tocHtml as string;
});

// 画面上部（id=application-title）へスクロール
const scrollAppTitle = (): void => {
  const element = document.getElementById('application-title');
  if (element) {
    element.scrollIntoView({ behavior: 'smooth' });
  }
};

// id="heading--1"までスクロールされたらページ上部までスクロールさせるボタンを出現
const showScrollBtn = ref(false);
document.addEventListener('scroll', function () {
  let scrollPosition = window.scrollY;
  let targetElement = document.getElementById('heading--1');
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
};

// コンポーネントマウント時にmermaid.jsを発動
onMounted(() => {
  mermaid.init();
});

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

const oneTimeUrl = ref('');
const oneTimeUuid = ref('');
const onetimeDurationMinits = ref(60);
const genOnetimeWikiUrl = async (): Promise<void> => {
  try {
    if (Number.isInteger(onetimeDurationMinits.value) === false) {
      messageModalOpenClose('数値を入力してください。');
      return;
    }
    if (onetimeDurationMinits.value < 10) {
      messageModalOpenClose('10分以上の設定が必要です。');
      return;
    }
    const payload = {
      minutes: onetimeDurationMinits.value,
    };
    const url = generateOnetimeWikiUrl + `${props.id}`;
    const response = await apiClient.post(url, payload);

    if (isDevelopLocalhost.value) {
      oneTimeUrl.value = `${protocol}//${hostname}:4080${response.data['url']}`;
    } else {
      if (port === '') {
        // 本番環境（HTTPS + ドメイン時）
        oneTimeUrl.value = `${protocol}//${hostname}${port}${response.data['url']}`;
      } else {
        // 開発環境（HTTP or LOCALHOST時）
        oneTimeUrl.value = `${protocol}//${hostname}:${port}${response.data['url']}`;
      }
    }

    oneTimeUuid.value = response.data['id'];
    openCloseOnetimeUrl();
  } catch (error) {
    console.error('Error');
  }
};

const invalidateOneTimeWiki = async (): Promise<void> => {
  try {
    const url = invalidateOntimeWikiUrl + `${oneTimeUuid.value}`;
    const response = await apiClient.delete(url);
    openCloseOnetimeUrl();
    messageModalOpenClose('共有を停止しました。');
  } catch (error) {
    console.error('Error');
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
    messageModalOpenClose('クリップボードにコピーしました。');
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
        selection.removeAllRanges(); // 現在の選択をクリア
        selection.addRange(range); // 新しい範囲を選択
      }
    }
  }
}

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

// キーボードショートカットを追加
const handleKeyDown = (event: KeyboardEvent) => {
  // List.vueへ移動
  if (event.ctrlKey && event.key === '1') {
    event.preventDefault(); // デフォルトのブラウザのショートカットをキャンセル
    listRedirect();

    // 目次モーダル
  } else if (event.ctrlKey && event.key === '2') {
    event.preventDefault();
    openCloseTocModal();

    // 画像一覧モーダル
  } else if (event.ctrlKey && event.key === '3') {
    event.preventDefault();
    updateWikiData(props.id);

    // ワンタイムURL生成モーダル
  } else if (event.ctrlKey && event.key === '4') {
    event.preventDefault();
    openCloseOnetimeSetting();

    // ワンタイムURLのinputにフォーカス
  } else if (event.ctrlKey && event.key === 'i') {
    event.preventDefault();
    const inputTitleElement = document.getElementById('minits');
    if (inputTitleElement) {
      inputTitleElement.focus();
    }

    // QRコード生成モーダル
  } else if (event.ctrlKey && event.key === '5') {
    event.preventDefault();
    onOpenCloseQRCodeCreateModal();

    // Escapeキーでモーダルウィンドウをクローズ
  } else if (event.key === 'Escape') {
    event.preventDefault();
    if (isMessageModal.value) {
      isMessageModal.value = false;
    }
  }
};

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

// ── タスクチェックボックス機能 ────────────────────────────

/**
 * Markdown本文（タイトル行を除く）をバックエンドにPATCHで送信する。
 * エンドポイント: PUT /wiki/modify/{id}  payload: { title: string, body: string, is_public: boolean }
 */
const patchMarkdownBody = async (markdown: string): Promise<void> => {
  try {
    const bodyToSend = markdown.slice(textTitleData.length);
    const updateData = {
      id: props.id,
      title: wiki.value.title,
      body: bodyToSend,
      is_public: wiki.value.is_public,
    };
    await apiClient.put(patchWikiBodyUrl + `/${props.id}`, updateData);
    wikiStore.updateWiki(updateData);
  } catch (error) {
    console.error('Checkbox update error:', error);
  }
};

/**
 * チェックボックスのchangeイベントをデリゲーションで受け取る。
 * data-start / data-end を使って currentMarkdown 内の [ ] / [x] をトグルし、
 * 再レンダリングとバックエンド同期を行う。
 */
const onCheckboxChange = async (event: Event): Promise<void> => {
  const target = event.target as HTMLInputElement;
  if (target.tagName !== 'INPUT' || target.type !== 'checkbox') return;

  const startStr = target.dataset.start;
  const endStr = target.dataset.end;
  if (startStr === undefined || endStr === undefined) return;

  const start = parseInt(startStr, 10);
  const end = parseInt(endStr, 10);

  // オフセット検証
  if (isNaN(start) || isNaN(end) || start < 0 || end > currentMarkdown.value.length || start >= end)
    return;

  const before = currentMarkdown.value.slice(0, start);
  const targetStr = currentMarkdown.value.slice(start, end);
  const after = currentMarkdown.value.slice(end);

  // target が正しいチェックボックス記法でなければ何もしない
  if (!/^\[[ x]\]$/.test(targetStr)) return;

  const newCheckbox = target.checked ? '[x]' : '[ ]';
  const newMarkdown = before + newCheckbox + after;

  currentMarkdown.value = newMarkdown;
  bindHtml.value = renderMarkdown(newMarkdown);

  await nextTick();
  Prism.highlightAll();

  await patchMarkdownBody(newMarkdown);
};

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});
onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});
</script>

<template>
  <div class="head-btn-zone">
    <button class="btn-head-img" v-on:click="listRedirect()">
      <img :src="`${assetsUrl}home_24.png`" class="btn-img" alt="home_24.png" />
    </button>
    <button class="btn-head-img" v-if="checkIsOwner()" v-on:click="updateWikiData(wiki.id)">
      <img :src="`${assetsUrl}edit_24.png`" class="btn-img" alt="edit_24.png" />
    </button>
    <button v-else class="btn-head-img" v-on:click="updateWikiData(wiki.id)">
      <img :src="`${assetsUrl}person_edit_24.png`" class="btn-img" alt="person_edit_24.png" />
    </button>
    <button class="btn-head-img" v-on:click="openCloseTocModal">
      <img :src="`${assetsUrl}toc_24.png`" class="btn-img" alt="toc_24.png" />
    </button>
    <button class="btn-head-img" v-if="checkIsOwner()" v-on:click="openCloseOnetimeSetting()">
      <img :src="`${assetsUrl}family_line24.png`" class="btn-img" alt="family_line24.png" />
    </button>
    <button class="btn-head-img" v-on:click="onOpenCloseQRCodeCreateModal">
      <img
        :src="`${assetsUrl}code_reader_line24.png`"
        class="btn-img"
        alt="code_reader_line24.png"
      />
    </button>
    <button class="btn-head-img" v-on:click="openCloseSearchBar()">
      <img :src="`${assetsUrl}search_fill24.png`" class="btn-img" alt="search_fill24.png" />
    </button>
  </div>

  <div class="scrolled-btn-zone" v-show="showScrollBtn">
    <button class="btn-scroll-toc" v-on:click="openCloseTocModal">
      <img :src="`${assetsUrl}toc_24.png`" class="btn-img" alt="toc_24.png" />
    </button>
    <button class="btn-scroll-top" v-on:click="scrollAppTitle">
      <img :src="`${assetsUrl}arrow_upward_24.png`" class="btn-img" alt="arrow_upward_24.png" />
    </button>
    <button class="btn-scroll-tolist" v-on:click="listRedirect()">
      <img :src="`${assetsUrl}home_24.png`" class="btn-img" alt="home_24.png" />
    </button>
    <button class="btn-scroll-top" v-on:click="openCloseSearchBar()">
      <img :src="`${assetsUrl}search_fill24.png`" class="btn-img" alt="search_fill24.png" />
    </button>
    <button class="btn-scroll-update" v-if="checkIsOwner()" v-on:click="updateWikiData(wiki.id)">
      <img :src="`${assetsUrl}edit_24.png`" class="btn-img" alt="edit_24.png" />
    </button>
    <button class="btn-scroll-update" v-else v-on:click="updateWikiData(wiki.id)">
      <img :src="`${assetsUrl}person_edit_24.png`" class="btn-img" alt="person_edit_24.png" />
    </button>
  </div>

  <div class="contants-area">
    <div class="notoc">
      <div class="markdown-isprint">
        <section v-html="bindHtml" ref="contentEl" @change="onCheckboxChange"></section>
      </div>
      <div class="footer-zone" v-if="checkIsOwner()">
        <button v-on:click="onOpenDeleteViewModal()" class="btn-delete">削除</button>
        <p class="wiki-owner">Wikiオーナー：{{ wikiOwner.publicName }}</p>
      </div>
      <p class="wiki-owner" v-if="!isOwner">Wikiオーナー：{{ wikiOwner.publicName }}</p>
    </div>
  </div>

  <!-- 目次 -->
  <transition>
    <div id="overlay-toc" v-show="showTocContent">
      <div id="content-toc">
        <h2 class="toc-title">目次</h2>
        <div class="toc toc-content" v-html="tocToHtml"></div>
        <img
          id="toc-close"
          v-on:click="openCloseTocModal()"
          :src="`${assetsUrl}close24.png`"
          alt="close24.png"
        />
      </div>
    </div>
  </transition>

  <!-- 削除ビューへの遷移確認モーダル -->
  <transition>
    <div class="overlay" v-show="isDeleteModal">
      <div class="content">
        <h2 class="modal-h2">警告</h2>
        <p>
          <strong
            >削除画面へ移動します。削除しない場合は 「戻る」 ボタンを選択してください。</strong
          >
        </p>
        <div class="btn-zone">
          <button v-on:click="onOpenDeleteViewModal()">戻る</button>
          <button v-on:click="deleteRedirect(wiki.id)" class="btn-delete">続ける</button>
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

  <!-- ワンタイムWikiの有効化設定モーダル -->
  <div id="overlay-onetime-setting" v-show="isOnetimeSettingModal">
    <div id="content-onetime-setting">
      <h2 class="modal-h2">Wikiの共有リンクを作成</h2>
      <div class="input-area-duration">
        <label for="minits" style="font-size: 14px">有効期限（分）</label>
        <input
          v-model="onetimeDurationMinits"
          type="number"
          step="10"
          class="input-minits"
          id="minits"
        />
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
        <pre
          :id="oneTimeUuid"
          class="hidden-code-text"
        ><code :id=oneTimeUuid>{{ oneTimeUrl }}</code></pre>
        <button id="link-copy-btn" v-on:click="selectTextOrClipboardCopy(`${oneTimeUuid}`)">
          リンクを取得
        </button>
      </div>
      <div class="input-text-zone" v-else="isHttpsProtocol">
        <p><strong>共有リンクを作成しました。</strong></p>
        <pre><code :id=oneTimeUuid v-on:click="selectTextOrClipboardCopy(`${oneTimeUuid}`)">{{ oneTimeUrl }}</code></pre>
      </div>
      <div class="btn-zone">
        <button id="message-close-btn" v-on:click="openCloseOnetimeUrl()">閉じる</button>
        <button id="message-close-btn" v-on:click="invalidateOneTimeWiki()">共有停止</button>
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

  <div id="search-word-box" v-show="isShowSearchBar">
    <FindBar :container="contentEl" :showOpenInBrowser="true" />
  </div>
</template>

<style scoped>
.v-enter-active,
.v-leave-active {
  transition: all 0.3s ease-in-out;
}

.v-enter-from,
.v-leave-to {
  opacity: 0;
}

.contants-area {
  display: flex;
  height: 100%;
}

.notoc {
  width: 100%;
}

#toc-close {
  background-color: transparent;
  position: fixed;
  bottom: 1.5%;
  color: transparent;
  margin: 5px 5px 10px 5px;
  border: none;
  box-shadow: none;
}

.markdown-isprint {
  background-color: white;
  color: black;
  padding-left: 20px;
  padding-right: 20px;
  padding-bottom: 20px;
  margin-bottom: 20px;
  border-collapse: collapse;
}

.wiki-owner {
  text-align: right;
  font-size: 16px;
  font-weight: bold;
  text-shadow: 1px 1px 2px rgb(202, 202, 202);
}

/* 目次モーダル */
#overlay-toc {
  z-index: 3;
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

/* 目次モーダルのコンテンツ */
#content-toc {
  z-index: 4;
  height: 100%;
  width: 70%;
  padding: 1em;
  margin-left: 30%;
  background: #fff;
  overflow-y: scroll;
}

.toc-title {
  font-size: 17px;
}

.toc {
  color: white;
  margin-left: -5%;
}

.scrolled-btn-zone {
  z-index: 3;
  position: fixed;
  bottom: 9%;
  left: 5%;
}

.btn-scroll-toc,
.btn-scroll-top,
.btn-scroll-tolist,
.btn-scroll-update {
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
  transition-duration: 0.5s;
  -webkit-transition-duration: 0.5s;
  transition: background-color 0.3s;
  margin: 5px 7px 10px 5px;
}

.footer-zone {
  justify-content: space-between;
}

.overlay {
  z-index: 4;
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
  z-index: 5;
  width: 80%;
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

#overlay-onetime-setting {
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

#content-onetime-setting {
  z-index: 4;
  width: 90%;
  padding: 1em;
  background: whitesmoke;
  border-radius: 10px;
}

.input-area-duration {
  margin-top: 3%;
  margin-bottom: 3%;
  display: flex;
  justify-content: center;
}

.input-minits {
  font-size: 16px;
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
  width: 90%;
  padding: 1em;
  background: whitesmoke;
  border-radius: 10px;
  text-align: center;
}

/* QRコード生成モーダル */
#overlay-gen-qrcode {
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
  text-align: center;
}

#content-gen-qrcode {
  z-index: 4;
  width: 90%;
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
  font-size: 18px;
  width: 90%;
  height: 26px;
  text-align: center;
  margin-bottom: 2%;
  border-radius: 5px;
}

#link-copy-btn {
  width: 180px;
  background: rgb(27, 168, 161);
  height: 45px;
  font-size: 16px;
  color: #fff;
  padding: 10px 7px;
  text-decoration: none;
  border: 1px;
  border-radius: 20px;
  transition-property: opacity;
  -webkit-transition-property: opacity;
  transition-duration: 0.5s;
  -webkit-transition-duration: 0.5s;
  margin: 5px 5px 10px 5px;
}

.input-text-zone {
  text-align: center;
}

.hidden-code-text {
  display: none;
}

#search-word-box {
  z-index: 3;
  position: fixed;
  bottom: 20%;
}
</style>
