import { ref, watch, onMounted, onBeforeUnmount, type Ref } from "vue";

type Options = {
    normalizeJa?: boolean; // 日本語正規化（NFKC + カナ同一視 + lower）
    hotkey?: string; // 入力フォーカスのホットキー（例: '/'）
    observeMutations?: boolean; // DOM更新を監視して再検索
};

type MatchPos = { 
    node: Text;
    start: number;
    end: number;
};
const SKIP_PARENTS = new Set(["SCRIPT", "STYLE", "NOSCTIPT", "INPUT", "TEXTAREA"]);

function kataToHira(s: string): string {
    return s.replace(/[\u30A1-\u30FA\u30FD\u30FE\u30FF]/g, ch => 
        String.fromCharCode(ch.charCodeAt(0) - 0x60)
    );
};

// 1文字ずつ正規化し、正規化結果と「正規化後各文字 → 原文インデックス」の写像を構築する
function normalizeWithMap(orig: string) {
    const normPieces: string[] = [];
    const map: number[] = [];

    // サロゲートペアも考慮して code point で回す
    for (const ch of Array.from(orig)) {
        let n = ch.normalize("NFKC").toLowerCase();
        n = kataToHira(n);
        normPieces.push(n);
        // n が複数文字に展開される場合も、各文字に同じ原文位置を対応づける
        for (let i = 0; i < n.length; i++) map.push(normPieces.join("").length - n.length + i >= 0 ? (map.length ? map[map.length-1] + 0 : 0) : 0);
    }
    
    // 上の複雑化を避けて書き直し
    const normArr: string[] = []; 
    const idxMap: number[] = [];
    let iOrig = 0;
    for (const ch2 of Array.from(orig)) {
        let n2 = kataToHira(ch2.normalize("NFKC").toLowerCase());
        for (const nCh of Array.from(n2)) {
            normArr.push(nCh);
            idxMap.push(iOrig);
        }
        iOrig += 1;
    }
    return { norm: normArr.join(''), map: idxMap };
};

function shouldSkip(node: Node): boolean {
    const p = node.parentElement?.tagName ?? "";
    return SKIP_PARENTS.has(p.toUpperCase());
};

// 祖先の<details>をすべてopenにし、どれを自動で開けたかマーキング
function revealDetailsAncestores(el: HTMLElement) {
    const opened: HTMLDetailsElement[] = [];
    let cur: HTMLElement | null = el;
    while (cur && cur !== document.body) {
        if (cur.tagName?.toLowerCase() === "details") {
            const det = cur as HTMLDetailsElement;
            if (!det.open) {
                det.open = true;
                det.dataset.findAutoOpen = ""; // 自動で開けた印
                opened.push(det);
            }
        }
        cur = cur.parentElement as HTMLElement | null;
    }
    return opened;
};

