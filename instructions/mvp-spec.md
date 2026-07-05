# tmux-copy-hop MVP仕様

## 目的

`tmux-copy-hop` は、tmux pane上の表示内容に対して Hop.nvim 風の2段階ジャンプを行うプラグインである。

主用途は copy-mode でのコピー範囲指定だが、設計上は「tmux画面ジャンパー」を土台にする。

```text
prefix+j
検索文字を1文字入力
該当文字位置にラベル表示
ラベル入力
その位置へジャンプ
```

copy-modeのコピー範囲指定では、以下の操作感を目指す。

```text
prefix+j
検索文字入力
ラベル入力
copy-modeに入り、開始位置へカーソル移動
Space または v で選択開始
prefix+j
検索文字入力
ラベル入力
選択状態を維持したまま終了位置へカーソル移動
Enterでコピー
```

## コマンド構成

MVPでは Rust 製CLIを作る。

```text
tmux-copy-hop jump
```

tmux binding例:

```tmux
bind-key j run-shell "tmux-copy-hop jump"
```

Rust CLIが以下を直接実行する。

```text
tmux display-message -p ...
tmux capture-pane ...
tmux display-popup ...
tmux send-keys -X ...
```

shell wrapperは、インストールやtmux plugin manager対応などの薄い用途に留める。

## 状態遷移

### 通常paneから実行した場合

通常paneで `prefix+j` を押した場合、ラベルが有効に確定するまでは copy-mode に入らない。

```text
prefix+j
検索文字入力
capture
popup表示
ラベル入力

有効ラベル:
  copy-modeへ入る
  選択した文字セルへカーソル移動

キャンセル/候補0/無効ラベル:
  copy-modeへ入らない
  カーソル移動しない
```

copy-modeへ入った後も、プラグインは自動で選択開始しない。選択開始はユーザーが `Space` または `v` など既存のcopy-mode操作で行う。

### copy-mode中に実行した場合

copy-mode中の `prefix+j` は、copy-modeを維持したままカーソルだけ移動する。

```text
prefix+j
検索文字入力
capture
popup表示
ラベル入力

有効ラベル:
  copy-modeのままカーソル移動

キャンセル/候補0/無効ラベル:
  copy-modeのまま
  カーソル移動しない
```

既に選択範囲がある場合、プラグインは選択状態を必ず維持する。`begin-selection`、`clear-selection`、`copy-selection` は勝手に送らない。

## 入力仕様

### 検索文字

検索文字はMVPでは1文字固定にする。

```text
prefix+j
検索文字を1文字読む
```

一致判定は大文字小文字を区別する。

```text
a は a のみに一致する
A は A のみに一致する
```

### ラベル入力

検索文字に一致した候補へラベルを付け、ユーザーはラベルを入力してジャンプ先を決める。

ラベル文字集合は以下とする。

```text
asdfghjklqwertyuiopzxcvbnm
```

候補数に応じて、全ラベルを同じ長さに揃える。

```text
候補数 <= 26:
  a, s, d, f ...

候補数 >= 27:
  aa, as, ad, af ...
```

可変長ラベルはMVPでは使わない。

### キャンセル

検索文字入力中、ラベル入力中のキャンセルキーは以下とする。

```text
Escape
Ctrl-C
```

キャンセル時は以下を保証する。

```text
popupを閉じる
copy-modeに入る前なら通常paneのまま
copy-mode中ならcopy-modeのまま
選択状態は変更しない
カーソル移動もしない
```

## 候補抽出

### 対象画面

対象は現在見えているpane内容のみとする。画面外スクロールバック全体はMVPでは対象外。

通常paneでは、現在paneに表示されている内容をcaptureする。

copy-mode中では、copy-modeで現在見えている内容を対象にする。スクロールバック内を見ている場合も、ユーザーが現在見ているcopy-mode表示内容から候補を抽出する。

### 候補位置

検索文字と一致した文字セルそのものを候補位置にする。

```text
表示行:   abcdef
検索文字: d
ジャンプ: d のセル
```

MVPでは、候補位置の直前・直後へジャンプするモードは持たない。

### 文字幅

MVPではASCII検索を主対象にする。

ただし、内部のセル幅計算は `unicode-width` を使う前提にする。全角文字が混ざる行でも、ASCII候補のx座標が大きくずれないようにする。

絵文字、結合文字、日本語・全角文字の完全対応はMVPでは非対応またはbest effortとする。

## ラベル割り当て

### 並び順

一致候補は、現在カーソル位置に近い順に並べる。

