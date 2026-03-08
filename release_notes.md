# MarkdownWiki2-SingleBin Release Note

## Version 1.4.2

- codeタグのスタイリングを修正

## Version 1.4.1

- 目次のスタイルを修正

## Version 1.4.0

### 各種JSファイルのnpm管理への移行

**直接保存していた各種JavaScriptファイルをnpmに完全移行**

`scripts/frontends-builder.sh`（または `.ps1`）を実行すると、以下の順で処理される。

```
frontend/        npm run build  ─┐
frontend-mobile/ npm run build  ─┼─→ main/dist/ に集約 ─→ プロジェクトルート dist/ にコピー
frontend-admin/  npm run build  ─┘
```

最終的にプロジェクトルートの `dist/assets/` に全静的ファイルが集まり、Rust の `rust-embed` によってバイナリに埋め込まれる。

**src/templates/ との関係**

`src/templates/` 配下の HTML ファイル（Rust バックエンドのサーバーサイドテンプレート）は、
`/assets/` パス経由で以下のライブラリを参照している。

| テンプレート          | 参照する主なアセット                                           |
| --------------------- | -------------------------------------------------------------- |
| `preview.html`        | marked, xss, mermaid, prismjs（コア＋言語）, katex, github.css |
| `preview-mobile.html` | 同上                                                           |
| `notfound.html`       | github.css                                                     |

- これらのファイルは Vite の `public/` で管理していたが、**npm に移行済み**。
- `frontend/package.json` の dependencies として宣言し、
- `frontend/scripts/copy-preview-assets.js` が `npm run build` 後に `node_modules/` から `dist/assets/` へコピーする。

### その他の変更

- 目次UIを更新
- フロントエンドをリファクタリング
- CDN依存であった pdfjs の .bmapsをローカルに移動
- チェックボックスを独立して使用できるようにアップデート
  - チェックボックスの入力支援ボタンを追加
- frontend のビルドファイルのチャンクサイズを縮小

## Version 1.3.4

- フロントエンドをリファクタリング
- CDN依存であった pdfjs の .bmapsをローカルに移動
- チェックボックスを独立して使用できるようにアップデート
  - チェックボックスの入力支援ボタンを追加
- frontend のビルドファイルのチャンクサイズを縮小

## Version 1.3.3

- バックエンドを大規模リファクタリング
  - 関連ファイルのサブディレクトリを作成（db, models, image, middleware）
  - 構造体を scheme.rs から分離分割

## Version 1.3.2

- コードフォーマッタによりプロジェクト全体をフォーマッティング
- フロントエンドの依存関係を更新

## Version 1.3.1

- Rust バックエンドをリファクタリング
- 共有中のWiki一覧のUI崩れを修正
- フロントエンドの依存関係を更新

## Version 1.3.0

- モバイルからも変更リクエストを送信できるようにアップデート
- 変更リクエスト画面であることを認知させるための遷移時メッセージ表示処理を追加
- 非推奨のHTMLテーブル記述を修正
- Wiki更新リクエストにおいて、承認者が却下直後に、承認し、申請者のUI上に「取り下げ」が存在する場合の申請取り下げリクエストを行った場合のエラーハンドリングを定義
- 画像一覧モーダル、Wiki変更申請モーダル、共有Wiki一覧モーダルにおいて、データがない場合のUIを自動調整
- Wiki更新リクエスト時に申請者からメッセージを送信できる機能を追加
  - XSS 対策実装済み

## Version 1.2.0

### バックエンド

- 各ハンドラー関数のエラーレスポンスを AppError に統一
  - match文などで行ってきた冗長的なエラーハンドリングを排除
- ユーザー作成時、JSONファイルのインポート時にトランザクションを使用
- 開発時には localhost:5173 でブラウザを開くように修正

### フロントエンド

- Pinia のバージョンを 2系 から 3系 に更新

## Version 1.1.16

- vue-router よるルーティング時（画面遷移）に無駄に発生させていたサーバとの通信を削除
- アプリケーション初期読み込み時に画像一覧を取得する仕様に変更
- Wiki作成画面にて入力中の内容はローカルストレージに保存し、誤ったタブやブラウザのクローズ後も復元できるように仕様を変更

## Version 1.1.15

- モバイルの数式作成機能を削除
- マークダウン入力ボタンの表示非表示切替機能を追加

## Version 1.1.14

- 既存のアクセストークンとリフレッシュトークンを期限0の無効トークンで上書きするAPIを追加
- `frontends-builder.ps1` PowerShell スクリプトでビルド時は Rust もビルドするように修正

## Version 1.1.13

- ライセンスをフリーウェアに変更

## Version 1.1.12

- 共有URLのエラーハンドリングを強化
  - UUID のパースエラー時の処理を適切にハンドリング

## Version 1.1.11

- 起動済みサーバの検証に使用するロジックを `TcpStream::connect` から `TcpListener::bind` に変更
  - connect はハンドシェイクを行うため起動確認に時間がかかる。
- サーバの起動を `localhost` から `127.0.0.1` に変更
  - localhost は言わばホスト名であり、IPは v4 か v6 か動的に使われる（2重起動ができてしまう）。
- Wikiのヘッダのスタイルを変更

## Version 1.1.10

