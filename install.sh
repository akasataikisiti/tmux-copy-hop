#!/usr/bin/env sh
set -eu

repo="akasataikisiti/tmux-copy-hop"
install_dir="${INSTALL_DIR:-"$HOME/.local/bin"}"
version="${VERSION:-latest}"

case "$(uname -s)" in
  Linux) os="linux" ;;
  *)
    echo "tmux-copy-hop: unsupported OS: $(uname -s)" >&2
    exit 1
    ;;
esac

case "$(uname -m)" in
  x86_64 | amd64) arch="x86_64" ;;
  *)
    echo "tmux-copy-hop: unsupported architecture: $(uname -m)" >&2
    exit 1
    ;;
esac

if ! command -v curl >/dev/null 2>&1; then
  echo "tmux-copy-hop: curl is required" >&2
  exit 1
fi

if ! command -v tar >/dev/null 2>&1; then
  echo "tmux-copy-hop: tar is required" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT INT TERM

if [ "$version" = "latest" ]; then
  asset_url="https://github.com/${repo}/releases/latest/download/tmux-copy-hop-${os}-${arch}.tar.gz"
else
  asset_url="https://github.com/${repo}/releases/download/${version}/tmux-copy-hop-${os}-${arch}.tar.gz"
fi

archive="$tmp_dir/tmux-copy-hop.tar.gz"
curl -fsSL "$asset_url" -o "$archive"
tar -xzf "$archive" -C "$tmp_dir"

binary="$(find "$tmp_dir" -type f -name tmux-copy-hop | head -n 1)"
if [ -z "$binary" ]; then
  echo "tmux-copy-hop: release archive did not contain tmux-copy-hop" >&2
  exit 1
fi

mkdir -p "$install_dir"
cp "$binary" "$install_dir/tmux-copy-hop"
chmod +x "$install_dir/tmux-copy-hop"

echo "Installed tmux-copy-hop to $install_dir/tmux-copy-hop"
echo
echo "Add this to tmux.conf:"
echo '  bind-key j run-shell "tmux-copy-hop jump"'
echo '  bind-key l run-shell "tmux-copy-hop line-jump"'
echo
echo "Make sure $install_dir is on PATH inside tmux."