export function usePageFind(containerRef: Ref<HTMLElement | null>, opts: Options = {}) {
    const { normalizeJa = true, hotkey = "/", observeMutations = true } = opts;

    const query = ref("");
    const count = ref(0);
    const current = ref(-1);
    const _marks = ref<HTMLElement[]>([]);
    const _observer = ref<MutationObserver | null>(null);

    function unwrapMarks(root: HTMLElement) {
        const marks = root.querySelectorAll("mark[data-find]");
        marks.forEach(m => {
            const text = document.createTextNode(m.textContent || "");
            m.replaceWith(text);
        });
        // 連続テキストノードを結合
        root.normalize();
    };

    function getTextNodes(root: HTMLElement): Text[] {
        const out: Text[] = [];
        const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
            acceptNode(node) {
                if (shouldSkip(node)) return NodeFilter.FILTER_REJECT;
                if (!node.nodeValue || !node.nodeValue.trim()) return NodeFilter.FILTER_REJECT;
                return NodeFilter.FILTER_ACCEPT;
            }
        });
        while (walker.nextNode()) out.push(walker.currentNode as Text);
        return out;
    };

    function searchAll(qRaw: string) {
        const root = containerRef.value;
        if(!root) return;

        unwrapMarks(root);
        _marks.value = [];
        count.value = 0;
        current.value = -1;

        const qn = normalizeJa ? kataToHira(qRaw.normalize('NFKC').toLowerCase()) : qRaw;
        if (!qn) return;

        const nodes = getTextNodes(root);
        const perNodeMatches: MatchPos[] = [];

        for (const node of nodes) {
            const text = node.nodeValue || "";
            const { norm, map } = normalizeJa ? normalizeWithMap(text) : { norm: text, map: [...text].map((_, i) => i) };

            let from = 0;
            for (;;) {
                const pos = norm.indexOf(qn, from);
                if (pos === -1) break;
                const startOrig = map[pos];
                const endOrigIdx = map[pos + qn.length - 1];
                const endOrig = (endOrigIdx ?? startOrig) + 1;
                perNodeMatches.push({ node, start: startOrig, end: endOrig });
                from = pos + (qn.length || 1);
            }
        }

        // 各Textノード内は末尾からwrapしてズレを防止
        const byNode = new Map<Text, MatchPos[]>();
        for (const m of perNodeMatches) {
            if (!byNode.has(m.node)) byNode.set(m.node, []);
            byNode.get(m.node)!.push(m);
        }

        for (const [node, arr] of byNode) {
            arr.sort((a, b) => b.start - a.start); // 末尾から
            for (const m of arr) {
                const range = document.createRange();
                range.setStart(node, m.start);
                range.setEnd(node, m.end);
                const mark = document.createElement("mark");
                mark.setAttribute("data-find", "");
                range.surroundContents(mark);
            }
        }

        _marks.value = Array.from(root.querySelectorAll("mark[data-find]")) as HTMLElement[];
        count.value = _marks.value.length;
        if (count.value > 0) {
            current.value = 0;
            updateCurrent();
        }
    }

    function updateCurrent() {
        const marks = _marks.value;
        marks.forEach(m => m.classList.remove("is-current"));
        if (current.value >= 0 && marks[current.value]) {
            const target = marks[current.value];
            target.classList.add("is-current");

            // <details> タグを開く
            revealDetailsAncestores(target);

            requestAnimationFrame(() => {
                target.scrollIntoView({ block: "center", inline: "nearest", behavior: "smooth" });
            })
        }
    }

    function next() {
        if (!count.value) return;
        current.value = (current.value + 1) % count.value;
        updateCurrent();
    }

    function prev() {
        if (!count.value) return;
        current.value = (current.value - 1 + count.value) % count.value;
        updateCurrent();
    }

    function clear() {
        const root = containerRef.value;
        if (!root) return;
        query.value = "";
        unwrapMarks(root);

        _marks.value = [];
        count.value = 0;
        current.value = -1;
    }

    // キーバインド
    const _onKeydown = (e: KeyboardEvent) => {
        if (!containerRef.value) return;
        const active = document.activeElement as HTMLElement | null;
        const isTyping = active && (active.tagName === "INPUT" || active.tagName === "TEXTAREA" || active.isContentEditable);

        if (e.key === hotkey && !isTyping) {
            e.preventDefault();
            const input = containerRef.value.querySelector<HTMLInputElement>("[data-find-input]");
            input?.focus();
            input?.select();
        } else if (e.key === "Enter" && (active?.dataset.findInput !== undefined)) {
            e.preventDefault();
            e.shiftKey ? prev() : next();
        }
    };

    function bind() {
        document.addEventListener("keydown", _onKeydown);
        if (observeMutations && containerRef.value) {
            _observer.value = new MutationObserver(() => {
                if (query.value) searchAll(query.value); // DOMが変わったら再評価
            });
            _observer.value.observe(containerRef.value, { childList: true, characterData: true, subtree: true });
        }
    };

    function unbind() {
        document.removeEventListener("keydown", _onKeydown);
        _observer.value?.disconnect();
    }

    watch(query, (q) => searchAll(q));

    onMounted(bind);
    onBeforeUnmount(unbind);

    const countLabel = () => count.value ? `${Math.max(1, current.value + 1)} / ${count.value}` : "0 / 0";

    return { query, count, current, countLabel, searchAll, next, prev, clear };
}