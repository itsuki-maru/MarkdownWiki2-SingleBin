# MarkdownWiki2-SingleBin 仕様書

## 1. 概要

本プロジェクトは、Rust/Axum 製の API サーバーと Vue 3 製のフロントエンドを 1 つの配布物にまとめた Markdown Wiki アプリケーションである。通常は Tauri アプリとして動作し、初回セットアップ後にローカル HTTP サーバーを内蔵起動して UI を表示する。加えて `-s` オプションによるサーバー単体モードも備える。

主な目的は以下のとおり。

- Markdown ベースの Wiki を作成、閲覧、更新、削除する
- Wiki を公開/非公開で管理する
- 画像、PDF、動画をアップロードし、Markdown から参照する
- 公開 Wiki に対する更新申請フローを扱う
- 期限付き共有 URL を発行する
- Cookie + JWT による認証と TOTP 二段階認証を提供する

## 2. システム構成

### 2.1 バックエンド

- 言語: Rust 2024
- Web: Axum
- DB: SQLite
- テンプレート: Tera
- 認証: JWT(access/refresh) + HttpOnly Cookie
- 二段階認証: TOTP
- 静的ファイル配布: `rust-embed`

### 2.2 フロントエンド

- Vue 3 + Vue Router + Pinia
- Vite ビルド
- Ace Editor による Markdown 編集
- `marked` + `Prism` + `KaTeX` + `mermaid` によるプレビュー強化
- `html2canvas` による数式画像化
- Service Worker 登録あり

### 2.3 デスクトップラッパー

- Tauri 2
- 通常起動時は `http://localhost:3080/index` を WebView で開く
- 外部リンクは Tauri コマンド経由で既定ブラウザに委譲する

## 3. 起動モード

### 3.1 Tauri 通常起動

- 設定ファイル `~/.markdown-wiki2-single/markdown-wiki2-single.env.json` がない場合、初回セットアップ画面を表示する
- セットアップ完了後、設定を保存し、SQLite と初期データを作成し、Axum サーバーを起動する
- 設定ファイルがある場合はその内容でサーバーを起動し、メイン画面を開く
- メインウィンドウ破棄時には Axum サーバーへシャットダウン信号を送る

### 3.2 サーバー単体モード

- `markdown_wiki2_single -s <ADDR>` で起動する
- `<ADDR>` はホストのみ（"0.0.0.0"）またはホスト:ポート（"0.0.0.0:9090"）形式を受け付ける
- ポートが省略された場合は `3080` を使用する
- 例: `-s 0.0.0.0` -> `0.0.0.0:3080`
- 例: `-s 0.0.0.0:9090` -> `0.0.0.0:9090`
- 事前に GUI 起動で設定ファイルを作成済みであることが前提
- Ctrl+C でグレースフルシャットダウンする

## 4. 初回セットアップ

初回セットアップ画面では次を入力する。

- アプリタイトル
- 管理者ユーザー名
- 管理者パスワード
- アカウントロック回数
- 待機制限開始回数
- 待機時間(分)
- アクセストークン有効期限(分)
- リフレッシュトークン有効期限(分)

セットアップ完了時に以下を自動生成する。

- 設定 JSON
- SQLite DB ファイル `~/.markdown-wiki2-single/markdown-wiki2.sqlite`
- 画像保存ディレクトリ `~/.markdown-wiki2-single/images`
- ランダムな `SECRET_KEY`

## 5. 設定と保存先

### 5.1 主な設定値

実装上、以下の環境変数相当を設定 JSON から起動時に注入する。

