# MarkdownWiki2-SingleBin Release Note

## Version 1.0.15

- アプリケーション起動時に `webbrowser` クレートを使用してOSのデフォルトブラウザで自動的に開く処理を追加
    - サーバーとしてのみ起動したい場合は `-s` オプションを指定することでブラウザは起動しない。

## Version 1.0.14

- 'VITE_' で始まっていた環境変数を修正
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
  .katex-display > .katex {
    text-align: left;
    margin-left: 5%;
  }
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