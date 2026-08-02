#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_NAME="xai-grok-pager"   # binary do cargo build ra
APP_NAME="xgrok"            # tên lệnh bạn muốn gõ

# Thư mục cài: mặc định ~/.local/bin, ghi đè bằng $1 hoặc biến INSTALL_DIR
INSTALL_DIR="${1:-${INSTALL_DIR:-$HOME/.local/bin}}"

cd "$REPO_DIR"

# Kiểm tra tiền đề build: cần protoc hoặc dotslash
if ! command -v protoc >/dev/null 2>&1 && ! command -v dotslash >/dev/null 2>&1; then
  echo "ERROR: cần 'protoc' (hoặc 'dotslash') để build proto. Gợi ý: brew install protobuf" >&2
  exit 1
fi

echo "==> Building $APP_NAME (release)..."
cargo build -p xai-grok-pager-bin --release

mkdir -p "$INSTALL_DIR"
if [ -w "$INSTALL_DIR" ]; then
  install -m 755 "target/release/$BIN_NAME" "$INSTALL_DIR/$APP_NAME"
else
  echo "==> $INSTALL_DIR không ghi được (cần quyền root), thử với sudo..."
  sudo install -m 755 "target/release/$BIN_NAME" "$INSTALL_DIR/$APP_NAME"
fi

echo "==> Đã cài: $INSTALL_DIR/$APP_NAME"
"$INSTALL_DIR/$APP_NAME" --version