通常paneでの基準点は tmux pane cursor位置とする。

```text
#{cursor_x}
#{cursor_y}
```

copy-mode中の基準点は copy-mode cursor位置とする。取得が難しい場合は調査対象とし、最終手段として pane cursor位置へフォールバックする。

距離計算の詳細は実装時に決めるが、MVPではマンハッタン距離を第一候補にする。

```text
distance = abs(candidate_x - cursor_x) + abs(candidate_y - cursor_y)
```

同距離の場合は、上から下、左から右の順で安定ソートする。

### ラベル衝突

2文字以上ラベルでは表示範囲が他のラベルと重なる可能性がある。

MVPでは、近い候補を優先し、後続の衝突候補は非表示・選択不可にする。

```text
1. 一致候補を現在カーソルに近い順に並べる
2. 候補総数から必要ラベル長を決める
3. その長さでラベルを割り当てる
4. ラベル表示セルが既存ラベルと重なる候補は落とす
5. 表示された候補だけ選択可能にする
```

## popup表示

MVPでは本物のoverlayではなく、popup内にラベル付き画面を再描画する。

```text
display-popup -w <pane_width> -h <pane_height>
```

popupは対象paneと同じサイズで中央に表示する。pane位置へ厳密に重ねる疑似overlayは後回しにする。

### 表示内容

元pane内容はプレーンテキストで表示する。ANSI属性や元paneの色・装飾はMVPでは再現しない。

ラベルだけANSI色で強調する。

### ラベル描画

候補位置の一致文字をラベルで置換する。

```text
候補文字が1文字ラベル:
  そのセルをラベル文字で置換

候補文字が2文字ラベル:
  そのセルと右隣セルをラベルで上書き
```

ラベルが右端からはみ出す場合は、左方向にずらして表示する。

この場合もジャンプ先座標は元の候補位置のままとする。ラベルの最後の文字が候補セルに乗る。

```text
候補x = pane_width - 1
label = "as"

表示:
  x-1 に "a"
  x   に "s"

ジャンプ先:
  元の候補x
```

## 移動実装

ラベルで選んだ候補位置へcopy-modeカーソルを移動する。

優先順位は以下とする。

```text
第一候補:
  copy-modeの座標指定移動が可能なら使う

第二候補:
  現在copy-mode cursorから dy/dx を計算
  send-keys -X cursor-up/down/left/right を必要回数送る

第三候補:
  行移動 + 行内検索などの組み合わせ
```

通常paneからの有効ジャンプ時は、ラベル確定後にcopy-modeへ入り、その後で選択候補位置へ移動する。

copy-mode中の有効ジャンプ時は、copy-modeから出ずに移動する。

## エラー・失敗時の挙動

### 候補0件

検索文字に一致する候補がない場合は、短いメッセージを出して終了する。

```text
tmux-copy-hop: no matches for 'x'
```

このとき以下を保証する。

```text
copy-modeに入る前なら入らない
copy-mode中ならそのまま
カーソル移動なし
選択状態変更なし
```

### 無効ラベル

存在しないラベルを入力した場合は、短いメッセージを出して終了する。

```text
tmux-copy-hop: invalid label
```

このとき以下を保証する。

```text
copy-modeに入る前なら入らない
copy-mode中ならそのまま
カーソル移動なし
選択状態変更なし
```

## 非要件

MVPでは以下を扱わない。

```text
画面外スクロールバック全体へのジャンプ
マウス対応
日本語・全角文字の完全対応
絵文字・結合文字の完全対応
fuzzy search
2文字検索
複数ペイン横断
OSC52コピー連携
tmux popup内コピー対応
元paneのANSI属性・色の完全再現
pane位置に完全一致する本物overlay
自動選択開始
```

## 配布方針

MVPは開発者向けに、ソースからビルドして使う形でよい。

```bash
cargo build --release
```

`target/release/tmux-copy-hop` を `PATH` に置いて使う。

TPM install時の自動ビルド、OS/arch別バイナリ配布、PATH設定補助は後回しにする。

## MVP完了条件

MVPは以下を満たしたら完了とする。

```text
現在見えているpane内容を取得できる
検索文字1文字に一致するセルを候補化できる
候補へ近い順でラベルを割り当てられる
popup内にラベル付き画面を再描画できる
有効ラベル入力で該当セルへジャンプできる
通常paneでは有効ラベル確定後だけcopy-modeへ入る
copy-mode中は選択状態を壊さずカーソルだけ移動できる
候補0件・無効ラベル・キャンセル時に副作用なしで終了できる
```