- `APP_TITLE`
- `DATABASE_URL`
- `CREATEDATABASE_PATH`
- `SECRET_KEY`
- `IMAGE_FILES_PATH`
- `UPLOAD_FILE_PATH`
- `FAILED_ACCOUNT_LOCK`
- `NEXT_CHALLENGE_MINUTES`
- `CHALLENGE_LIMIT_TIME_FAILEDCOUNT`
- `ADMIN_USERNAME`
- `ADMIN_PASSWORD`
- `ACCESS_TOKEN_EXP_MINUTUES`
- `REFRESH_TOKEN_EXP_MINUTUES`
- `CACHE_CONTROL`
- `SECURE_COOKIE`
- `SERVICE_NAME`
- `RUST_LOG`
- `ALLOW_USER_CREATE_ACCOUNT`
- `ALLOW_ORIGINS`

### 5.2 ファイル保存先

- 設定: `~/.markdown-wiki2-single/markdown-wiki2-single.env.json`
- DB: `~/.markdown-wiki2-single/markdown-wiki2.sqlite`
- アップロードファイル: `~/.markdown-wiki2-single/images/<先頭5文字>/<uuid_filename>`

## 6. データモデル

### 6.1 `user_model`

- ユーザー ID
- ログインユーザー名
- 表示名 `public_name`
- bcrypt ハッシュ済みパスワード
- 管理者フラグ `is_superuser`
- ログイン失敗回数
- 次回ログイン許可時刻
- ロック状態
- プライバシーモード
- TOTP 一時認証状態
- TOTP 本番シークレット / 仮シークレット

### 6.2 `wiki_model`

- Wiki ID
- 所有者ユーザー ID
- 作成日時相当 `date`
- タイトル
- 本文
- 作成日時
- 更新日時
- 公開フラグ `is_public`
- 編集申請中フラグ `is_edit_request`

### 6.3 `image_model`

- 画像 ID
- 所有者ユーザー ID
- 元ファイル名
- UUID ベース保存名
- 作成日時

### 6.4 `temporary_urls`

- 一時 URL ID
- 発行ユーザー ID
- 対象 Wiki ID
- URL パス
- 有効期限
- タイトル
- 本文のスナップショット
- 作成日時

### 6.5 `edit_request_wiki_model`

- 編集申請 ID
- Wiki オーナー ID
- 申請者 ID
- 対象 Wiki ID
- 申請タイトル
- 申請本文
- 申請メッセージ
- ステータス `REJECT | REQUESTNOW | DRAFT | APPLIED`
- 作成日時

### 6.6 `application_settings`

起動後に DB 上で参照されるログイン制限設定。

- `login_attempts_limit`
- `next_challenge_minutes`
- `challenge_limit_start`

## 7. 認証とセキュリティ

### 7.1 認証方式

- アクセストークンとリフレッシュトークンを JWT で発行する
- 両トークンは HttpOnly Cookie で返却する
- `SameSite=Strict`
- `Secure` は設定に依存する
- リフレッシュトークン Cookie の Path は `/account/refresh`

### 7.2 ミドルウェア

- `CookieValidator`: アクセストークン必須 API 用
- `RefreshCookieValidator`: リフレッシュトークン必須 API 用
- `FlexibleCookieValidator`: トークンがなくても匿名相当で通す静的画像配信用

### 7.3 ログイン制御

- 認証失敗回数は DB 設定に従ってカウントされる
- `challenge_limit_start` 回以上失敗すると一定時間再試行待ちになる
- `login_attempts_limit - 1` 到達時にアカウントはロックされる
- ログイン成功時は失敗回数をリセットする

### 7.4 TOTP

- パスワード認証成功後、TOTP 有効ユーザーは追加トークン入力を求められる
- 一次認証済み状態 `is_basic_authed` は 3 分以内のみ有効
- TOTP 有効化は QR コード提示 -> 6 桁コード検証 -> 本番シークレット昇格の流れ

### 7.5 ファイル配信制御

- 画像ファイルは所有者本人、または所有者がプライバシーモード OFF の場合のみ他者アクセス可
- 静的アセット配信では簡易ファイル名サニタイズを行う
- アップロード許可拡張子は `png/jpg/jpeg/gif/webp/pdf/mp4`
- 画像は再エンコードされ、EXIF 等を除去して保存される

