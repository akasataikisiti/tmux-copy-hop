## 目的

**tmux copy-mode内で、Hop.nvim / Flash.nvim のように「開始位置」と「終了位置」をラベルジャンプで指定してコピーできる軽量プラグイン**を作る。

tmux の copy-mode は `send-keys -X begin-selection` や `copy-selection` などの copy-mode コマンドで操作できます。また `pane_mode` を見て copy-mode 中か判定することもできます。([Tmux][1])

## MVP要件

### 1. 通常モードから開始位置へジャンプ

```text
prefix + j
↓
画面内の候補にラベル表示
↓
ラベル入力
↓
その位置で copy-mode に入る
```

### 2. copy-mode中に終了位置へジャンプ

```text
v または Space で選択開始
↓
prefix + j または専用キー
↓
候補ラベル表示
↓
ラベル入力
↓
カーソルが終了位置へ移動
↓
Enter でコピー
```

これが今回の一番重要な機能です。

### 3. 行ジャンプ

```text
prefix + J
↓
各行にラベル表示
↓
ラベル入力
↓
その行へ移動
```

最初は「行頭へジャンプ」で十分です。

## 非要件・後回し

最初からやらなくていいもの：

* 画面外スクロールバック全体へのジャンプ
* マウス対応
* 日本語・全角文字の完全対応
* fuzzy search
* 複数ペイン横断
* OSC52コピー連携
* tmux popup内コピー対応

まずは **現在見えているpane内だけ** で十分です。

## 推奨アーキテクチャ

構成はこれが軽いです。

```text
tmux binding
  ↓
shell script
  ↓
capture-pane で画面取得
  ↓
候補位置を計算
  ↓
ラベル付きoverlayを表示
  ↓
ユーザー入力
  ↓
send-keys -X でcopy-modeカーソル移動
```

tmux は `capture-pane` でpane内容を取得でき、copy-mode操作は `send-keys -X` 系で行えます。copy-modeの基本機能として選択開始・コピー・検索・移動があります。([Tmux][1])

## 言語選定

### 第一候補: Rust

一番おすすめです。

理由：

* 起動が速い
* 単体バイナリで配布しやすい
* 文字幅計算をちゃんと扱いやすい
* tmuxプラグインとしても重くなりにくい
* 将来的に日本語・絵文字・全角対応しやすい

使うクレート候補：

```text
unicode-width
clap
anyhow
```

MVPならこれくらいで十分です。

### 第二候補: Go

Rustより書きやすさ重視ならGoも良いです。

理由：

* 単体バイナリ
* 起動が速い
* 配布が楽
* 実装がシンプル

ただし、ターミナル上の文字幅処理はRustの方がやりやすい印象です。

### 第三候補: POSIX shell + awk

最軽量に見えますが、今回の用途ではおすすめ度は低めです。

理由：

* ラベル計算が複雑になる
* 全角文字に弱い
* 保守がつらい
* 高速化しづらい

ただし、**プロトタイプ** なら shell で十分です。

## 私ならこう作ります

### 実装方針

```text
tmux-hop-copy
```

というRust製CLIにする。

コマンド例：

```bash
tmux-hop-copy start
tmux-hop-copy end
tmux-hop-copy line
```

### tmux.conf

```tmux
bind-key j run-shell "tmux-hop-copy start"
bind-key -T copy-mode-vi j send-keys -X begin-selection \; run-shell "tmux-hop-copy end"
bind-key J run-shell "tmux-hop-copy line"
```

最終的にはこんな操作感を目指します。

```text
prefix + j   開始位置へジャンプしてcopy-mode
v            選択開始
j            終了位置へジャンプ
Enter        コピー
```

## 技術的な難所

一番難しいのは **「overlay表示」** です。

候補ラベルを画面上に一時的に表示する方法としては、候補があります。

1. `display-popup` を使う
2. 画面全体をANSIエスケープで一時描画する
3. tmux paneに一時的にラベル付き画面を描いて戻す
4. 既存の `tmux-jump` の方式を参考にする

一番現実的なのは、まず `display-popup` または一時pane描画でMVPを作り、あとで本物のoverlayに近づけることです。

## MVPの完成条件

最初のバージョンはこれで完成扱いにしてよいです。

```text
✅ 現在表示中のpaneを取得できる
✅ 単語先頭にラベルを振れる
✅ ラベル入力でcopy-modeのカーソルを移動できる
✅ copy-mode中でも再ジャンプできる
✅ 選択中の範囲を壊さない
✅ 行頭ジャンプできる
```

## プロジェクト名案

```text
tmux-flash-copy
tmux-hop-copy
tmux-copy-hop
tmux-leap-copy
tmux-jump-select
```

個人的には **`tmux-copy-hop`** が分かりやすいです。

[1]: https://tmux-tmux.mintlify.app/guides/copy-mode?utm_source=chatgpt.com "Copy Mode - tmux"

