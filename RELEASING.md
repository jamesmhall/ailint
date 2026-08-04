# Releasing ailint

Canonical release procedure. This is the source of truth — if you are cutting
a release (human or agent), follow this checklist top to bottom. **No steps
skipped. No guesswork.**

Every user-visible change eventually needs a release. This document explains
what a release is, when to cut one, and exactly how to do it.

## Contents

- [What a release actually is](#what-a-release-actually-is)
- [Semver decision](#semver-decision)
- [Version locations](#version-locations)
- [Expected workflow runs](#expected-workflow-runs)
- [Release checklist](#release-checklist)
- [Automated path (`scripts/release.sh`)](#automated-path-scriptsreleasesh)
- [Manual path (fallback)](#manual-path-fallback)
- [Recovery: something failed mid-release](#recovery-something-failed-mid-release)
- [Notes for AI agents](#notes-for-ai-agents)

## What a release actually is

A release is the coordinated publication of one workspace version to five
distribution channels:

1. **crates.io** — four crates published in dependency order:
   `ailint-extractor`, then `ailint-core`, then `ailint-llm`, then `ailint-cli`.
2. **npm** — the `ailint` package (thin installer that downloads the CLI
   binary).
3. **GitHub Container Registry** — `ghcr.io/jamesmhall/ailint:{tag}` +
   `:latest`, multi-arch.
4. **GitHub Releases** — precompiled binaries for 5 targets + SHA256s +
   sigstore attestations.
5. **Homebrew** (in-repo formula) — `Formula/ailint.rb` `url` and `sha256`
   pointing at the new tag's source tarball.

**A release is not "the merge commit."** A merged PR alone releases nothing.
The release happens when a `v{X.Y.Z}` tag is pushed and the `Release`
workflow runs to completion.

## Semver decision

Bump by user-visible effect, not by internal code churn.

| Change | Bump |
|---|---|
| Bug fix, doc typo, internal refactor | **patch** (`1.0.2` → `1.0.3`) |
| New lint rule, new CLI flag, new reporter, changed terminal output, new config knob | **minor** (`1.0.2` → `1.1.0`) |
| Removed/renamed CLI flag, removed rule, breaking config schema, MSRV bump | **major** (`1.0.2` → `2.0.0`) |

If you can't decide between patch and minor, choose minor. Terminal output
changes count as minor — some downstream tools grep our output.

## Version locations

All must be bumped to the same value in the same commit:

1. `Cargo.toml` — root `[workspace.package] version`
2. `Cargo.toml` — internal dep pins under `[workspace.dependencies]`:
   `ailint-core = { path = "…", version = "X.Y.Z" }`,
   `ailint-llm = { path = "…", version = "X.Y.Z" }`, and
   `ailint-extractor = { path = "…", version = "X.Y.Z" }`
3. `npm/package.json` — `"version": "X.Y.Z"`
4. `README.md` — the pre-commit example rev (`rev: v{X.Y.Z}`)

`Formula/ailint.rb` and `Cargo.lock` update automatically as part of the
release script (or the release workflow / `cargo build`).

## Expected workflow runs

**Per release: exactly 1 GitHub Actions run.**

That one run is `Release`, triggered by pushing the `v*` tag. It contains:

- `test` (3 OSes, matches CI gates)
- `build` (5 target matrix)
- `release` (attest + GitHub Release)
- `publish-crates`
- `publish-docker`

Any additional runs mean something went wrong. See [Recovery](#recovery-something-failed-mid-release).

**CI (`ci.yml`) intentionally does NOT run on release commits or on
`push: main`.** CI is a PR gate only. The `Release` workflow runs the same
validation before publishing, so main-branch pushes do not need separate CI.

## Release checklist

Preflight (before running anything):

- [ ] All PRs for this release are merged. `git log origin/main -1` shows
      the tip you want to ship.
- [ ] Working tree clean: `git status` is empty.
- [ ] On branch `main`, up to date: `git switch main && git pull --ff-only`.
- [ ] Decided the version and semver bump (see [Semver decision](#semver-decision)).
- [ ] `cargo publish --dry-run -p ailint-extractor --allow-dirty` succeeds.
      Only the leaf crate can be dry-run locally; the others depend on
      unpublished internal crates, so their dry-run always fails until
      CI's real publish step runs them in order. The tag snapshots the
      workflow — a broken tag cannot be re-run.

Then choose one path below.

## Automated path (`scripts/release.sh`)

**One command:**

```bash
scripts/release.sh 1.0.3
```

The script performs, in order:

1. Preflight: clean tree, on `main`, up to date with origin, tag doesn't
   exist yet.
2. Bumps all 4 [version locations](#version-locations).
3. `cargo build --workspace` (updates `Cargo.lock`).
4. `cargo fmt --all --check`.
5. `cargo clippy --workspace --all-targets -- -D warnings`.
6. `cargo test --workspace`.
7. `cargo publish --dry-run` for `ailint-extractor` (leaf crate only —
   the other crates depend on unpublished internal crates, so their
   dry-run always fails; CI's real publish step covers them).
8. Commits `chore: release v{X.Y.Z}` and pushes to `main` (admin bypass;
   no CI run because `ci.yml` doesn't trigger on push).
9. Tags `v{X.Y.Z}` and pushes the tag → **triggers the single `Release`
   workflow run**.
10. Polls the release workflow until it completes. On success, moves to
    step 11. On failure, exits with the failing job link.
11. Downloads the GitHub-generated source tarball for the new tag, computes
    its SHA256, updates `Formula/ailint.rb`, commits
    `chore: update Homebrew formula for v{X.Y.Z}`, and pushes to `main`.

The script is idempotent per step — if it fails at step N, fix the cause
and re-run; it detects what's already done.

## Manual path (fallback)

Use only if the script is broken or unavailable.

```bash
# 0. Preflight
git switch main && git pull --ff-only
git status                     # must be clean
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo publish --dry-run -p ailint-extractor --allow-dirty

# 1. Bump versions (edit files by hand):
#    - Cargo.toml   [workspace.package] version = "X.Y.Z"
#    - Cargo.toml   [workspace.dependencies] ailint-core / ailint-llm / ailint-extractor version = "X.Y.Z"
#    - npm/package.json  "version": "X.Y.Z"
#    - README.md    pre-commit rev: v{X.Y.Z}

# 2. Refresh Cargo.lock via a build
cargo build --workspace

# 3. Commit and push
git add -A
git commit -m "chore: release v{X.Y.Z}"
git push origin main

# 4. Tag and push
git tag v{X.Y.Z}
git push origin v{X.Y.Z}       # this triggers the Release workflow

# 5. Watch the release workflow
gh run watch --exit-status

# 6. Update Homebrew formula (only after step 5 succeeds)
TAG=v{X.Y.Z}
URL="https://github.com/jamesmhall/ailint/archive/refs/tags/${TAG}.tar.gz"
SHA=$(curl -fsSL "${URL}" | shasum -a 256 | cut -d' ' -f1)
sed -i '' -E "s|archive/refs/tags/v[0-9][0-9.]*\.tar\.gz|archive/refs/tags/${TAG}.tar.gz|" Formula/ailint.rb
sed -i '' -E "s|^  sha256 \"[0-9a-f]+\"|  sha256 \"${SHA}\"|" Formula/ailint.rb
git add Formula/ailint.rb
git commit -m "chore: update Homebrew formula for ${TAG}"
git push origin main
```

## Recovery: something failed mid-release

**Never delete a tag that has already published to crates.io or npm.**
Crates.io does not allow republishing the same version. If the tag published
crates but the workflow died before finishing Docker or the GitHub Release,
you must bump to the next patch and re-release — do not retry the same tag.

### The `test` job failed on the tag

Nothing was published. Delete the tag locally + remote, fix the code, re-tag:

```bash
git tag -d v{X.Y.Z}
git push origin :refs/tags/v{X.Y.Z}
# fix, commit, push, re-tag, re-push
```

Delete the local "chore: release v{X.Y.Z}" commit if present using
`git reset --hard HEAD~1` **before** doing this, if that commit was already
pushed to main you have to push a revert — do not force-push main.

### `publish-crates` failed on `ailint-core`

Nothing landed. Re-run the failed job from the Actions UI. If a code fix is
needed, cut the next patch version.

### `publish-crates` failed on `ailint-llm` or `ailint-cli`

`ailint-core` already published. You cannot un-publish. Options:

- If the failure is transient (crates.io index lag): re-run the failed job.
- If a fix is needed: bump patch, ship a new tag. The already-published
  `ailint-core` at the old version stays; the new tag will publish all
  three at the new version.

### `publish-docker` failed

Rerun the job. Docker Hub / GHCR are idempotent — safe to retry.

### `release` (GitHub Release) failed

Rerun. Idempotent.

### Homebrew formula update failed (step 11 of the script)

The tag is fully released everywhere except brew. Run the last block of the
[manual path](#manual-path-fallback) yourself.

## Notes for AI agents

- Never run `cargo publish` (non-dry-run) yourself. Never push a tag yourself.
  Always ask the human first; a tag cannot be un-pushed cleanly.
- Never run `git push --tags`. Push individual tags: `git push origin vX.Y.Z`.
- Never `git reset --hard` on `main` or force-push `main`.
- Follow this file exactly. If a step here contradicts something you
  remember, this file wins.