## 8. 権限モデル

- 管理者: 初期作成される `is_superuser = true` ユーザー
- 一般ユーザー: 通常アカウント
- Wiki オーナー: 対象 Wiki の `user_id` と一致するユーザー

権限の基本ルール:

- 自分の Wiki は常に閲覧可
- 他人の Wiki は `is_public = true` の場合のみ閲覧可
- 更新/削除はオーナーのみ
- 公開 Wiki に対しては非オーナーが更新申請を送れる
- 一時 URL 発行はオーナーのみ
- 管理系 API は管理者のみ

## 9. 機能仕様

### 9.1 アカウント

- ログイン
- ログアウト相当: 期限 0 のトークンで Cookie を上書き
- サインアップ
  - `ALLOW_USER_CREATE_ACCOUNT` が true の場合のみ公開
  - 初回セットアップ既定値は false
- 自分の表示名/ID 取得
- プライバシーモード切替
- TOTP 有効化/無効化

### 9.2 Wiki 管理

- Wiki 作成
- Wiki 一覧取得
  - 自分の Wiki
  - 他人の公開 Wiki
  - 編集申請中 Wiki
- 件数制限付き一覧取得
- Wiki 単体取得
- Wiki 更新
- Wiki 削除
- タイトル/本文のキーワード検索
  - `query1` と `query2` の 2 語検索に対応
- Markdown ダウンロード
- オーナー情報取得

### 9.3 Markdown 表示/編集

フロントエンドは以下の表現をサポートする。

- 通常 Markdown
- コードハイライト
- Mermaid
- KaTeX
- 動画/YouTube/詳細表示/注意表示等の拡張トークン
- 画像/PDF/動画リンク埋め込み

補助機能:

- Ace Editor
- Vim キーバインド切替
- ツール表示切替
- プレビュー表示切替
- ページ内検索
- 目次表示
- 共有メモモーダル
- QR コード生成モーダル

### 9.4 タスクリストチェックボックス

プレビュー画面では Markdown タスクリストをインタラクティブなチェックボックスとして表示する。実装上、変更時は現在 Wiki への更新 API を呼ぶため、実際に保存できるのはオーナーである場合に限られる。

### 9.5 画像/PDF/動画管理

- アップロード
- 一覧取得
- クライアント側検索
- プレビュー
  - 画像
  - PDF
  - MP4
- 削除
- Markdown 埋め込み文字列の自動生成
  - 画像: `![alt](url)`
  - PDF: `[name](url)`
  - 動画: `?[name](url)` 形式

### 9.6 一時共有 URL

- オーナーは Wiki ごとに期限付き共有 URL を発行できる
- 入力単位は分、フロントエンドでは 10 分以上を要求する
- 同一 Wiki に既存 URL がある場合は新規発行前に削除される
- 一時 URL は本文のスナップショットを保持する
- 期限切れ URL はアクセス時に削除され、専用 Not Found ページを返す

### 9.7 更新申請

- 非オーナーは公開 Wiki に対して変更申請を送れる
- 申請対象 Wiki は `is_edit_request = true` に設定される
- 同一 Wiki に複数の申請は持てない
- 申請者は取り下げ可能
- オーナーは差分表示を確認して承認/却下できる
- 却下時:
  - 申請レコードは `REJECT`
  - Wiki の `is_edit_request` は false
- 承認時:
  - Wiki 本文とタイトルを申請内容で更新
  - `is_edit_request` は false
  - 申請レコードは削除される

### 9.8 管理者機能

- 管理画面 HTML 配信
- ユーザー一覧取得
- 一般ユーザー作成
- ユーザーパスワード再設定
- 公開名変更
- アカウントロック解除

## 10. 画面仕様

### 10.1 主画面一覧

