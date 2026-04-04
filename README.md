# MarkdownWiki2-SingleBin

Rust/Axum 製 API サーバーと Vue 3 製フロントエンドを 1 つの配布物にまとめた Markdown Wiki アプリケーション。Tauri 2 によるデスクトップアプリとして動作し、SQLite をデータベースとして使用する。

> このプロジェクトは PostgreSQL をデータベースに使用したフル構成の [MarkdownWiki2](https://www.marudev.org/) を Windows 環境でオフライン動作させることを目的に再構築したもの。ゆえに **SingleBin**。

**現時点では Windows のみサポートしています。** 配布形式は Windows NSIS インストーラ（`.exe`）です。

## スクリーンショット

![プレビュー](https://markdown-wiki2-single.pages.dev/02_markdown-wiki2-single-preview.png)

![Wiki作成](https://markdown-wiki2-single.pages.dev/01_markdown-wiki2-single-create.png)

## 機能

### Wiki 編集・閲覧

- **Ace Editor** による Markdown 編集（Vim キーバインド切替対応）
- リアルタイム **プレビュー**（表示/非表示切替可）
- **目次** 自動生成
- **ページ内検索**
- タイトル・本文の **2 語キーワード検索**
- **Markdown ファイルダウンロード**
- **タスクリストチェックボックス**：プレビュー上でチェックするとそのまま保存される（オーナーのみ）
- **QR コード生成** モーダル
- **共有メモ** モーダル

Markdown の表現力:

| 機能 | 詳細 |
| ---- | ---- |
| コードハイライト | Prism.js |
| 数式 | KaTeX（`html2canvas` による画像化にも対応）|
| ダイアグラム | Mermaid |
| 動画 | MP4 埋め込み、YouTube 埋め込み |
| 拡張トークン | 詳細表示、注意表示など |

### ファイル管理

アップロード対応形式: **PNG / JPG / JPEG / GIF / WebP / PDF / MP4**

- アップロード・一覧表示・プレビュー・削除
- 画像は **再エンコードして保存**（EXIF 等のメタデータを除去）
- **Markdown 埋め込み文字列を自動生成**
  - 画像: `![alt](url)`
  - PDF: `[name](url)`
  - 動画: `?[name](url)`
- ファイルは `~/.markdown-wiki2-single/images/<先頭5文字>/<uuid>` に保存

### 公開・共有

**公開/非公開切替**

- Wiki ごとに公開フラグを管理する
- 他ユーザーは公開 Wiki のみ閲覧可

**期限付き共有 URL**

- オーナーが Wiki ごとに発行（10 分以上の有効期限を指定）
- 発行時点のタイトル・本文を **スナップショットとして保持**（その後の編集は反映されない）
- 同一 Wiki への二重発行は不可（既存 URL を削除してから新規発行）
- 期限切れ URL へのアクセス時に自動削除し、専用 Not Found ページを返す

**プライバシーモード**

- ユーザー単位で画像の外部公開を制御する
- プライバシーモード ON の場合、自分以外からの画像アクセスを遮断する

### 更新申請フロー（協業）

他ユーザーの公開 Wiki に対して変更内容を提案できる。

1. 非オーナーが Wiki の更新申請を送信する
2. Wiki に「申請中」フラグが立ち、多重申請を防止する
3. オーナーが **差分を確認** して承認または却下する
4. **承認**：Wiki の本文・タイトルが申請内容で更新される
5. **却下**：申請レコードが `REJECT` ステータスになる
6. 申請者は承認前に取り下げ可能

### 認証・セキュリティ

**認証方式**

- アクセストークン + リフレッシュトークンを JWT で発行
- 両トークンは **HttpOnly Cookie** で保持（`SameSite=Strict`）
- リフレッシュトークンは専用パス `/account/refresh` に限定

**TOTP 二段階認証**

1. QR コードをアプリで読み取る
2. 6 桁のコードを入力して検証する
3. 検証成功でシークレットが本番昇格される
4. 以降のログインはパスワード認証後に TOTP コード入力が必要（一次認証の有効期限は 3 分）

**ログイン失敗制御**

- 失敗回数が閾値を超えると一定時間の再試行待ち状態になる
- 上限回数に達するとアカウントがロックされる（管理者が解除可能）
- ログイン成功時に失敗カウントをリセットする

### 管理者機能

- ユーザー一覧取得
- 一般ユーザー作成
- パスワード再設定
- 表示名（公開名）変更
- アカウントロック解除

### 権限モデル

| ロール | 内容 |
| ------ | ---- |
| 管理者 | 初回セットアップで作成。管理系 API すべてにアクセス可 |
| 一般ユーザー | 管理者が作成、またはサインアップ（設定で制御）|
| Wiki オーナー | 対象 Wiki の作成者。更新・削除・共有 URL 発行が可能 |

### モバイル対応

User-Agent に `Mobile` が含まれるアクセスにはモバイル向け UI (`index-mobile.html`) を返す。一時共有 URL のプレビューも同様にモバイル用レイアウトを使い分ける。

## 技術スタック

```txt
バックエンド  : Rust 2024 + Axum + SQLx + SQLite
フロントエンド: Vue 3 + Vue Router + Pinia + Vite
デスクトップ  : Tauri 2
テンプレート  : Tera
認証          : JWT + HttpOnly Cookie + TOTP
アセット埋込  : rust-embed
エディタ      : Ace Editor
表示拡張      : marked + Prism + KaTeX + Mermaid
配布          : GitHub Actions + NSIS Installer
```

## セットアップ

インストーラで導入後、アプリを起動する。初回起動時はセットアップ画面が表示され、管理者アカウントや各種設定を入力する。

![初回セットアップ](https://markdown-wiki2-single.pages.dev/03_markdown-wiki2-single-init-setup.png)

初回セットアップで設定できる項目:

| 項目 | 内容 |
| ---- | ---- |
| アプリタイトル | ウィンドウタイトルや UI に表示される名称 |
| 管理者ユーザー名 / パスワード | 初期管理者アカウント |
| アカウントロック回数 | この回数の失敗でアカウントをロックする |
| 待機制限開始回数 | この回数以上の失敗で再試行待ち状態にする |
| 待機時間（分） | 再試行待ち状態の継続時間 |
| アクセストークン有効期限（分） | |
| リフレッシュトークン有効期限（分） | |

設定完了後、以下が自動生成されてそのまま Wiki が使い始められる。

```txt
~/.markdown-wiki2-single/
  ├── markdown-wiki2-single.env.json   # 設定ファイル
  ├── markdown-wiki2.sqlite            # SQLite DB
  └── images/                          # アップロードファイル保存先
```

## 起動方法

### 通常起動（Tauri デスクトップアプリ）

インストーラで導入後、アプリを起動する。

### サーバー単体モード

GUI を使わずコマンドラインで Axum サーバーを起動する。事前に Tauri GUI でセットアップを完了しておく必要がある。

```bash
# ホストのみ指定（ポートは 3080 を使用）
markdown_wiki2_single -s 0.0.0.0

# ホスト:ポート形式でポートを指定
markdown_wiki2_single -s 0.0.0.0:9090
```

Ctrl+C でグレースフルシャットダウンする。

> **HTTP 環境で運用する場合**
> デフォルトでは `SECURE_COOKIE=true` のため、HTTPS なしの環境では Cookie が機能しない。
> `~/.markdown-wiki2-single/markdown-wiki2-single.env.json` の `"secure_cookie"` を `"false"` に変更すること。

## 開発環境のセットアップ

### 1. 環境変数の設定（`.env`）

```
DATABASE_URL=sqlite:/path/to/MarkdownWiki2-SingleBin/markdown-wiki2.sqlite
CREATEDATABASE_PATH=/path/to/MarkdownWiki2-SingleBin/markdown-wiki2.sqlite

# 開発時（フロントエンド開発サーバー使用時）
# VITE_IP_ADDRESS=http://localhost:3080
# VITE_ASSET_PATH=/

# フロントエンドビルド時
VITE_IP_ADDRESS=
VITE_ASSET_PATH=/assets/
```

### 2. SQLite データベースの作成

```bash
sqlx database create
sqlx migrate run
```

### 3. フロントエンドのビルド

```bash
cd src_frontend/scripts
./frontends-builder.ps1
```

### 4. sqlx オフラインクエリの準備

```bash
cargo sqlx prepare
```

### 5. Tauri インストーラのビルド

```bash
cargo tauri build
```

## GitHub Release ワークフロー

GitHub Actions では [.github/workflows/release.yml](.github/workflows/release.yml) により、Windows 向けリリースビルドを行う。

- トリガーは `v*` タグ push、または `workflow_dispatch`
- 実行環境は `windows-latest`
- ルートに CI 用の `.env` を生成し、`sqlx-cli` で SQLite スキーマを作成する
- `src_frontend/scripts/frontends-builder.ps1` で 3 系統のフロントエンド成果物を `dist/` に集約する
- `cargo tauri build` で Windows NSIS インストーラを生成する
- 生成した `.exe` / `.msi` を収集し、SHA-256 チェックサム付きで GitHub Release の draft に添付する

## Docker によるバイナリビルド（サーバー単体モード用）

Linux バイナリをクロスビルドする場合に使用する。

```bash
docker build -t markdown-wiki2-single .
docker run --name markdown-wiki2-app markdown-wiki2-single
docker cp markdown-wiki2-app:/web/target/release/markdown_wiki2_single .
```

## Nginx プロキシサーバーを Docker で用意

```bash
docker run --name proxy-nginx --network=host -p 80:80 \
  -v $(pwd)/utils/nginx/nginx.conf.template:/etc/nginx/nginx.conf:ro \
  -d nginx
```
