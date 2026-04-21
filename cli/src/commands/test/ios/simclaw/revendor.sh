#!/usr/bin/env bash
# Re-vendor the simClaw tree from a local checkout of the upstream repo.
#
# Usage:
#   ./revendor.sh <path-to-homebrew-simClaw>
#
# Copies bin/ + lib/simclaw/ from the upstream checkout into this directory,
# then prints the upstream HEAD SHA and a diff of file names so you know
# whether SIMCLAW_FILES in ../mod.rs needs updating.

set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <path-to-homebrew-simClaw>" >&2
  exit 2
fi

UPSTREAM="$1"
HERE="$(cd "$(dirname "$0")" && pwd)"

[[ -d "$UPSTREAM/bin" && -d "$UPSTREAM/lib/simclaw" ]] \
  || { echo "ERROR: $UPSTREAM does not look like homebrew-simClaw (missing bin/ or lib/simclaw/)" >&2; exit 1; }

echo "==> Snapshotting old file list"
OLD_FILES=$(cd "$HERE" && find bin lib -type f 2>/dev/null | sort)

echo "==> Clearing existing bin/ and lib/"
rm -rf "$HERE/bin" "$HERE/lib"

echo "==> Copying bin/ and lib/"
cp -R "$UPSTREAM/bin" "$HERE/bin"
cp -R "$UPSTREAM/lib" "$HERE/lib"

echo "==> Diffing file list"
NEW_FILES=$(cd "$HERE" && find bin lib -type f 2>/dev/null | sort)
ADDED=$(comm -13 <(echo "$OLD_FILES") <(echo "$NEW_FILES"))
REMOVED=$(comm -23 <(echo "$OLD_FILES") <(echo "$NEW_FILES"))

if [[ -n "$ADDED" ]]; then
  echo ""
  echo "NEW files (must be added to SIMCLAW_FILES in ../mod.rs):"
  echo "$ADDED" | sed 's/^/  + /'
fi
if [[ -n "$REMOVED" ]]; then
  echo ""
  echo "REMOVED files (must be removed from SIMCLAW_FILES in ../mod.rs):"
  echo "$REMOVED" | sed 's/^/  - /'
fi
if [[ -z "$ADDED" && -z "$REMOVED" ]]; then
  echo "No file additions or removals — manifest is still correct."
fi

echo ""
if command -v git >/dev/null && [[ -d "$UPSTREAM/.git" ]]; then
  SHA=$(git -C "$UPSTREAM" rev-parse HEAD)
  SUBJECT=$(git -C "$UPSTREAM" log -1 --format='%s')
  DATE=$(date -u +%Y-%m-%d)
  echo "==> Upstream provenance"
  echo "  Commit: $SHA"
  echo "  Ref:    $SUBJECT"
  echo "  Date:   $DATE"
  echo ""
  echo "Update these fields in VENDORED.md before committing."
fi