- `/account/login`
- `/account/create`
- `/wiki/list`
- `/wiki/create`
- `/wiki/preview/:id`
- `/wiki/update/:id`
- `/wiki/delete/:id`

### 10.2 初回セットアップ画面

- Tauri カスタムプロトコル `app-setup://index` で配信
- サーバー未起動状態でも表示可能

### 10.3 モバイル向け表示

- `/index` は User-Agent に `Mobile` を含む場合 `index-mobile.html` を返す
- 一時共有プレビューも `preview-mobile.html` を使い分ける

## 11. API 概要

### 11.1 認証不要

- `GET /`
- `GET /index`
- `GET /health-check`
- `GET /app-init`
- `GET /favicon.ico`
- `GET /assets/{uri}`
- `POST /account/token`
- `POST /account/totp/token`
- `GET /onetime/{url_id}`
- `GET /licanses`
- `POST /account/signup` (`ALLOW_USER_CREATE_ACCOUNT=true` の場合のみ)

### 11.2 アクセストークン必須

- `/wiki/*`
- `/images/*`
- `/account/auth`
- `/account/info`
- `/account/privacy`
- `/account/totp/setup`
- `/account/totp/verify`
- `/account/totp/disable`
- `/account/token/disable`
- `/admin/*`
- `/onetimeurl/*`
- `/wiki-edit/*`

### 11.3 リフレッシュトークン必須

- `POST /account/refresh`

### 11.4 匿名許容

- `GET /static/images/{image_name}`
  - ただし実際の返却可否は所有者のプライバシーモードで判定

## 12. エラー応答

API エラーは原則 JSON で返る。

```json
{
  "error": "..."
}
```

主なステータス:

- `400 Bad Request`
- `401 Unauthorized`
- `404 Not Found`
- `409 Conflict`
- `415 Unsupported Media Type`
- `500 Internal Server Error`

## 13. 既知の実装上の挙動

- ユーザー自己登録は設定値で制御され、初回セットアップ既定では無効
- 画像プライバシーはファイル単位ではなくユーザー単位
- 一時 URL は Wiki の live データではなく、発行時点のタイトル/本文を保持する
- フロントエンドは JST 前提の表示補正を複数箇所で行う
- `SECURE_COOKIE=true` が既定のため、HTTP 運用時は設定変更が必要

## 14. 配布物

- Tauri バンドル対象: Windows NSIS
- 組み込み静的ファイル: `dist/`
- 組み込みテンプレート: `dist/templates/`

## 15. GitHub Release Workflow

本リポジトリには Windows 向けリリースビルド用の GitHub Actions workflow として `.github/workflows/release.yml` を持つ。

### 15.1 トリガー

- `v*` タグ push
- `workflow_dispatch`

### 15.2 ビルド時前提

- 実行環境は `windows-latest`
- ビルド前に workflow がルートへ CI 用 `.env` を生成する
- `.env` には少なくとも次の値を設定する
  - `DATABASE_URL`
  - `CREATEDATABASE_PATH`
  - `VITE_IP_ADDRESS`
  - `VITE_ASSET_PATH`
- `sqlx` のコンパイル時クエリ検証を通すため、workflow 内で `sqlx-cli` を導入し、`sqlx database create` と `sqlx migrate run` を実行して CI 用 SQLite DB を作成する

### 15.3 ビルド処理

- `npm ci` を `frontend`、`frontend-mobile`、`frontend-admin` で実行する
- `src_frontend/scripts/frontends-builder.ps1` でフロントエンド成果物を `dist/` に集約する
- `cargo tauri build` で Windows インストーラを生成する
- `target/release/bundle` 配下の `.exe` / `.msi` を成果物として収集する

### 15.4 リリース処理

- 収集済み成果物をまとめてダウンロードする
- SHA-256 チェックサム `checksums.txt` を生成する
- GitHub Release を draft で作成し、成果物とチェックサムを添付する

以上が、現行実装に基づく MarkdownWiki2-SingleBin の仕様である。
