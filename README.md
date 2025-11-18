# MarkdownWiki2-SingleBin

シングルバイナリで配布可能な MarkdownWiki2 アプリケーション。データベースに SQLite を使用。

## 環境変数

ビルドを含めた開発環境時に必要となる環境変数.env。

```
DATABASE_URL=sqlite:/path/to/your-path/MarkdownWiki2-SingleBin/markdown-wiki2.sqlite
CREATEDATABASE_PATH=/path/to/your-path/MarkdownWiki2-SingleBin/markdown-wiki2.sqlite

# 開発環境時（フロントエンドのビルド時にコメントアウト）
# VITE_IP_ADDRESS=http://localhost:3080
# VITE_ASSET_PATH=/

# フロントエンドビルド時
VITE_IP_ADDRESS=
VITE_ASSET_PATH=/assets/
```

## SQLiteデータベース作成

```bash
sqlx database create
sqlx migrate run
cd src_frontend/scripts
./frontends-builder.sh # Windows環境の場合は ./frontends-builder.ps1
cargo sqlx prepare
```

## Dockerによるバイナリビルド

Dockerコンテナを使用してバイナリを作成する用途

```bash
docker build -t markdown-wiki2-single .
docker run --name markdown-wiki2-app markdown-wiki2-single
docker cp markdown-wiki2-app:/web/target/release/markdown_wiki2_single .
docker cp markdown-wiki2-app:/web/dist .
```

## NginxプロキシサーバーをDockerで用意

```bash
docker run --name proxy-nginx --network=host -p 80:80 -v $(pwd)/utils/nginx/nginx.conf.template:/etc/nginx/nginx.conf:ro -d nginx
```

## ローカルネットワーク内でHTTP運用

デフォルトでは HTTP 環境下では使用できないため、明示的に `markdown-wiki2-single.env.json` ファイル内の次の行を変更する。

```json
- "secure_cookie": "true",
+ "secure_cookie": "false",
```