- フロントエンドのビルドスクリプト（frontend-builder.sh）を修正

## Version 1.1.9

- 初回起動時にコンソールが出力されない問題を修正
  - セットアップのプロンプト入力のタイミングでコンソールを表示し、その後、非表示
- フロントエンドの依存関係を更新

## Version 1.1.8

- Windows環境下ではコンソールを非表示に変更
  - コマンド起動の場合は `-c` オプションにより、表示可能
  - 初回起動以降、バックグラウンドでサーバプロセスが起動し続け、2回目以降はブラウザのみ起動
- Dockerfile 内の Node.js の取得を 20.X から 22.X に変更

## Version 1.1.7

- 認可トークン（Cookie）に Secure 属性の設定オプションを追加。
  - HTTP環境下での運用の場合、`markdown-wiki2-single.env.json` 設定ファイル内の `secure_cookie` を `false` に変更する。

## Version 1.1.6

- 開発時のみオリジンの許可が必要であった `http://localhost:5173` をリリースビルド時に除外

## Version 1.1.5

- パブリックのWikiがオーナー以外からダウンロードできない問題を修正

## Version 1.1.4

- フロントエンドの依存関係を更新
  - 将来的に Pinia を3系に更新

## Version 1.1.3

- YouTubeの埋め込みエラーを修正
  - iframe の置換内容を YouTube のものと統一

## Version 1.1.2

- 未使用コードを除去

## Version 1.1.1

- 管理者画面からのユーザー作成エラーを修正。
  - Version 1.1.0 での実装ミスを修正
- .sqlx ファイルを更新

## Version 1.1.0

### 公開ユーザー名の実装

- `user_model` テーブルに `public_name` を追加。これにより、Wikiのオーナー名を任意の名称設定（2から10文字以内）が可能となった。
- これまでのユーザー名からアカウントをロックされる攻撃の可能性を軽減。
- 初回設定以降の変更は管理者アカウントからのみ管理者画面から変更が可能。これはアプリケーションの性質上、頻繁な変更を避けるため。
- 公開ユーザー名は他のユーザーと重複可（ユーザーの自由度を優先）

### 1.17.X からの変更手順

```sql
ALTER TABLE user_model ADD COLUMN public_name CHARACTER VARYING(256);
UPDATE user_model SET public_name = username;
ALTER TABLE user_model ALTER COLUMN public_name SET NOT NULL;
```

## Version 1.0.16

- 改行コードを LF に統一

## Version 1.0.15

- アプリケーション起動時に `webbrowser` クレートを使用してOSのデフォルトブラウザで自動的に開く処理を追加
  - サーバーとしてのみ起動したい場合は `-s` オプションを指定することでブラウザは起動しない。

## Version 1.0.14

- 'VITE\_' で始まっていた環境変数を修正
- 過去の更新で不要となった 'VITE_APP_TITLE' 環境変数を除去

## Version 1.0.13

- 未使用の変数や構造体を削除
- 変数名を変更

## Version 1.0.12

- アプリケーションタイトルやアカウント作成許可設定、オリジン設定情報等を `/app-init` で取得可能とした。
  - 不要となったタイトル取得のURLを削除

## Version 1.0.11

- ビルドスクリプトを更新
- `frontend/public/templates` から `src/templates` へテンプレートファイルを移動
- licenses.html の返却が過去の設定のままであったため修正

## Version 1.0.10

- アイコンの各種サイズを作成
- icon.ico を更新

## Version 1.0.9

[embed-resource](https://github.com/nabijaczleweli/rust-embed-resource) クレートを使用してバイナリにアイコンを設定。

## Version 1.0.8

- アプリケーションアイコンを一新。

## Version 1.0.7

- 一時共有WikiのKaTexとYouTube埋め込みに対応

## Version 1.0.6

可読性を向上させるため、Katex による数式の配置をセンターから左寄せに変更。

**index.html**

```html
.katex-display > .katex { text-align: left; margin-left: 5%; }
```

モバイルUIでKatex数式の背景が馴染まない問題を修正。

## Version 1.0.5

- サーバ起動時のメッセージを修正（httpプロトコルを追加）
  - `========== Listening on localhost:3080 ==========` => `========== Listening on http://localhost:3080 ==========`

## Version 1.0.4

- HTTPプロトコル時に共有Wikiの一覧テーブルの表示が乱れていた問題を修正。

## Version 1.0.3

### バグ修正

- 共有URLにおいて画像リソースなどがサーバーエラーとなる問題を修正
  - Cookie検証において、アカウント不要でも通す使用であったが、ダミーのIDがUuid型のままであったため、Stringに修正。

## Version 1.0.2

### バグ修正

- ローカルネットワーク向けの起動時（`-h 192.168.0.1`など）にフロントエンド側にオリジン伝達するための環境変数定義方法を変更。

## Version 1.0.1

### バグを修正

- TOTP によるログインが正常に動作しないエラー: TOTP検証時に経過時間が正しく検査されていなかった（文字列からNaiveDateTimeへのパースエラー）問題を修正

## Version 1.0.0

MarkdownWiki2をシングルバイナリで配布できるようにしたプロジェクト。小規模利用の際に、DockerあるいはPostgreSQLなどの環境構築不要で起動できる。データベースとしてSQLiteを使用。
