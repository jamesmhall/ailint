#!/usr/bin/env bash
#
# scripts/release.sh — cut an ailint release.
#
# Usage: scripts/release.sh <X.Y.Z>
#
# See RELEASING.md for the full procedure and recovery guidance.

set -euo pipefail

VERSION="${1:-}"
if [[ -z "${VERSION}" ]]; then
  echo "usage: scripts/release.sh <X.Y.Z>" >&2
  exit 2
fi

if ! [[ "${VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "error: version '${VERSION}' is not X.Y.Z" >&2
  exit 2
fi

TAG="v${VERSION}"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${REPO_ROOT}"

log()  { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m==>\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m==>\033[0m %s\n' "$*" >&2; exit 1; }

# ---------------------------------------------------------------------------
# 1. Preflight
# ---------------------------------------------------------------------------
log "Preflight: git state"

if [[ -n "$(git status --porcelain)" ]]; then
  die "working tree not clean; commit or stash first"
fi

BRANCH="$(git rev-parse --abbrev-ref HEAD)"
if [[ "${BRANCH}" != "main" ]]; then
  die "must be on main (currently on ${BRANCH})"
fi

git fetch origin --tags --prune

if git rev-parse "${TAG}" >/dev/null 2>&1 || \
   git ls-remote --tags origin "${TAG}" | grep -q "${TAG}"; then
  die "tag ${TAG} already exists locally or on origin"
fi

LOCAL="$(git rev-parse main)"
REMOTE="$(git rev-parse origin/main)"
if [[ "${LOCAL}" != "${REMOTE}" ]]; then
  die "local main (${LOCAL:0:7}) differs from origin/main (${REMOTE:0:7}); pull first"
fi

# Read current version to detect "already bumped"
CUR_VERSION="$(grep -E '^version = "' Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/')"
if [[ "${CUR_VERSION}" == "${VERSION}" ]]; then
  warn "workspace version is already ${VERSION}; skipping bump step"
  ALREADY_BUMPED=1
else
  ALREADY_BUMPED=0
fi

# ---------------------------------------------------------------------------
# 2. Version bump (all 4 locations)
# ---------------------------------------------------------------------------
if [[ "${ALREADY_BUMPED}" -eq 0 ]]; then
  log "Bumping versions ${CUR_VERSION} -> ${VERSION}"

  # Root Cargo.toml [workspace.package] version
  # Root Cargo.toml [workspace.dependencies] ailint-core / ailint-llm version pins
  python3 - <<PY
import re, pathlib
p = pathlib.Path("Cargo.toml")
s = p.read_text()
# workspace.package version
s = re.sub(r'(\[workspace\.package\][^\[]*?\nversion = ")[^"]+(")',
           lambda m: m.group(1) + "${VERSION}" + m.group(2), s, count=1)
# internal dep pins
s = re.sub(r'(ailint-core = \{ path = "crates/ailint-core", version = ")[^"]+(")',
           lambda m: m.group(1) + "${VERSION}" + m.group(2), s)
s = re.sub(r'(ailint-llm  = \{ path = "crates/ailint-llm",  version = ")[^"]+(")',
           lambda m: m.group(1) + "${VERSION}" + m.group(2), s)
s = re.sub(r'(ailint-llm = \{ path = "crates/ailint-llm", version = ")[^"]+(")',
           lambda m: m.group(1) + "${VERSION}" + m.group(2), s)
p.write_text(s)
PY

  # npm/package.json
  python3 - <<PY
import json, pathlib
p = pathlib.Path("npm/package.json")
d = json.loads(p.read_text())
d["version"] = "${VERSION}"
p.write_text(json.dumps(d, indent=2) + "\n")
PY

  # README pre-commit example rev
  python3 - <<PY
import re, pathlib
p = pathlib.Path("README.md")
s = p.read_text()
s = re.sub(r'(rev: v)[0-9]+\.[0-9]+\.[0-9]+', r'\g<1>${VERSION}', s)
p.write_text(s)
PY

  # Refresh Cargo.lock
  log "Refreshing Cargo.lock via cargo build"
  cargo build --workspace --quiet
else
  log "Skipping bump; ensuring Cargo.lock is current"
  cargo build --workspace --quiet
fi

# ---------------------------------------------------------------------------
# 3. Validate
# ---------------------------------------------------------------------------
log "cargo fmt --all --check"
cargo fmt --all --check

log "cargo clippy --workspace --all-targets -- -D warnings"
cargo clippy --workspace --all-targets -- -D warnings

log "cargo test --workspace"
cargo test --workspace --quiet

log "cargo publish --dry-run (all three crates)"
cargo publish --dry-run -p ailint-core --allow-dirty --quiet
cargo publish --dry-run -p ailint-llm  --allow-dirty --quiet
cargo publish --dry-run -p ailint-cli  --allow-dirty --quiet

# ---------------------------------------------------------------------------
# 4. Commit release bump (if there's anything to commit) and push
# ---------------------------------------------------------------------------
if [[ -n "$(git status --porcelain)" ]]; then
  log "Committing 'chore: release ${TAG}'"
  git add -A
  git commit -m "chore: release ${TAG}"
  log "Pushing main"
  git push origin main
else
  log "No changes to commit (already released bump?); continuing"
fi

# ---------------------------------------------------------------------------
# 5. Tag and push
# ---------------------------------------------------------------------------
log "Tagging ${TAG}"
git tag -a "${TAG}" -m "Release ${TAG}"
log "Pushing tag (triggers Release workflow)"
git push origin "${TAG}"

# ---------------------------------------------------------------------------
# 6. Watch the release workflow
# ---------------------------------------------------------------------------
if command -v gh >/dev/null 2>&1; then
  log "Waiting for Release workflow to start..."
  # Give GitHub a moment to register the tag and start the workflow.
  RUN_ID=""
  for _ in {1..30}; do
    sleep 2
    RUN_ID="$(gh run list --workflow=release.yml --limit=1 \
      --json databaseId,headBranch,event \
      --jq '.[] | select(.event=="push") | .databaseId' 2>/dev/null || true)"
    [[ -n "${RUN_ID}" ]] && break
  done

  if [[ -z "${RUN_ID}" ]]; then
    warn "could not find release workflow run; check https://github.com/jamesmhall/ailint/actions"
  else
    log "Watching release workflow run ${RUN_ID}"
    if ! gh run watch "${RUN_ID}" --exit-status; then
      die "release workflow failed; see RELEASING.md 'Recovery' before retrying"
    fi
  fi
else
  warn "gh CLI not installed; watch the release workflow manually:"
  warn "  https://github.com/jamesmhall/ailint/actions"
  read -r -p "Press Enter once the Release workflow finishes successfully..." _
fi

# ---------------------------------------------------------------------------
# 7. Update Homebrew formula
# ---------------------------------------------------------------------------
log "Updating Homebrew formula"

URL="https://github.com/jamesmhall/ailint/archive/refs/tags/${TAG}.tar.gz"

# Small retry loop; GitHub sometimes needs a beat to materialise the tarball.
SHA=""
for _ in {1..10}; do
  if SHA="$(curl -fsSL "${URL}" | shasum -a 256 | cut -d' ' -f1)"; then
    [[ -n "${SHA}" ]] && break
  fi
  sleep 3
done
[[ -n "${SHA}" ]] || die "could not fetch ${URL}; run the manual formula step from RELEASING.md"

# BSD sed on macOS needs '' after -i; GNU sed does not. Detect.
if sed --version >/dev/null 2>&1; then
  SED_INPLACE=(sed -i -E)
else
  SED_INPLACE=(sed -i '' -E)
fi

"${SED_INPLACE[@]}" \
  "s|archive/refs/tags/v[0-9][0-9.]*\.tar\.gz|archive/refs/tags/${TAG}.tar.gz|" \
  Formula/ailint.rb
"${SED_INPLACE[@]}" \
  "s|^  sha256 \"[0-9a-f]+\"|  sha256 \"${SHA}\"|" \
  Formula/ailint.rb

if [[ -z "$(git status --porcelain Formula/ailint.rb)" ]]; then
  log "Formula already up to date; nothing to commit"
else
  git add Formula/ailint.rb
  git commit -m "chore: update Homebrew formula for ${TAG}"
  git push origin main
fi

log "Done. ${TAG} released to crates.io, npm, ghcr.io, GitHub Releases, and Homebrew."
