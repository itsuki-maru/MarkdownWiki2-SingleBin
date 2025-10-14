# MarkdownWiki2-SingleBin Release Note

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