#!/usr/bin/env bash
# Bump the fork version, build `thanh` locally, and publish a GitHub Release
# with the local binary — NO GitHub Actions / CI.
#
# The updater (`thanh update` / Ctrl+U) reads from a GitHub Release:
#   releases/latest/download/stable          (plain-text channel pointer)
#   releases/latest/download/thanh-<ver>-<os>-<arch>  (the binary)
#   releases/latest/download/thanh-<ver>-<os>-<arch>.sha256
#
# Usage:
#   scripts/publish_release.sh            # bump patch: 1.0.0 -> 1.0.1
#   scripts/publish_release.sh 1.1.0      # explicit version
#
# Prereqs / notes:
#   - Binary is built LOCALLY via ./build.sh for the CURRENT platform only.
#     To ship other platforms, build on each machine and upload the assets to
#     the release yourself (e.g. `gh release upload vX.Y.Z thanh-...-macos-aarch64`).
#   - The final publish uses `gh`; install it (brew install gh / apt install gh)
#     and authenticate once (gh auth login). If `gh` is missing the script
#     bumps/tags/pushes and prints the exact `gh release create` command to run.
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_DIR"

VERSION_FILE="crates/codegen/xai-grok-version/Cargo.toml"
PAGER_FILE="crates/codegen/xai-grok-pager-bin/Cargo.toml"
LOCK_FILE="Cargo.lock"
APP="thanh"
REPO="weseegod/thanh"

if [ ! -f "$VERSION_FILE" ] || [ ! -f "$PAGER_FILE" ]; then
  echo "ERROR: lockstepped Cargo.toml files not found" >&2
  exit 1
fi

current="$(grep -m1 '^version = ' "$VERSION_FILE" | sed -E 's/^version = "([^"]+)"/\1/')"
echo "==> Current version: $current"

if [ $# -ge 1 ]; then
  new="$1"
  echo "==> Bumping to requested version: $new"
else
  major="$(echo "$current" | cut -d. -f1)"
  minor="$(echo "$current" | cut -d. -f2)"
  patch="$(echo "$current" | cut -d. -f3)"
  new="$major.$minor.$((patch + 1))"
  echo "==> Bumping patch to: $new"
fi

# Plain 3-part semver, strictly increasing — the stable channel rejects pre-releases.
if ! echo "$new" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
  echo "ERROR: '$new' is not plain 3-part semver (e.g. 1.0.1)" >&2
  exit 1
fi

# Bump the two lockstepped version crates.
for f in "$VERSION_FILE" "$PAGER_FILE"; do
  sed -i -E "s/^version = \"[^\"]+\"/version = \"$new\"/" "$f"
done

# Bump the same two entries in Cargo.lock (portable via awk).
awk -v new="$new" '
  /^name = "xai-grok-version"$/ || /^name = "xai-grok-pager-bin"$/ { name=1 }
  name && /^version = / { sub(/^version = ".*"/, "version = \"" new "\""); name=0 }
  { print }
' "$LOCK_FILE" > "$LOCK_FILE.tmp" && mv "$LOCK_FILE.tmp" "$LOCK_FILE"

git add "$VERSION_FILE" "$PAGER_FILE" "$LOCK_FILE"
git commit -m "Release v$new"
git tag "v$new"

echo "==> Building $APP locally (./build.sh)..."
./build.sh

# Stage the binary asset for the current platform.
os="$(uname -s)"
arch="$(uname -m)"
case "$os-$arch" in
  Linux-x86_64)             platform="linux-x86_64" ;;
  Darwin-arm64|Darwin-aarch64) platform="macos-aarch64" ;;
  Darwin-x86_64)            platform="macos-x86_64" ;;
  *) platform="$(echo "$os" | tr '[:upper:]' '[:lower:]')-$(echo "$arch" | tr '[:upper:]' '[:lower:]')" ;;
esac
asset="$APP-$new-$platform"
cp "target/release/xai-grok-pager" "$asset"
chmod +x "$asset"
shasum -a 256 "$asset" > "$asset.sha256"
printf '%s\n' "$new" > stable
printf '%s\n' "$new" > alpha

git push origin main
git push origin "v$new"

cleanup() {
  rm -f "$asset" "$asset.sha256" stable alpha
}

if ! command -v gh >/dev/null 2>&1; then
  echo "==> v$new tagged & pushed; asset built: $asset"
  echo "==> 'gh' not found. Install + authenticate, then run:"
  echo "      gh auth login"
  echo "      gh release create v$new $asset $asset.sha256 stable alpha \\"
  echo "          --repo $REPO --title v$new --generate-notes"
  echo "    (then: rm -f $asset $asset.sha256 stable alpha)"
  exit 0
fi

gh release create "v$new" "$asset" "$asset.sha256" stable alpha \
  --repo "$REPO" --title "v$new" --generate-notes
cleanup
echo "==> Released v$new. Users get it via \`thanh update\` (Ctrl+U)."
