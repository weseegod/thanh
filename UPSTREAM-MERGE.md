# Upstream merge playbook (AI-oriented)

This document is a step-by-step guide for syncing this fork with upstream
[xai-org/grok-build](https://github.com/xai-org/grok-build). Follow it
verbatim when performing automated upstream merges.

## Project goal

This repository is a **BYOK-focused fork** of Grok Build. Upstream owns the
agent runtime, TUI, tools, and core features. This fork keeps upstream core
as-is and adds a thin customization layer so third-party models (DeepSeek,
OpenRouter, etc.) work reliably with Bring Your Own Key (BYOK) configuration.

Do **not** reimplement upstream features. Extend only where third-party models
need different behavior (model config parsing, text-only image handling, BYOK
auth, and small TUI ergonomics).

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
| File not in fork inventory above | Take **upstream** |
| Fork-only file (`build.sh`, `docs/byok-models.md`) | Keep **fork** |
| Shared hot file (`models.rs`, `config.rs`, `compaction.rs`, etc.) | **Combine**: upstream refactor/renames + fork BYOK logic |
| Generated/read-only (`Cargo.toml` workspace root, `SOURCE_REV`) | Take **upstream** |
| Upstream deleted/renamed something the fork touched | Follow **upstream** structure; re-port fork logic into new locations |

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

## Reference: Aug 2026 syncs

Sync #1 (merge `1e99e1e`):
- 3 upstream commits ("Synced from monorepo") merged with **zero conflicts**
- Build verified: `thanh 0.2.120` via `./build.sh`
- Fork had 8 commits ahead of upstream at merge time

Sync #2 (merge `45939f6`, [PR #1](https://github.com/weseegod/thanh/pull/1)):
- 1 upstream commit (`a5589e9`); 231 files, ~20.7k insertions, ~4.3k deletions
- **Zero conflicts**; 7 fork-inventory files auto-merged (non-overlapping hunks)
- Fork markers verified identical pre/post via the `git grep -n` diff
- `cargo check` passed; full `./build.sh` waived by user
- Delivered via **PR** (`merge/upstream-main` → `main`) instead of local merge + push
- Fork had 12 commits ahead of upstream at merge time

Sync #3 (merge `6f2d9d5`, direct local merge + push):
- 1 upstream commit (`393430e`, "Synced from monorepo"); 262 files, ~21.1k insertions, ~6.3k deletions
- **2 conflicts**, both resolved by combining:
  - `agent_view/render.rs`: fork keeps `turn_status::row_count` (expanded watching-cue detail rows) while adopting upstream's `wake_display_state` refactor (`wake_display_state.unwrap_or(&self.session.state)` as the state arg)
  - `acp_session_impl/goal.rs`: fork keeps preferred-model goal-evaluator logic (`effective_suggest_model` from `prompt_suggest_model_pin`, fallback to active model via `prepare_chat_completion`); upstream removed the whole preferred-model branch and the 30s timeout, so re-added `GOAL_EVALUATOR_TIMEOUT` const to `goal_evaluator.rs`
- Fork markers: 110/110 lines preserved (content identical; line numbers shifted by upstream insertions/reorderings — e.g. one `ModelByok` use line moved within `auth_error_no_retry_tests.rs` as upstream reordered tests)
- `cargo check` passed; full `./build.sh` waived by user
- Delivered via **direct merge** into `main` + push (per user request)
- Fork had 14 commits ahead of upstream at merge time

Sync #4 (merge `5fa1439`, + Cargo.lock resync `5d3b005`):
- 1 upstream commit (`afbc0fb`, "Synced from monorepo"); 74 files, ~4.7k insertions, ~0.7k deletions
- **Zero conflicts** on the merge branch; Cargo.lock resynced on `main` afterwards (shell/pager follow upstream 1.0.0; fork version crates stay 0.2.122)
- Follow-up **fork cleanup** — removed the consumer billing/paywall surface (BYOK sessions have no grok.com billing):
  - Deleted `dispatch/billing.rs`, `views/credit_bar.rs`, `scrollback/blocks/credit_limit.rs`, shell `subscription_check.rs` + `extensions/billing.rs`
  - `subscription.rs` trimmed to the server-driven gate chokepoint; `Effect::FetchBilling` / `CheckSubscription` / `GateVerify*` / `SchedulePaywallCheck` and the `credit_limit_blocked` / `free_usage_blocked` / `usage_visible` fields removed
  - Tier-restricted upsell (SuperGrok modal/toast) replaced with a terse "isn't available on your current plan" notice
- `cargo check --all-targets` clean (0 warnings); pager + shell lib tests: only **pre-existing** failures remain (paste file-URL probe, scrollback token teal, terminal-cursor tests, shell auth order-dependent flakes + one stack-overflow test — all fail identically at pre-cleanup HEAD)
- Full `./build.sh` verified (release build + install)
- Fork markers: 110/110 preserved (verified via the `git grep -n` diff pre/post)
- Delivered via **direct merge** into `main` + push (per user request)

Sync #5 (merge `de01de1`, direct local merge into `main`):
- 3 upstream commits (`75e73f3`..`be71313`, "Synced from monorepo"); 296 files, ~18.0k insertions, ~2.7k deletions
- **7 conflicts**, resolved by combining:
  - `pager-bin/main.rs`: adopt upstream `build_with_blocking_pool` + `resolve_update_trigger`; keep `thanh` branding and worker-count error text
  - `dispatch/session/fork.rs`: keep `conversation_entry` stamping; drop reintroduced `apply_credit_balance` / `credit_balance` (no consumer billing surface)
  - `dispatch/tests/task_result.rs`: keep new `ResetSessionTitle*` tests; drop `CheckSubscription` / `pending_gate_verification` billing tests
  - `slash/commands/mod.rs`: keep bare `/usage` tests (no `ManageBilling` / `billing_surface_visible`)
  - `shell/agent/app.rs`: keep `thanh login` / `thanh agent stdio` copy + orphaned-upload cleanup comment
  - `shell/config/mod.rs`: take upstream ZDR video-tools doc wording; keep `~/.thanh/managed_config.toml`
  - `update/auto_update.rs`: adopt channel-aware `reinstall_hint(installer, channel)` API + Rosetta / install-phase helpers; keep `weseegod/thanh` feed, `thanh` binary name, and fork release-page manual install hint (channel arg accepted, ignored)
- Fork markers: 113/113 preserved (content identical; line numbers shifted by upstream insertions/reorderings)
- `cargo check -p xai-grok-shell -p xai-grok-pager -p xai-grok-sampling-types -p xai-grok-update -p xai-grok-pager-bin` passed
- Full `./build.sh` not run yet (waive or run before release)
- Delivered via **direct merge** into local `main` (not pushed; push + release when requested)
- Fork had 32 commits ahead of upstream at merge time; version crates remain `1.0.1` pending post-sync bump
