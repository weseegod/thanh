<div align="center">

<h1>
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://media.x.ai/v1/website/spacexai-symbol-white-transparent-0c31957f.png">
    <source media="(prefers-color-scheme: light)" srcset="https://media.x.ai/v1/website/spacexai-symbol-black-transparent-6435cf42.png">
    <img alt="SpaceXAI logo" src="https://media.x.ai/v1/website/spacexai-symbol-black-transparent-6435cf42.png" width="96">
  </picture>
  <br>
  Grok Build (<code>grok</code>)
</h1>

**Grok Build** is SpaceXAI's terminal-based AI coding agent. It runs as a
full-screen TUI that understands your codebase, edits files, executes shell
commands, searches the web, and manages long-running tasks — interactively,
headlessly for scripting/CI, or embedded in editors via the Agent Client
Protocol (ACP).

[Installing the released binary](#installing-the-released-binary) ·
[Building from source](#building-from-source) ·
[Documentation](#documentation) ·
[Repository layout](#repository-layout) ·
[Development](#development) ·
[Contributing](#contributing) ·
[License](#license)

![Grok Build TUI](https://media.x.ai/v1/website/universe-tui-screenshot-6f7a0837.png)

**Learn more about Grok Build at [x.ai/cli](https://x.ai/cli)**

This repository contains the Rust source for the `grok` CLI/TUI and its agent
runtime. It is synced periodically from the SpaceXAI monorepo.

A small `SOURCE_REV` file at the root records the full monorepo commit SHA
for the version of the code present in this tree.

</div>

---

## About this fork (xgrok)

This repository is a **BYOK-focused fork** of upstream
[Grok Build](https://github.com/xai-org/grok-build). The goal is to leverage
the upstream agent/TUI core unchanged and customize only what third-party
models need — for example DeepSeek, OpenRouter, or any OpenAI-compatible API.

The fork ships as a command named **`thanh`** (not `grok`) and keeps its own
home directory **`~/.thanh`** (config, auth, sessions, binaries, caches) —
completely separate from the official grok CLI's `~/.grok`, so both can run
side by side without ever clobbering each other.

| Topic | Link |
|-------|------|
| BYOK model setup | [`docs/byok-models.md`](docs/byok-models.md) |
| Syncing upstream | [`UPSTREAM-MERGE.md`](UPSTREAM-MERGE.md) |
| Build from source | [`build.sh`](build.sh) → installs `thanh` into `~/.thanh/bin/thanh` (+ symlink in `~/.local/bin`) |
| Publish a release | [`scripts/publish_release.sh`](scripts/publish_release.sh) |

---

## Installing & updating `thanh`

Prebuilt binaries are published on this fork's
[GitHub Releases](https://github.com/weseegod/xgrok/releases) for **macOS
(Apple Silicon)** and **Linux (x86_64)** — built by CI, so you never need to
compile on your own machine (handy on memory-constrained Macs).

**Install the latest release:**

```sh
# macOS (Apple Silicon / M1):
curl -fsSL -o ~/.local/bin/thanh \
  https://github.com/weseegod/xgrok/releases/latest/download/thanh-0.2.122-macos-aarch64
chmod +x ~/.local/bin/thanh
# Linux (x86_64): replace the asset name with thanh-0.2.122-linux-x86_64
```

> [!NOTE]
> Assets are named `thanh-<version>-<os>-<arch>`. Check the latest version on
> the [releases page](https://github.com/weseegod/xgrok/releases).

**Auto-update (the easy path):** once installed via the managed layout
(`./build.sh` or the first update), the TUI checks for new versions in the
background. When one is available the welcome screen shows
`Update: vX available — press ctrl+u to restart` — press **Ctrl+U** to
download and restart onto the new binary. You can also run `thanh update`
manually. Official grok is unaffected: the updater manages
`~/.thanh/bin/thanh` and never touches grok's `~/.grok` tree.

> [!NOTE]
> Migrating from an earlier `xgrok` setup? Everything lived in `~/.grok`
> before; the fork now reads `~/.thanh`. Copy what you need across, e.g.:
> `mkdir -p ~/.thanh && cp ~/.grok/config.toml ~/.grok/auth.json ~/.thanh/`

**Build from source** (only if you want a local dev build):

```sh
./build.sh              # needs Rust + dotslash (see "Building from source" below)
```

## Building from source

Requirements:

- **Rust** — the toolchain is pinned by [`rust-toolchain.toml`](rust-toolchain.toml);
  `rustup` installs it automatically on first build.
- **[DotSlash](https://dotslash-cli.com)** — required so hermetic tools under
  [`bin/`](bin/) (notably [`bin/protoc`](bin/protoc)) can download and run.
  Install it and ensure `dotslash` is on your `PATH` **before** building:

  ```sh
  cargo install dotslash
  # or: prebuilt packages — https://dotslash-cli.com/docs/installation/
  /usr/bin/env dotslash --help   # sanity check
  ```

- **protoc** — proto codegen resolves [`bin/protoc`](bin/protoc) via DotSlash,
  or falls back to a `protoc` on `PATH` / `$PROTOC`.
- macOS and Linux are supported build hosts; Windows builds are best-effort
  and not currently tested from this tree.

```sh
cargo run -p xai-grok-pager-bin              # build + launch the TUI
cargo build -p xai-grok-pager-bin --release  # release binary: target/release/xai-grok-pager
cargo check -p xai-grok-pager-bin            # fast validation
```

The binary artifact is named `xai-grok-pager`; official installs ship it as
`grok`. On first launch it opens your browser to authenticate — see the
[authentication guide](crates/codegen/xai-grok-pager/docs/user-guide/02-authentication.md).

## Documentation

Full online documentation is available at
[docs.x.ai/build/overview](https://docs.x.ai/build/overview).

The user guide ships with the pager crate:
[`crates/codegen/xai-grok-pager/docs/user-guide/`](crates/codegen/xai-grok-pager/docs/user-guide/)
— getting started, keyboard shortcuts, slash commands, configuration, theming,
MCP servers, skills, plugins, hooks, headless mode, sandboxing, and more.

## Repository layout

| Path | Contents |
|------|----------|
| `crates/codegen/xai-grok-pager-bin` | Composition-root package; builds the `xai-grok-pager` binary |
| `crates/codegen/xai-grok-pager` | The TUI: scrollback, prompt, modals, rendering |
| `crates/codegen/xai-grok-shell` | Agent runtime + leader/stdio/headless entry points |
| `crates/codegen/xai-grok-tools` | Tool implementations (terminal, file edit, search, ...) |
| `crates/codegen/xai-grok-workspace` | Host filesystem, VCS, execution, checkpoints |
| `crates/codegen/...` | The rest of the CLI crate closure (config, MCP, markdown, sandbox, ...) |
| `crates/common/`, `crates/build/`, `prod/mc/` | Small shared leaf crates pulled in by the closure |
| `third_party/` | Vendored upstream source (Mermaid diagram stack) — see below |

> [!IMPORTANT]
> The root `Cargo.toml` (workspace members, dependency versions, lints,
> profiles) is **generated** — treat it as read-only. Prefer editing per-crate
> `Cargo.toml` files.

## Development

```sh
cargo check -p <crate>        # always target specific crates; full-workspace builds are slow
cargo test -p xai-grok-config # per-crate tests
cargo clippy -p <crate>       # lint config: clippy.toml at the repo root
cargo fmt --all               # rustfmt.toml at the repo root
```

## Contributing

> [!NOTE]
> External contributions are not accepted. See [`CONTRIBUTING.md`](CONTRIBUTING.md).

## License

First-party code in this repository is licensed under the **Apache License,
Version 2.0** — see [`LICENSE`](LICENSE).

Third-party and vendored code remains under its original licenses. See:

- [`THIRD-PARTY-NOTICES`](THIRD-PARTY-NOTICES) — crates.io / git dependencies,
  bundled UI themes, and **in-tree source ports** (including openai/codex and
  sst/opencode tool implementations)
- [`crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md`](crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md)
  — crate-local notice for the codex and opencode ports (license texts +
  Apache §4(b) change notice)
- [`third_party/NOTICE`](third_party/NOTICE) — vendored Mermaid-stack index
