# MarkdownWiki2-SingleBin

Rust/Axum 製 API サーバーと Vue 3 製フロントエンドを 1 つの配布物にまとめた Markdown Wiki アプリケーション。Tauri 2 によるデスクトップアプリとして動作し、SQLite をデータベースとして使用する。

## サポートプラットフォーム

**現時点では Windows のみサポートしています。**
配布形式は Windows NSIS インストーラ（`.exe`）です。

## 主な機能

- Markdown ベースの Wiki 作成・閲覧・更新・削除
- 公開/非公開 Wiki の管理
- 画像・PDF・動画のアップロードと Markdown への埋め込み
- 公開 Wiki に対する更新申請フロー
- 期限付き共有 URL の発行
- Cookie + JWT 認証 / TOTP 二段階認証

## 起動方法

### 通常起動（Tauri デスクトップアプリ）

インストーラで導入後、アプリを起動する。初回起動時はセットアップ画面が表示され、管理者アカウントや各種設定を入力する。設定完了後は `~/.markdown-wiki2-single/` に設定ファイルと SQLite DB が自動生成される。

### サーバー単体モード

GUI を使わずコマンドラインで Axum サーバーを起動する。事前に Tauri GUI でセットアップを完了しておく必要がある。

```bash
# ホストのみ指定（ポートは 3080 を使用）
markdown_wiki2_single -s 0.0.0.0

# ホスト:ポート形式でポートを指定
markdown_wiki2_single -s 0.0.0.0:9090
```

Ctrl+C でグレースフルシャットダウンする。

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

## ローカルネットワーク内で HTTP 運用する場合

デフォルトでは `SECURE_COOKIE=true` のため、HTTPS なしの環境では Cookie が機能しない。HTTP 環境で運用する場合は `~/.markdown-wiki2-single/markdown-wiki2-single.env.json` を編集する。

```json
- "secure_cookie": "true",
+ "secure_cookie": "false",
```
