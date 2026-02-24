# チェックボックス関連機能

### 1) 表示（レンダリング）フロー

- `renderMarkdown(markdown)` が呼ばれる
- `headingIndex = -1` にリセット
- `taskOffsets = buildTaskOffsets(markdown)` を作る
  - `codeRegex` で fenced code block（```）の範囲を収集
  - `taskRegex` で task 行（`- [ ] ` / `- [x] `）を検出
  - `[` の絶対位置を `start` として `offsets.push({start, end:start+3})`
  - `start` が code block 範囲内なら除外

- `taskOffsetIdx = 0` にリセット
- `marked.parse(markdown)` を実行
  - marked が checkbox をレンダリングするタイミングで `renderer.checkbox` が呼ばれる
  - `renderer.checkbox` は `taskOffsets[taskOffsetIdx]` を読み、該当があれば
    - `data-start`, `data-end` を付与
    - `disabled` を付けずインタラクティブに出力
    - `taskOffsetIdx` を進める（※修正推奨：offsetがある時だけ）
  - 該当が無ければ disabled checkbox を出力

- HTML を `myXss.process` でサニタイズ
- `renderIframe` して表示する

---

### 2) チェック操作フロー

- ユーザーが checkbox を変更（change）
- `onCheckboxChange(event)` が起動
- `event.target` が checkbox input であることを確認
- `data-start/end` を取得
- start/end の範囲を検証
- `currentMarkdown` を `before / targetStr / after` に分割
- `targetStr` が `"[ ]"` または `"[x]"` であることを確認
- `target.checked` に応じて `newCheckbox` を決定
- `currentMarkdown = before + newCheckbox + after` に更新
- `bindHtml = renderMarkdown(newMarkdown)` で再描画（新しい data-start/end が再付与される）
- `nextTick()` の後に `Prism.highlightAll()`
- `patchMarkdownBody(newMarkdown)` で backend に PUT
  - body を title 分 slice して送信
  - store 更新
