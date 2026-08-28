# Upstream merge playbook (AI-oriented)

This document is a step-by-step guide for syncing this fork with upstream
[xai-org/grok-build](https://github.com/xai-org/grok-build). Follow it
verbatim when performing automated upstream merges.

## Fork purpose — keep upstream's core intact

**This fork changes upstream as little as possible.** It exists for one
reason: to let third-party models (DeepSeek, OpenRouter, any OpenAI-compatible
API) run reliably on the upstream agent/TUI core with Bring Your Own Key
(BYOK) configuration — plus a few small TUI ergonomics — and to trim what a
personal BYOK fork doesn't need (billing/paywall, product telemetry).
Everything else is taken from upstream exactly as shipped.

- **KEEP (upstream-owned — never fork):** the agent runtime, TUI, tools, core
  features, module layout, generated/read-only files. Upstream is the source
  of truth; every sync imports its changes wholesale.
- **ADAPT (fork-owned — thin layer only):** model config parsing
  (`input_modalities`), text-only image stripping, BYOK auth/sampling, and a
  small set of TUI ergonomics (`/clear`, `/new`, task-list expansion).
- **TRIM (fork-owned — removals):** upstream features this fork doesn't need.
  Already done: the consumer free→paid paywall in
  `crates/codegen/xai-grok-pager/src/app/subscription.rs` (BYOK sessions have
  no grok.com billing; the server-driven gate chokepoint is kept). Target:
  product telemetry/analytics for privacy (`xai-grok-telemetry` — Mixpanel,
  Sentry, OTel — and its wiring in `xai-grok-pager-bin/src/main.rs`).
- **NEVER:** reimplement upstream features, redesign upstream UI, restructure
  upstream modules, or carry features that only this fork would maintain.

**Scope test — a change belongs in this fork only if it is one of:**

1. A BYOK / third-party-model adaptation.
2. A small TUI ergonomic improvement.
3. A trim — removal of something this fork doesn't need (billing, telemetry,
   …) — or a genuine bug fix that upstream hasn't accepted yet.

Anything else belongs upstream, not here.

**Pull everything, adapt after.** When upstream ships new features, merge them
wholesale — do **not** pre-filter or pre-adapt. Grok's own models are
OpenAI-compatible, so new upstream features keep working with BYOK models.
BYOK adaptation and trimming happen afterwards, in the same pass that re-applies
the fork layer.

## Remotes

| Remote | Repository | Role |
|--------|------------|------|
| `origin` | `weseegod/thanh` | This fork — push here |
| `upstream` | `xai-org/grok-build` | Source of truth for core |

Setup (if `upstream` is missing):

```bash
git remote add upstream https://github.com/xai-org/grok-build
git fetch --all
```

## Standard merge workflow

1. Ensure a clean working tree on `main`:

   ```bash
   git checkout main
   git status   # must be clean
   ```

2. Create a merge branch. If `merge/upstream-main` already exists from a
   previous sync, re-point it at `main` instead of creating anew — its old
   tip is normally already an ancestor of `main`, so nothing is lost:

   ```bash
   git checkout -B merge/upstream-main main
   ```

3. Fetch and merge upstream (fetch first — never merge a stale ref):

   ```bash
   git fetch upstream
   git merge upstream/main
   ```

