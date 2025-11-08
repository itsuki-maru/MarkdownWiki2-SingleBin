import { marked } from "marked";
import type { Token } from "marked";
import katex from "katex";

// カスタムトークンの型定義
interface CustomVideoToken {
    type: "video" | Token["type"]; // 既存の型に "video"を追加
    href: string;
    text: string;
}

// カスタムトークン"video"の定義（型は緩くanyとする）
const videoToken: any = {
    name: "video",
    level: "inline",
    start(src: string) {
        return src.match(/\?\[.*\]\(.*\)/)?.index;
    },
    tokenizer(src: string, tokens: Token[]): CustomVideoToken | null {
        const rule = /^\?\[(.*?)\]\((.*?)\)/;
        const match = rule.exec(src);
        if (match) {
            return {
                type: "video", // カスタムトークンタイプ
                raw: match[0],
                text: match[1],
                href: match[2],
            } as CustomVideoToken; // 型アサーション
        }
        return null;
    },
    renderer(token: CustomVideoToken) {
        return `<video controls src="${token.href}">${token.text}</video>`;
    }
};

// カスタムトークンの型定義 YouTubeのみ埋め込みを実現
interface CustomYouTubeToken {
    type: "youtube" | Token["type"]; // 既存の型に "youtube"を追加
    href: string;
    text: string;
}

// カスタムトークン"youtube"の定義（型は緩くanyとする）
const youtubeToken: any = {
    name: "youtube",
    level: "inline",
    start(src: string) {
        return src.match(/\?\[.*\]\(.*\)/)?.index;
    },
    tokenizer(src: string, tokens: Token[]): CustomYouTubeToken | null {
        const rule = /^\@\[(youtube)\]\((.*?)\)/;
        const match = rule.exec(src);
        if (match) {
            const id = extractYouTubeId(match[2]);
            if (!id) return null;
            return {
                type: "youtube", // カスタムトークンタイプ
                raw: match[0],
                text: id,
                href: match[2],
            } as CustomYouTubeToken; // 型アサーション
        }
        return null;
    },
    renderer(token: CustomYouTubeToken) {
        // 生iframeではなく、自前テンプレートにする（例：Web Component）
        return `<app-youtube video-id="${token.text}" data-src="${token.href}"></app-youtube>`;
    }
};

// 11文字のYouTube ID検証
const ID_RE = /^[\w-]{11}$/;
function extractYouTubeId(rawUrl: string): string | null {
    try {
        const url = new URL(rawUrl);
        const host = url.hostname.toLowerCase();
        const allowYouTubeList = [
            "www.youtube.com",
            "youtube.com",
            "m.youtube.com",
            "youtu.be",
            "www.youtube-nocookie.com"
        ]
        if (!allowYouTubeList.includes(host)) return null;

        // shorts / watch / youtu.be に対応
        if (host === "youtu.be") {
            const id = url.pathname.slice(1);
            return ID_RE.test(id) ? id : null;
        };
        if (url.pathname.startsWith("/shorts/")) {
            const id = url.pathname.split("/")[2] ?? "";
            return ID_RE.test(id) ? id : null;
        };
        if (url.pathname === "/watch") {
            const id = url.searchParams.get("v") ?? "";
            return ID_RE.test(id) ? id : null;
        };
        if (url.pathname.startsWith("/embed/")) {
            const id = url.pathname.split("/")[2] ?? "";
            return ID_RE.test(id) ? id : null;
        };
        return null;
    } catch { return null; }
}

// 共通インターフェース
interface CustomDetailsToken {
    type: "details" | "note" | "warning" | Token["type"];
    raw: string;
    title: string;
    tokens: Token[];
}

// ネスト対応トークナイザの共通関数
function createNestedTokenizer(typeName: "details" | "note" | "warning") {
    return {
        name: typeName,
        level: "block",
        start(src: string) {
            const re = new RegExp(`^:::${typeName}\\s`, "m");
            return src.match(re)?.index;
        },
        tokenizer(src: string, tokens: Token[]): CustomDetailsToken | null {
            const self = this as any;
            if (!src.startsWith(`:::${typeName}`)) return null;

            const lines = src.split(/\r?\n/);
            let nestLevel = 0;
            let endIndex = -1;

            for (let i = 0; i < lines.length; i++) {
                const line = lines[i].trim();
                if (/^:::(\w+)/.test(line)) {
                    nestLevel++;
                } else if (/^:::\s*$/.test(line)) {
                    nestLevel--;
                    if (nestLevel === 0) {
                        endIndex = i;
                        break;
                    }
                }
            }

            if (endIndex === -1) return null;

            const rawLines = lines.slice(0, endIndex + 1);
            const raw = rawLines.join("\n");

            const titleMatch = lines[0].match(new RegExp(`^:::${typeName}\\s+(.+)`));
            const title = titleMatch ? titleMatch[1].trim() : typeName.toUpperCase();

            const content = lines.slice(1, endIndex).join("\n");

            return {
                type: typeName,
                raw,
                title,
                tokens: self.lexer.blockTokens(content),
            } as CustomDetailsToken;
        },
        renderer(token: CustomDetailsToken) {
            const body = marked.parser(token.tokens);
            if (token.type === "details") {
                return `<details>\n<summary>${token.title}</summary>\n${body}\n</details>\n`;
            } else {
                return `<div class="box ${token.type}">\n<summary>${token.title}</summary>\n${body}\n</div>\n`;
            }
        },
    };
}

// それぞれのトークンを生成
const detailsToken = createNestedTokenizer("details");
const noteToken = createNestedTokenizer("note");
const warningToken = createNestedTokenizer("warning");

// Katexカスタムトークンの型定義
interface CustomKatexToken {
    type: "math" | Token["type"]; // 既存の型に "math"を追加
    text: string;
    displayMode: boolean;
}

const mathExtentionToken: any = {
    name: "math",
    level: "inline",
    start(src: string) {
        return src.match(/\$+/)?.index;
    },
    tokenizer(src: string, _tokens: Token[]): CustomKatexToken | null {
        const blockMath = /^\$\$([^$]+)\$\$/; // $$...$$
        const inlineMath = /^\$([^$\n]+)\$/;  // $...$

        const blockMatch = blockMath.exec(src);
        if (blockMatch) {
            return {
                type: "math",
                raw: blockMatch[0],
                text: blockMatch[1],
                displayMode: true,
            } as CustomKatexToken;
        }
        const inlineMatch = inlineMath.exec(src);
        if (inlineMatch) {
            return {
                type: "math",
                raw: inlineMatch[0],
                text: inlineMatch[1],
                displayMode: false,
            } as CustomKatexToken;
        }
        return null;
    },
    renderer(token: any) {
        try {
            return katex.renderToString(token.text, {
                throwOnError: false,
                displayMode: token.displayMode,
                output: "html",
            });
        } catch (error) {
            return token.text;
        }
    }
};

// app-youtubeからiframeに置換
function renderIframe(html: string): string {
    return html.replace(
        /<app-youtube\s+[^>]*video-id=["']([\w-]{11})["'][^>]*>(?:<\/app-youtube>)?/g,
        (_, videoId) => {
            const src = `https://www.youtube-nocookie.com/embed/${videoId}`;
            return `
            <iframe
                src="${src}"
                title="YouTube video player"
                frameborder="0"
                allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin"
                allowfullscreen
                width="560" height="315"
                style="border:0;"
            ></iframe>
            `.trim();
        }
    );
};

export { videoToken, detailsToken, noteToken, warningToken, mathExtentionToken, youtubeToken, renderIframe }
