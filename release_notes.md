# MarkdownWiki2-SingleBin Release Note

## Version 1.0.1

### バグを修正

- TOTP によるログインが正常に動作しないエラー: TOTP検証時に経過時間が正しく検査されていなかった（文字列からNaiveDateTimeへのパースエラー）問題を修正

## Version 1.0.0

MarkdownWiki2をシングルバイナリで配布できるようにしたプロジェクト。小規模利用の際に、DockerあるいはPostgreSQLなどの環境構築不要で起動できる。データベースとしてSQLiteを使用。