4. Resolve conflicts using the rules in [Conflict resolution](#conflict-resolution).
   The merge brings upstream **in full** — all new features arrive, so do not
   filter features out mid-merge; fork ADAPT/TRIM changes are re-applied on
   top afterwards.

5. Verify the build (fast `cargo check` pre-gate first, then the slow full
   build — see [Post-merge verification](#post-merge-verification)):

   ```bash
   cargo check -p xai-grok-shell -p xai-grok-pager -p xai-grok-sampling-types
   ./build.sh
   ```

6. Merge into `main` (direct delivery):

   ```bash
   git checkout main
   git merge merge/upstream-main
   ```

   Alternative — **PR delivery** (use when the user wants review before
   `main` moves; see [Aug 2026 sync #2](#reference-aug-2026-syncs)):

   ```bash
   git push origin merge/upstream-main
   gh pr create -R <fork> --base main --head merge/upstream-main
   # merge via GitHub, then update local main:
   git checkout main && git pull
   ```

7. Push `main` (only when the user explicitly asks):

   ```bash
   git push origin main
   ```

8. **Bump the fork version and publish a release** (see
   [Release & versioning](#release--versioning)) — **do this after every
   sync** so `thanh update` (Ctrl+U) on all machines picks up the new
   binaries (only skip if the user explicitly says no release):

   ```bash
   scripts/publish_release.sh
   ```

   The script bumps `xai-grok-version` + `xai-grok-pager-bin` (+ `Cargo.lock`),
   tags `vX.Y.Z`, builds `thanh` **locally** via `./build.sh` for the current
   platform, and publishes the GitHub Release with the local binary + `stable`/
   `alpha` pointers. **There is no CI** — the fork builds and releases from the
   machine running the script (needs `gh` installed + authenticated). Before
   closing out the sync, confirm the release + `stable` pointer are live
   (`gh release view vX.Y.Z`). To ship other platforms, build on each machine
   and `gh release upload vX.Y.Z thanh-...-<os>-<arch>`.

**Strategy:** always **merge** `upstream/main` into a branch off fork `main`.
Do **not** rebase fork commits onto upstream — that drops fork history and
makes BYOK customizations harder to track.

**Pull everything, adapt after.** When upstream ships new features, merge them
wholesale — do **not** pre-filter or pre-adapt. Grok's own models are
OpenAI-compatible, so new upstream features keep working with BYOK models.
BYOK adaptation and trimming happen afterwards, in the same pass that re-applies
the fork layer.

```mermaid
flowchart LR
  upstreamMain["upstream/main"]
  forkMain["origin/main"]
  mergeBranch["merge/upstream-main"]
  mainOut["main"]

  forkMain --> mergeBranch
  upstreamMain --> mergeBranch
  mergeBranch --> mainOut
```

## Fork-owned inventory

These paths contain fork customizations. Preserve them during merges.

| Category | Paths | Rule |
|----------|-------|------|
| Fork-only files | `build.sh`, `docs/byok-models.md` | Never delete; keep fork version |
| Fork release pipeline | `scripts/publish_release.sh` (self-build via `./build.sh`; **no CI** — `.github/` is removed) | Never delete; keep fork-owned |
| Self-update feed (`thanh`) | `crates/codegen/xai-grok-update/src/version.rs`, `auto_update.rs`, `crates/codegen/xai-grok-config/src/paths.rs`, `crates/codegen/xai-fast-worktree/src/db/mod.rs` (`resolve_grok_home`) | Keep fork feed (`weseegod/thanh` releases), fork home `~/.thanh` (default in `default_grok_home()`/`resolve_grok_home()`, never upstream's `~/.grok`), `thanh` managed binary name (`~/.thanh/bin/thanh`, assets `thanh-<ver>-<os>-<arch>`), `version-thanh.json` cache, single-link swap (never touch `bin/grok`/`bin/agent`) |
| Version lockstep | `crates/codegen/xai-grok-version/Cargo.toml`, `crates/codegen/xai-grok-pager-bin/Cargo.toml` | Keep fork version; bump after every sync (see [Release & versioning](#release--versioning)) |
| BYOK model config | `crates/codegen/xai-grok-shell/src/agent/config.rs`, `config_model_override_parse.rs`, `models.rs` | Keep fork `input` / `input_modalities` parsing and text-only capability checks |
| Image stripping | `crates/codegen/xai-grok-sampling-types/src/conversation.rs`, `types.rs`, `crates/codegen/xai-grok-shell/src/session/compaction.rs`, `acp_session_impl/turn.rs`, `helpers/full_replace_compaction.rs` | Keep `strip_image_parts_for_text_only` and all call sites |
| BYOK auth/sampling | `crates/codegen/xai-grok-shell/src/session/acp_session.rs`, `acp_session_impl/sampler_turn.rs`, `acp_session_impl/spawn.rs`, `crates/codegen/xai-grok-sampler/src/client.rs`, `crates/codegen/xai-grok-shell/src/remote/client.rs` | Keep BYOK auth memo; do not refresh session tokens against third-party endpoints |
| BYOK goal evaluator | `crates/codegen/xai-grok-shell/src/session/acp_session_impl/goal.rs`, `goal_evaluator.rs` | Keep preferred-model goal-evaluator logic (`effective_suggest_model` from pin, fallback to active) and `GOAL_EVALUATOR_TIMEOUT` guard (re-add const if upstream removes it) |
| Fork TUI UX | `crates/codegen/xai-grok-pager/src/slash/commands/clear.rs`, `new.rs`, `views/turn_status.rs`, `views/tasks_pane.rs`, related `agent_view/` and `dispatch/` changes | Keep fork UX; merge upstream structural refactors around them |
| Fork docs edits | `crates/codegen/xai-grok-pager/docs/user-guide/04-slash-commands.md`, `05-configuration.md`, `11-custom-models.md`, `17-sessions.md`, `20-background-tasks.md` | Prefer upstream wording, then re-apply fork additions |
| Trims (removals) | `crates/codegen/xai-grok-pager/src/app/subscription.rs` (fork-modified: paywall removed, server-driven gate chokepoint kept); target removals: `crates/codegen/xai-grok-telemetry/` (Mixpanel, Sentry, OTel) + its wiring in `crates/codegen/xai-grok-pager-bin/src/main.rs`, config in `crates/codegen/xai-grok-config-types` | Keep the fork trim; after every sync take upstream's re-additions first, then re-apply the removal |

The table is illustrative, not exhaustive — many more files carry fork
markers (e.g. `xai-grok-pager/src/acp/model_state.rs`, `xai-grok-pager/src/models.rs`,
`xai-grok-shell/src/agent/auth_method.rs`, `model_providers.rs`, `cli_models.rs`,
`xai-grok-memory/src/backend.rs`, `xai-grok-models/default_models.json`). The marker
diff in [Post-merge verification](#post-merge-verification) is the source of truth.

When both sides touch a hot file in non-overlapping hunks, git auto-merges
without a conflict — the fork hunks still need checking. In the Aug 2026
sync #2, 7 inventory files merged that way; the marker diff caught no loss.

## Release & versioning

The fork ships binaries as **`thanh`** (not `grok`) with its own home
**`~/.thanh`** (config, auth, sessions, `bin/`, `downloads/`, caches) so it
runs fully isolated from an official grok install that keeps `~/.grok`.
Release assets on `weseegod/thanh` GitHub Releases are named
`thanh-<version>-<os>-<arch>` (e.g. `thanh-0.2.122-macos-aarch64`), plus
plain-text `stable` / `alpha` channel pointers that the built-in updater
(Ctrl+U / `thanh update`) reads from `releases/latest/download/`.

Rules:

- **Version** is plain 3-part semver, strictly increasing per release — the
  stable channel rejects pre-release targets (`0.2.121-thanh.1` would never
  be considered an update). Keep `xai-grok-version` and `xai-grok-pager-bin`
  lockstepped (they already are, both synced to upstream's current version).
- **After every upstream sync**, bump the version (typically patch
  `1.0.0 → 1.0.1`) and publish: `scripts/publish_release.sh`. It builds
  `thanh` **locally** via `./build.sh` for the machine it runs on (no CI).
  `gh` must be installed and authenticated (`gh auth login`) to create the
  GitHub Release; to ship other platforms, build on each machine and
  `gh release upload vX.Y.Z thanh-...-<os>-<arch>`.
- **Verify the release after publishing**: the `stable`/`alpha` pointers and
  the `thanh-<ver>-<os>-<arch>` assets must exist on the GitHub Release
  before `thanh update` can serve them — check `gh release view vX.Y.Z`.
- The updater's default installer is `internal` (pure HTTP against the fork's
  GitHub Releases); `gh-release` (needs `gh`) is also supported. It manages
  `~/.thanh/bin/thanh` only and never touches grok's `~/.grok` tree.

### Fork commit map

Use `git log upstream/main..origin/main` to see current fork-only commits.
Historical fork commits (reference):

| Commit | Summary |
|--------|---------|
| `f7e1933` | `input_modalities` in model config + BYOK docs |
| `7834d58` | Strip image parts for text-only BYOK models |
| `588bac6` | BYOK model adaptation (sampler, compaction, goal classifier) |
| `aa7516d` | `/clear` command clears TUI texts |
| `5202d9d` | `/new` command keeps same model |
| `641974a` | Click still-running status to expand inline task list |
| `e61c126` | Add `build.sh` to build and install thanh binary |
| `2c0193f` | Fix `build.sh` to auto-install dotslash for `bin/protoc` |
| `b9b61ba` | Add UPSTREAM-MERGE.md playbook + document fork BYOK goal |
| `c2b4b6f` | Stop subagent child sessions when workflow cancel lands |

## Conflict resolution

| Situation | Action |
|-----------|--------|
| Any change not required by BYOK support, small TUI ergonomics, or the trim list | Take **upstream** — it does not belong in the fork |
| File not in fork inventory above | Take **upstream** |
| Fork-only file (`build.sh`, `docs/byok-models.md`) | Keep **fork** |
| Shared hot file (`models.rs`, `config.rs`, `compaction.rs`, etc.) | **Combine**: upstream refactor/renames + fork BYOK logic |
| Generated/read-only (`Cargo.toml` workspace root, `SOURCE_REV`) | Take **upstream** |
| Upstream deleted/renamed something the fork touched | Follow **upstream** structure; re-port fork logic into new locations |

Trims (removals) are re-applied **after** the wholesale merge, not decided
during conflict resolution: take upstream's re-additions first, then delete
again (see the inventory TRIM row).

After resolving shared hot files, grep for fork markers:

```bash
rg "strip_image_parts_for_text_only|input_modalities|ModelByok|byok" --type rust
```

All expected matches must still be present.

### Common upstream structural changes

Upstream monorepo syncs may rename or remove modules. Examples seen in past merges:

- `project_picker/` renamed to `recent_dirs.rs` — follow upstream layout
- `trace_classifier/` removed — do not restore; take upstream deletion

When upstream refactors a file the fork also edited, read both sides and
merge function-by-function rather than picking one side wholesale.

## Post-merge verification

Run all checks before merging to `main`:

```bash
# Fast pre-gate (a few minutes) — run first; the full release build is slow
cargo check -p xai-grok-shell -p xai-grok-pager -p xai-grok-sampling-types
# Full gate (slow) — this is also the self-build path used for releases
./build.sh
```

Verify fork markers survived the merge — diff marker lines between the
pre-merge `main` tip and the merged tree. Identical output means fork logic
is fully preserved:

```bash
git grep -n "strip_image_parts_for_text_only|input_modalities|ModelByok" <main-tip> -- '*.rs' | sed 's/<main-tip>://' | sort > /tmp/pre.txt
git grep -n "strip_image_parts_for_text_only|input_modalities|ModelByok" -- '*.rs' | sort > /tmp/post.txt
diff /tmp/pre.txt /tmp/post.txt   # empty = preserved

# Spot-check markers, including non-inventory files:
rg "strip_image_parts_for_text_only|input_modalities|ModelByok|byok" --type rust
```

Note: `git grep -c` and `rg -c` both count matching lines but their output
formats differ — use `git grep -n` (not `-c`) when diffing pre vs post.

Manual checks:

- [ ] `docs/byok-models.md` exists
- [ ] `build.sh` exists and is executable
- [ ] `./build.sh` prints a version (e.g. `thanh 0.2.x`)
- [ ] Fork markers preserved: `strip_image_parts_for_text_only|input_modalities|ModelByok|byok`, plus `weseegod/thanh`, `version-thanh.json`, `~/.thanh`, `bin/thanh`
- [ ] No conflict markers left in source (`rg -n '^(<<<<<<<|=======|>>>>>>>)'` — match at line start only; mid-line matches in string literals are false positives)
- [ ] Release published after the sync (when binaries are shipped): `gh release view vX.Y.Z` shows the `stable` pointer + `thanh-<ver>-<os>-<arch>` assets

## Anti-patterns

- Do **not** force-push `main`
- Do **not** rebase fork commits onto upstream
- Do **not** delete `build.sh` or `docs/byok-models.md`
- Do **not** commit API keys or real credentials from `~/.thanh/config.toml`
- Do **not** add fork-only features unrelated to BYOK support, small TUI
  ergonomics, or trimming — propose them upstream instead
- Do **not** pre-filter upstream features during a merge — bring everything
  in first, then adapt/trim
