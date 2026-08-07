#!/usr/bin/env bash
# Bump the fork version, tag it, and push — this triggers the CI release
# workflow (`.github/workflows/release.yml`) which builds `thanh` for every
# platform and publishes a GitHub Release with the `stable` pointer + binaries.
#
# Usage:
#   scripts/publish_release.sh            # bump patch: 0.2.121 -> 0.2.122
#   scripts/publish_release.sh 0.3.0      # explicit version
#
# Version rule (see UPSTREAM-MERGE.md): plain 3-part semver, strictly
# increasing per release — the stable channel rejects pre-release targets.
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_DIR"

VERSION_FILE="crates/codegen/xai-grok-version/Cargo.toml"
PAGER_FILE="crates/codegen/xai-grok-pager-bin/Cargo.toml"

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

# Must be valid 3-part semver (no pre-release suffix — the stable channel
# rejects pre-releases).
if ! echo "$new" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
  echo "ERROR: '$new' is not plain 3-part semver (e.g. 0.2.122)" >&2
  exit 1
fi

for f in "$VERSION_FILE" "$PAGER_FILE"; do
  sed -i -E "s/^version = \"[^\"]+\"/version = \"$new\"/" "$f"
done

git add "$VERSION_FILE" "$PAGER_FILE"
git commit -m "Release v$new"
git tag "v$new"
git push origin main
git push origin "v$new"

echo "==> Pushed v$new — CI is building and publishing the release."
echo "==> After the release is live, macOS/Ubuntu users get it via \`thanh update\` (or Ctrl+U in the TUI)."
