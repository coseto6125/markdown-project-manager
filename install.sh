#!/usr/bin/env sh
# mpm 一鍵安裝（Linux / macOS）
#
#   curl -sSfL https://github.com/coseto6125/markdown-project-manager/releases/latest/download/install.sh | sh
#   curl -sSfL https://github.com/coseto6125/markdown-project-manager/releases/download/v0.1.0/install.sh | sh
#
# 環境變數：
#   MPM_VERSION       指定版本（不含 v 前綴）。預設 latest。
#   MPM_INSTALL_DIR   安裝目錄。預設 $HOME/.local/bin，root 時 /usr/local/bin。
#   MPM_NO_VERIFY=1   跳過 SHA256 驗證（不建議）。
#   MPM_FORCE_CARGO=1 跳過 release binary，強制走 cargo install --git。
#
# 全程使用 GitHub Release 的匿名公開資產，無需任何帳號 / token。
# 目標平台沒有 prebuilt（或尚無 Release）時，自動 fallback 到
# `cargo install --git`（需要 cargo / rustup）。

set -eu

REPO="coseto6125/markdown-project-manager"
BIN="mpm"
MPM_VERSION="${MPM_VERSION:-latest}"
MPM_FORCE_CARGO="${MPM_FORCE_CARGO:-0}"

if [ -z "${MPM_INSTALL_DIR:-}" ]; then
  if [ "$(id -u)" -eq 0 ]; then
    MPM_INSTALL_DIR="/usr/local/bin"
  else
    MPM_INSTALL_DIR="$HOME/.local/bin"
  fi
fi

cargo_fallback() {
  reason="$1"
  if ! command -v cargo >/dev/null 2>&1; then
    echo "error: $reason" >&2
    echo "       and \`cargo\` not found in PATH — install Rust from https://rustup.rs," >&2
    echo "       then re-run this script (or wait for a prebuilt release)." >&2
    exit 1
  fi
  echo "==> $reason"
  echo "==> Falling back to \`cargo install --git\` (source build, may take a few minutes)"

  build_root="$(mktemp -d 2>/dev/null || mktemp -d -t mpm-build)"
  trap 'rm -rf "$build_root"' EXIT
  if [ "${MPM_VERSION}" = "latest" ]; then
    cargo install --root "$build_root" --git "https://github.com/$REPO" --bin "$BIN" --locked
  else
    cargo install --root "$build_root" --git "https://github.com/$REPO" --tag "v${MPM_VERSION#v}" --bin "$BIN" --locked
  fi
  mkdir -p "$MPM_INSTALL_DIR"
  install -m 0755 "$build_root/bin/$BIN" "$MPM_INSTALL_DIR/$BIN"

  echo
  echo "✓ Installed $BIN via cargo → $MPM_INSTALL_DIR/$BIN"
  exit 0
}

if [ "${MPM_FORCE_CARGO}" = "1" ]; then
  cargo_fallback "MPM_FORCE_CARGO=1 set"
fi

os="$(uname -s | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"

case "$os/$arch" in
  linux/x86_64)               target="x86_64-unknown-linux-gnu" ;;
  linux/aarch64|linux/arm64)  target="aarch64-unknown-linux-gnu" ;;
  darwin/x86_64)              target="x86_64-apple-darwin" ;;
  darwin/arm64|darwin/aarch64) target="aarch64-apple-darwin" ;;
  *)
    cargo_fallback "unsupported prebuilt platform $os/$arch (linux/macOS x86_64/aarch64 only)"
    ;;
esac

if [ "$MPM_VERSION" = "latest" ]; then
  # 從 redirect 解析 latest tag，免 GitHub API 額度。
  tag="$(curl -sSfLI -o /dev/null -w '%{url_effective}' "https://github.com/$REPO/releases/latest" 2>/dev/null | sed -n 's|.*/tag/||p')"
  if [ -z "$tag" ]; then
    cargo_fallback "no published GitHub Release yet for $REPO"
  fi
else
  tag="v${MPM_VERSION#v}"
fi
version="${tag#v}"

name="${BIN}-${tag}-${target}"
archive="${name}.tar.gz"
url="https://github.com/$REPO/releases/download/${tag}/${archive}"
sha_url="${url}.sha256"

tmpdir="$(mktemp -d 2>/dev/null || mktemp -d -t mpm)"
trap 'rm -rf "$tmpdir"' EXIT

echo "==> Downloading $BIN $version ($target)"
echo "    $url"
if ! curl -sSfL "$url" -o "$tmpdir/$archive"; then
  cargo_fallback "release asset for $target not found (tag $tag)"
fi

if [ "${MPM_NO_VERIFY:-0}" != "1" ]; then
  curl -sSfL "$sha_url" -o "$tmpdir/$archive.sha256"
  echo "==> Verifying SHA256"
  if command -v shasum >/dev/null 2>&1; then
    ( cd "$tmpdir" && shasum -a 256 -c "$archive.sha256" )
  elif command -v sha256sum >/dev/null 2>&1; then
    ( cd "$tmpdir" && sha256sum -c "$archive.sha256" )
  else
    echo "warning: no shasum/sha256sum; skipping verification" >&2
  fi
fi

tar -xzf "$tmpdir/$archive" -C "$tmpdir"
mkdir -p "$MPM_INSTALL_DIR"
install -m 0755 "$tmpdir/$name/$BIN" "$MPM_INSTALL_DIR/$BIN"

echo
echo "✓ Installed $BIN $version → $MPM_INSTALL_DIR/$BIN"
echo

case ":$PATH:" in
  *":$MPM_INSTALL_DIR:"*) ;;
  *)
    echo "  ⚠  $MPM_INSTALL_DIR is not in PATH. Add it:"
    case "$(basename "${SHELL:-/bin/sh}")" in
      bash) echo "       echo 'export PATH=\"$MPM_INSTALL_DIR:\$PATH\"' >> ~/.bashrc" ;;
      zsh)  echo "       echo 'export PATH=\"$MPM_INSTALL_DIR:\$PATH\"' >> ~/.zshrc" ;;
      fish) echo "       fish_add_path $MPM_INSTALL_DIR" ;;
      *)    echo "       export PATH=\"$MPM_INSTALL_DIR:\$PATH\"" ;;
    esac
    echo
    ;;
esac
