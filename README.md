# tmux-copy-hop

Hop-style pane jumping for tmux copy workflows.

`tmux-copy-hop` lets you choose a visible character and then jump to one of its
matches using short labels. The main target workflow is selecting copy-mode
ranges without moving the cursor step by step.

行頭だけを選ぶモードも利用できます。`line-jump` は検索文字の入力を省略し、
表示中の各行の先頭にラベルを表示して、選んだ行の行頭へ移動します。

## MVP Behavior

```text
prefix+j
type one search character
choose a displayed label
```

From a normal pane, a valid label enters copy-mode and moves the copy-mode
cursor to the selected cell. It does not start a selection.

From copy-mode, a valid label keeps copy-mode active and moves only the cursor.
Any existing selection is preserved.

Cancel with `Escape` or `Ctrl-C`.

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
```

Put `target/release/tmux-copy-hop` on `PATH`, then add this binding:

```tmux
bind-key j run-shell "tmux-copy-hop jump"

# 行頭ジャンプ（任意の追加マッピング）
bind-key l run-shell "tmux-copy-hop line-jump"
```

This repository also includes a minimal TPM-style entry file:

```tmux
run-shell /path/to/tmux-copy-hop/tmux-copy-hop.tmux
```

The MVP does not automatically build the Rust binary during plugin install.

### Install Options

Install to a custom directory:

```bash
INSTALL_DIR=/usr/local/bin sh install.sh
```

Install a specific version:

```bash
VERSION=v0.1.0 sh install.sh
```

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

### 行頭ジャンプ

既存のマッピングは変更せず、別のキーに追加できます。

```tmux
bind-key j run-shell "tmux-copy-hop jump"
bind-key l run-shell "tmux-copy-hop line-jump"
```

`prefix+l` を押し、行頭に表示されたラベルを入力すると、その行の先頭へ
移動します。通常ペインから実行した場合は copy-mode に入ります。

## Current Limitations

- Only the currently visible pane content is targeted.
- Search is one character and case-sensitive.
- ASCII search is the primary target.
- Popup rendering is plain text with highlighted labels.
- The popup is centered with the target pane's width and height.
- Movement uses copy-mode commands from the visible top line.
- Full Unicode, mouse support, fuzzy search, OSC52, and true overlay rendering
  are out of scope for the MVP.

## Development

Run tests:

```bash
cargo test
```

When testing against a dedicated tmux socket, set:

```bash
TMUX_COPY_HOP_SOCKET=/tmp/tmux-copy-hop-test.sock tmux-copy-hop jump
```
