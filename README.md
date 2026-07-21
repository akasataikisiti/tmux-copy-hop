# tmux-copy-hop

Hop-style pane jumping for tmux copy workflows.

`tmux-copy-hop` lets you choose a visible character and then jump to one of its
matches using short labels. The main target workflow is selecting copy-mode
ranges without moving the cursor step by step.

行頭だけを選ぶモードも利用できます。`line-jump` は検索文字の入力を省略し、
表示中の各行の先頭にラベルを表示して、選んだ行の行頭へ移動します。

## Usage

### Character jump

```text
prefix+j
type one search character
choose a displayed label
```

From a normal pane, a valid label enters copy-mode and moves the copy-mode
cursor to the selected cell. It does not start a selection.

From copy-mode, a valid label keeps copy-mode active and moves only the cursor.
Any existing selection is preserved.

Cancel either mode with `Escape` or `Ctrl-C`.

### Line-head jump

`line-jump` labels the beginning of every visible line. It skips the character
search step and moves to the beginning of the line selected by the label.

```text
prefix+l
choose a displayed line label
```

The line-head mode uses the same copy-mode behavior as character jump: it enters
copy-mode when started from a normal pane and preserves an existing selection
when started from copy-mode.

## Install

Install the latest Linux x86_64 release:

```bash
curl -fsSL https://raw.githubusercontent.com/akasataikisiti/tmux-copy-hop/main/install.sh | sh
```

Then add this binding:

```tmux
bind-key j run-shell "tmux-copy-hop jump"

# 行頭ジャンプ（任意の追加マッピング）
bind-key l run-shell "tmux-copy-hop line-jump"
```

Make sure `~/.local/bin` is on `PATH` inside tmux.

### Build From Source

Build the binary:

```bash
cargo build --release
mkdir -p ~/.local/bin
cp target/release/tmux-copy-hop ~/.local/bin/tmux-copy-hop
```

Then add these bindings:

```tmux
bind-key j run-shell "tmux-copy-hop jump"

# 行頭ジャンプ（任意の追加マッピング）
bind-key l run-shell "tmux-copy-hop line-jump"
```

This repository also includes a minimal TPM-style entry file:

```tmux
run-shell /path/to/tmux-copy-hop/tmux-copy-hop.tmux
```

After changing the binary or tmux configuration, reload the configuration:

```bash
tmux source-file ~/.tmux.conf
```

The TPM-style entry file only installs the key bindings; it does not build the
Rust binary automatically.

### Install Options

Install to a custom directory:

```bash
INSTALL_DIR=/usr/local/bin sh install.sh
```

Install a specific version:

```bash
VERSION=v0.1.0 sh install.sh
```

### Updating an Existing Installation

リリース版をすでに導入している場合は、インストールスクリプトをもう一度
実行すると最新版に更新できます。

```bash
curl -fsSL https://raw.githubusercontent.com/akasataikisiti/tmux-copy-hop/main/install.sh | sh
```

`INSTALL_DIR` を指定して導入した場合は、更新時にも同じ値を指定します。

```bash
INSTALL_DIR=/path/to/bin sh install.sh
```

既存の `jump` 設定はそのまま使えます。行頭ジャンプも使う場合は、次の設定を
追加してから tmux 設定を再読み込みします。

```tmux
bind-key l run-shell "tmux-copy-hop line-jump"
```

```bash
tmux source-file ~/.tmux.conf
```

ソースから導入している場合は、リポジトリを更新してバイナリを再ビルドします。

```bash
git pull
cargo build --release
cp target/release/tmux-copy-hop ~/.local/bin/tmux-copy-hop
```

なお、インストールスクリプトで取得できるのは公開済みリリースです。新機能が
リリースに含まれていない場合は、公開後に更新してください。

## Copy Workflow

Use `tmux-copy-hop` to place the copy-mode cursor at the start and end of the
range you want to copy. The plugin only moves the cursor; selection start and
copy are still normal tmux copy-mode actions.

### 1. Jump to the start position

Press your binding from a normal pane:

```text
prefix+j
```

Then type one visible character near the place where you want the copy range to
start. `tmux-copy-hop` shows labels on every matching character. Type the label
for the exact start position.

After a valid label, tmux enters copy-mode and moves the cursor to that cell.

### 2. Start the selection

Use your usual tmux copy-mode key to begin selection:

```text
Space
```

or, if your copy-mode bindings use vi-style selection:

```text
v
```

### 3. Jump to the end position

While the selection is active, press the binding again:

```text
prefix+j
```

Type one visible character near the end of the range, then type the label for
the exact end position. The existing selection is preserved while the cursor
moves.

### 4. Copy

Use the normal tmux copy key:

```text
Enter
```

Full example:

```text
prefix+j        # choose the start cell
Space or v      # begin selection
prefix+j        # choose the end cell
Enter           # copy selection
```

### 行頭ジャンプの操作

既存のマッピングは変更せず、別のキーに追加できます。

```tmux
bind-key j run-shell "tmux-copy-hop jump"
bind-key l run-shell "tmux-copy-hop line-jump"
```

`prefix+l` を押し、行頭に表示されたラベルを入力すると、その行の先頭へ
移動します。通常ペインから実行した場合は copy-mode に入ります。

## Limitations

- Only the currently visible pane content is targeted.
- Character jump searches one character and is case-sensitive.
- Line-head jump labels only the currently visible lines.
- ASCII search is the primary target.
- Popup rendering is plain text with highlighted labels.
- The popup is centered with the target pane's width and height.
- Movement uses copy-mode commands from the visible top line.
- Full Unicode, mouse support, fuzzy search, OSC52, and true overlay rendering
  are currently out of scope.

## Development

Run tests:

```bash
cargo test
```

When testing against a dedicated tmux socket, set:

```bash
TMUX_COPY_HOP_SOCKET=/tmp/tmux-copy-hop-test.sock tmux-copy-hop jump
```
