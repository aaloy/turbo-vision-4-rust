# AGENTS.md

This file provides guidance to the agent when working with code in this repository.

## Build, Lint, and Test Commands

This is a Cargo workspace with two crates: the root `turbo-vision` library and the `extras` crate (`turbo-vision-extras`). Edition 2024, stable toolchain.

```bash
# Build / check
cargo build
cargo build --all-targets          # lib + bins + examples + tests
cargo check --all-targets

# Tests
cargo test                         # lib unit tests + doctests (full suite)
cargo test --lib                   # unit tests only (~575 tests)
cargo test --lib <module path>     # one module, e.g. cargo test --lib views::window
cargo test --lib <test_name> -- --nocapture

# Lint
cargo clippy --all-targets         # passes; warnings are expected and allowed
```

**Important lint gotcha:** `cargo clippy --all-targets -- -D warnings` does **not** pass. `Cargo.toml` enables a large set of clippy lints (pedantic, restriction, complexity, etc.) at `warn` level, and the codebase currently emits ~1760 warnings. Use plain `cargo clippy --all-targets` and treat warnings as advisory, not a gate. Do not "fix" the lint config or mass-suppress warnings unless asked.

### Feature-gated commands

```bash
# SSH server example only compiles with the ssh feature (gated via required-features)
cargo run --example ssh_server --features ssh
cargo build --all-targets          # without features: skips ssh_server, no tokio resolution

# Extras crate
cargo build -p turbo-vision-extras
cargo run -p turbo-vision-extras --example extras_controls
cargo run -p turbo-vision-extras --example extras_data
cargo run -p turbo-vision-extras --example extras_notebook

# Debug binary (key code inspector)
cargo run --bin key_debug
```

### Examples

```bash
cargo run --example <name>         # e.g. cargo run --example pascal_ide
cargo build --examples
```

## Architecture

- **Workspace layout:** root crate `turbo-vision` (the framework) + `extras` crate with additional controls (ComboBox, GridView, Gauge, Slider, SpinControl, Notebook, popup_menu, VirtualListBox, ScrollPane). `extras` depends on the root crate by path.
- **Modules (`src/`):**
  - `core/` — fundamental types: `geometry` (Point/Rect), `event` (Event/KeyCode), `command`/`command_set`, `palette` (colors/Attr), `draw` (Cell/Buffer), `error`, `state`, `clipboard`, `history`, `menu_data`, `status_data`, `ansi`/`ansi_dump`, `screenshot`.
  - `views/` — all widgets and containers (Window, Dialog, Desktop, Button, InputLine, Editor, ListBox, Table, TabbedPane, SplitPane, Tooltip, MenuBar, StatusLine, msgbox, etc.). Every widget implements the `View` trait.
  - `app/` — `Application`, the central coordinator (terminal, desktop, menu bar, status line, event loop, modal execution).
  - `terminal/` — `Terminal` plus the `Backend` trait abstraction. `CrosstermBackend` is the default; `SshBackend` is feature-gated. Terminal handles double-buffered rendering, clipping, event queuing, and screen capture.
  - `ssh/` — feature-gated SSH server (`SshServer`, `SshServerConfig`, `SshAuthPolicy`).
  - `helpers/` — small utility helpers (msgbox).
  - `test_util/` — `MockTerminal` and test helpers, only compiled with the `test-util` feature.
- **Event-driven model:** events flow through `handle_event` and bubble up the call stack. Child-to-parent communication is done by *event transformation* (`*event = Event::command(cmd)`), not owner pointers. There is no unsafe downcasting from `View` trait objects.
- **Command set:** a thread-local global command set (`command_set` module) with `enable_command`/`disable_command`/`command_enabled`. Buttons auto-update via the `CM_COMMAND_SET_CHANGED` broadcast from `Application::idle()`.
- **Palette:** context-aware palette chain resolved through `QCell`-based owner chains (no unsafe in the view system).
- **SSH:** async server (tokio + russh) with a synchronous TUI per connection; mpsc channels bridge the async/sync boundary. Auth is **default-deny** — `SshAuthPolicy` rejects everything unless callbacks or `allow_anonymous()` are set.
- **`prelude` module** re-exports common types (Application, View, Event, Rect, command constants). It uses explicit re-exports, not glob imports.

## Setup and Environment Variables

No special setup beyond a stable Rust toolchain. `Cargo.lock` is gitignored and not committed, so builds resolve dependencies fresh.

Runtime environment variables (all optional):

- `TV_REMOTE_KEYS` — set to a port number to enable remote key injection in `Application::new()` (used for testing/automation). Off unless set.
- `TERM`, `TERM_PROGRAM`, `KITTY_WINDOW_ID` — used by `Terminal::supports_kitty_graphics()` to detect Kitty-compatible terminals (kitty, wezterm, ghostty).

## Non-obvious Gotchas and Workflow Quirks

- **Clippy gate:** see above — `-D warnings` fails by design of the lint config.
- **`tests/` directory files are example-style binaries** (`fn main()`), not `#[test]` functions. They compile under `cargo test` but do not run as tests; the real tests are `#[cfg(test)]` modules inside `src/`.
- **Feature gating:** `ssh` feature gates the `ssh` module, `SshBackend`, and the `ssh_server` example. `test-util` gates `MockTerminal`. `cargo build --all-targets` without features intentionally skips `ssh_server` so it doesn't fail resolving tokio.
- **Deliberate deviations from Borland Turbo Vision** are documented at the bottom of `TO-DO.md` (e.g. Vec<String> buffers, multi-step undo/redo, CUA keybindings, markdown help, commands < 1000 close dialog, double-ESC cancel). Don't "fix" these to match C++ without checking that list.
- **Command IDs were renumbered to Borland values** — a breaking change; treat command constants as a contract.
- **`docs/`** contains design and implementation references (`TURBO-VISION-DESIGN.md`, `RUST-IMPLEMENTATION-REFERENCE.md`, `RUST-CODING-GUIDELINES.md`, `key-files-summary.txt`). `docs/superpowers/plans/` holds dated implementation plans. `docs/key-files-summary.txt` is partially stale (references old example filenames).
- **`MISSING-SIMPLE.md` / `MISSING-INHERITANCE.md`** are untracked analysis documents, not part of the shipped repo. Their "gate" command (`cargo test && cargo build --all-targets && cargo clippy --all-targets -- -D warnings`) is aspirational and currently fails at the clippy step.
- **Release profile** is size-optimized (`opt-level = "z"`, `lto`, `codegen-units = 1`, `panic = "abort"`, `strip`). Panics abort rather than unwind in release builds.
- **UTF-8 correctness is a recurring concern:** cursor/selection/string operations are char-indexed; byte offsets are computed at string-op boundaries. When editing text-handling code, keep char vs byte indexing straight (see `TO-DO.md` P0 items).
