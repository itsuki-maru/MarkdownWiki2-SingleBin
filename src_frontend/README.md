# src_frontend

フロントエンドの全プロジェクトと、ビルドスクリプトを管理するディレクトリ。

## ディレクトリ構成

```
src_frontend/
├── frontend/          # PC向け Vue3 SPA
├── frontend-mobile/   # モバイル向け Vue3 SPA
├── frontend-admin/    # 管理者向け Vue3 SPA
├── main/              # ビルド成果物の集積ディレクトリ（ビルド時に自動生成）
└── scripts/
    ├── frontends-builder.sh   # ビルドスクリプト（Linux / macOS）
    └── frontends-builder.ps1  # ビルドスクリプト（Windows PowerShell）
```

## ビルドフロー

`scripts/frontends-builder.sh`（または `.ps1`）を実行すると、以下の順で処理される。

```
frontend/        npm run build  ─┐
frontend-mobile/ npm run build  ─┼─→ main/dist/ に集約 ─→ プロジェクトルート dist/ にコピー
frontend-admin/  npm run build  ─┘
```

最終的にプロジェクトルートの `dist/assets/` に全静的ファイルが集まり、
Rust の `rust-embed` によってバイナリに埋め込まれる。

## src/templates/ との関係

`src/templates/` 配下の HTML ファイル（Rust バックエンドのサーバーサイドテンプレート）は、
`/assets/` パス経由で以下のライブラリを参照している。

| テンプレート | 参照する主なアセット |
|---|---|
| `preview.html` | marked, xss, mermaid, prismjs（コア＋言語）, katex, github.css |
| `preview-mobile.html` | 同上 |
| `notfound.html` | github.css |

これらのファイルは Vite の `public/` で管理していたが、**npm に移行済み**。
`frontend/package.json` の dependencies として宣言し、
`frontend/scripts/copy-preview-assets.js` が `npm run build` 後に
`node_modules/` から `dist/assets/` へコピーする。

### コピー対象と npm パッケージの対応

| 配置先ファイル | npm パッケージ | コピー元パス |
|---|---|---|
| `marked.min.js` | `marked` | `lib/marked.umd.js` |
| `xss.min.js` | `xss` | `dist/xss.min.js` |
| `mermaid.min.js` | `mermaid` | `dist/mermaid.min.js` |
| `prism.js` | `prismjs` | `prism.js` |
| `prism-okaidia.css` | `prismjs` | `themes/prism-okaidia.css` |
| `prism-{lang}.js` | `prismjs` | `components/prism-{lang}.js` |
| `katex.min.js` | `katex` | `dist/katex.min.js` |
| `katex.min.css` | `katex` | `dist/katex.min.css` |
| `auto-render.min.js` | `katex` | `dist/contrib/auto-render.min.js` |
| `fonts/` | `katex` | `dist/fonts/`（katex.min.css が相対参照） |
| `github.css` | `github-markdown-css` | `github-markdown.css` |

### アセットバージョンの管理方法

`src/templates/` が使うライブラリのバージョンは `frontend/package.json` で一元管理する。
バージョンを更新する際は `frontend/` で `npm install` を実行すればよい。

```bash
cd src_frontend/frontend
npm install <package>@<version>
```
