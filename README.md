# tmux-copy-hop

Hop-style pane jumping for tmux copy workflows.

`tmux-copy-hop` lets you choose a visible character and then jump to one of its
matches using short labels. The main target workflow is selecting copy-mode
ranges without moving the cursor step by step.

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

```text
prefix+j
type the start character
type the start label
Space or v
prefix+j
type the end character
type the end label
Enter
```

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
