#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_NAME="xai-grok-pager"   # binary do cargo build ra
APP_NAME="xgrok"            # tên lệnh bạn muốn gõ

# Thư mục cài: mặc định ~/.local/bin, ghi đè bằng $1 hoặc biến INSTALL_DIR
INSTALL_DIR="${1:-${INSTALL_DIR:-$HOME/.local/bin}}"

cd "$REPO_DIR"

# Cargo install (dotslash) cài vào ~/.cargo/bin; thêm vào PATH để script này
# tự dùng được ngay cả khi shell của user chưa có sẵn.
export PATH="$HOME/.cargo/bin:$PATH"

# ── Tìm protoc ──────────────────────────────────────────────────────
# Ưu tiên: (1) protoc hệ thống chạy được → (2) bin/protoc của repo
# (dotslash wrapper, tự tải protoc v29.3 từ GitHub releases) → (3) cài
# dotslash rồi dùng bin/protoc. Kiểm tra bằng cách CHẠY thử (--version),
# không chỉ `command -v` — một dotslash wrapper thiếu dotslash vẫn "tồn tại"
# nhưng không chạy được, khiến cargo build fail sâu bên trong prost.
protoc_on_path() {
  command -v protoc >/dev/null 2>&1 && protoc --version >/dev/null 2>&1
}
repo_protoc_ok() {
  command -v dotslash >/dev/null 2>&1 && "$REPO_DIR/bin/protoc" --version >/dev/null 2>&1
}

if protoc_on_path; then
  echo "==> Dùng protoc từ PATH: $(protoc --version 2>/dev/null | head -1)"
elif repo_protoc_ok; then
  echo "==> Dùng bin/protoc (dotslash wrapper) của repo"
else
  echo "==> Chưa có protoc chạy được. Cài 'dotslash' để dùng bin/protoc của repo..."
  if ! command -v dotslash >/dev/null 2>&1; then
    cargo install dotslash || {
      echo "ERROR: cài dotslash thất bại. Gợi ý: brew install protobuf" >&2
      exit 1
    }
  fi
  if ! repo_protoc_ok; then
    echo "ERROR: bin/protoc (dotslash wrapper) không chạy được dù đã có dotslash. Gợi ý: brew install protobuf" >&2
    exit 1
  fi
  echo "==> Dùng bin/protoc (dotslash wrapper) của repo"
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
