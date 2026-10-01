# Core removal and 4.0 release — Implementation Plan (3 of 4)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Delete from turbo-vision core everything that now lives in tv-extensions (SSH, remote input, Kitty/ANSI-art graphics, the logging window, the extras crate), document the move, release 4.0.0, and point tv-extensions at it.

**Architecture:** Core loses five groups of modules plus their features, dependencies, examples and docs, in one commit per group. The code survives unchanged in tv-extensions (plan 2). The core hooks those modules now use (`event_injector`, the capture hook, `write_raw`, public `InputParser`, `Backend`) stay. Screen capture stays in core per the 2026-10-02 decision: ANSI dump always, PNG behind `screenshot`.

**Tech Stack:** Rust 2024, Cargo features, crates.io release, MkDocs site sync.

**Spec:** `docs/superpowers/specs/2026-10-01-core-extensions-split-design.md` (section "Target layout → turbo-vision core 4.0 → Removed from core", amended by "Screen capture (amended 2026-10-02)").

## Global Constraints

- Core work is on branch `feat/core-4.0` in `/Users/enzol/Code/turbo-vision-4-rust`. tv-extensions work is on its own `main` in `~/Code/tv-extensions`. Only the controller pushes or publishes.
- Removed from core, exactly:
  - `src/ssh/`, `src/terminal/ssh_backend.rs`, the `ssh` feature and its dependencies, and `examples/ssh_server.rs`;
  - `src/terminal/remote_input.rs`, `Terminal::enable_remote_input`, `Application::enable_remote_input`, and the `TV_REMOTE_KEYS` handling in `Application::new`;
  - `src/views/kitty_image.rs`, `src/views/ansi_background.rs`, `src/core/ansi.rs`, `Terminal::{write_kitty_graphics, supports_kitty_graphics, delete_kitty_image, clear_kitty_images}`, the `base64` dependency, and `examples/{kitty_image,kitty_background,kitty_biorhythm,desktop_logo}.rs` plus any data file only they use;
  - `src/views/log_window.rs`, `src/views/terminal_widget.rs`, the `SharedTerminalWidget` newtype in `src/views/shared.rs`, the `tracing` and `simplelog` dependencies, and `examples/{log_window,terminal_widget}.rs`;
  - the `extras/` workspace member.
- Kept in core: `ansi_dump`, `screenshot` (behind the feature), `chrono`, `log`, `event_injector`, the capture hook and its injected Ctrl+F12/F12 handling, `write_raw`, `InputParser`, the Black Window palette (tv-extensions' LogWindow uses it), and `WindowPaletteType`.
- After each removal commit, all of these stay green: `cargo build --all-targets` (0 warnings), `cargo test`, `cargo test --no-default-features --lib`, `cargo build --no-default-features --features native`, `sh scripts/check-wasm.sh`.
- No dangling references: after Task 1, `grep -rnE "kitty|Kitty|ansi_background|AnsiBackground|log_window|LogWindow|terminal_widget|TerminalWidget|remote_input|TV_REMOTE_KEYS|enable_remote_input|ssh|Ssh|SSH" src examples demo tests Cargo.toml` shows only intentional mentions: the Black Window palette comment, and doc lines pointing to tv-extensions.
- Version: 4.0.0. Commits end with a `Co-Authored-By:` trailer naming the model that wrote them.

## Review Focus

1. **Nothing tv-extensions needs is removed.** tv-extensions must still build against core 4.0 with every feature (Task 3 proves it).
2. **Injected Ctrl+F12/F12 still capture** through `event_injector` after `remote_input` is gone (a core test pins it).
3. **Default and no-default builds still work**, and `screenshot` off still builds.
4. **The upgrade guide tells a 3.x user exactly where each removed item went** (crate, feature, path), including the 3.1 accessor note and the `screenshot` feature.
5. **The docs site has no page describing removed APIs as core features**, and no nav entry pointing at a deleted page.

---

### Task 1: Remove the moved groups from core (one commit per group)

**Files:** as listed in the Global Constraints, plus `src/lib.rs`, `src/views/mod.rs`, `src/terminal/mod.rs`, `src/app/application.rs`, `src/core/mod.rs`, `src/views/shared.rs` and `Cargo.toml`.

**Interfaces:**
- Produces: core without the removed items. Kept APIs are unchanged.

Do the five commits in this order. Each one is a deletion plus whatever fix-ups the compiler asks for:

- [ ] **Step 1: SSH.**
  - Delete `src/ssh/` and `src/terminal/ssh_backend.rs`.
  - Remove `pub mod ssh` from `src/lib.rs`, plus its prelude and doc mentions.
  - Remove `mod ssh_backend` and its `pub use` from `src/terminal/mod.rs`.
  - Remove the `ssh` feature, the optional deps only it used (`russh`, `russh-keys`, `async-trait`, `parking_lot`, `tokio`, `rand`, `ssh-key`), `examples/ssh_server.rs` and its `[[example]]` entry.
  - Keep `InputParser` public.
  - Add a one-line doc note where SSH was documented in `src/lib.rs`: "Serving an application over SSH lives in the `tv-extensions` crate (`ssh` feature)."
  - Run the checks, then commit: `refactor!: SSH serving moves to tv-extensions`.
- [ ] **Step 2: Remote input.**
  - Delete `src/terminal/remote_input.rs`, `Terminal::enable_remote_input` and `Application::enable_remote_input`.
  - In `Application::new`, delete the `TV_REMOTE_KEYS` block. Update `Application::with_terminal`'s doc, which mentions it.
  - Keep `event_injector`, and keep the injected-capture handling in `poll_event`.
  - Write the test first (it must pass before and after the deletion): `injected_capture_chords_still_capture_without_remote_input`. It injects Ctrl+F12 through `event_injector` with a capture hook set, and asserts the hook ran. If an equivalent test already exists, name it in the report instead.
  - Update `examples/screenshot.rs`'s header, which describes `TV_REMOTE_KEYS` and `nc`: injection now comes from tv-extensions' `remote_input::enable` (feature `remote-input`).
  - Run the checks, then commit: `refactor!: remote key input moves to tv-extensions`.
- [ ] **Step 3: Graphics.**
  - Delete `src/views/kitty_image.rs`, `src/views/ansi_background.rs` and `src/core/ansi.rs`, plus their `mod`/`pub use` lines.
  - Delete the four Kitty `Terminal` methods and the `base64` dependency.
  - Delete `examples/{kitty_image,kitty_background,kitty_biorhythm,desktop_logo}.rs`. Delete `examples/logo.txt` and `examples/tv-logo.txt` only if no remaining example or test uses them (grep for both).
  - Run the checks, then commit: `refactor!: Kitty images and ANSI-art backgrounds move to tv-extensions`.
- [ ] **Step 4: Logging window.**
  - Delete `src/views/log_window.rs` and `src/views/terminal_widget.rs`, plus their `mod`/`pub use` lines.
  - Delete the `SharedTerminalWidget` newtype and its docs in `src/views/shared.rs`. Keep the rest of `Shared`.
  - Remove the `tracing` and `simplelog` dependencies. `simplelog` was used only by `ssh_server`; remove it only if nothing else uses it.
  - Delete `examples/{log_window,terminal_widget}.rs`.
  - Keep the Black Window palette. Change its comments from "for LogWindow" to "for dark-themed windows (e.g. tv-extensions' LogWindow)".
  - Run the checks, then commit: `refactor!: LogWindow and TerminalWidget move to tv-extensions`.
- [ ] **Step 5: Extras.**
  - Delete `extras/`.
  - In `Cargo.toml`, set the `[workspace]` `members` to `["."]`, or remove the table if nothing else needs it. Check `.github` workflows and scripts for `extras` and update them.
  - Run the checks, then commit: `refactor!: turbo-vision-extras retires (duplicates folded into core; the rest in tv-extensions)`.
- [ ] **Step 6: Final grep.**
  - Run the "No dangling references" grep from the Global Constraints. Fix anything left in `src/`, `examples/`, `demo/` and `tests/`; docs are Task 2.
  - Paste the grep output in the report.

---

### Task 2: Upgrade guide, changelog, docs and website

**Files:**
- Create: `UPGRADING-TO-4.0.md`
- Modify: `CHANGELOG.md`, `README.md`, `docs/*.md`, `website/sync_docs.py`, `website/mkdocs.yml`, `website/docs/whats-new.md`, `website/docs/index.md`, `website/docs/getting-started.md`, `website/docs/examples/index.md`, and the other hand-written site pages that mention removed APIs.

- [ ] **Step 1: `UPGRADING-TO-4.0.md`**, written in the plain style of `UPGRADING-TO-3.0.md`, covering:
  - **Where each item went:** a table of every removed item, one row per group, with tv-extensions module, feature, and the exact `use` line. For example, `turbo_vision::views::log_window::LogWindowBuilder` → `tv_extensions::log::LogWindowBuilder` (feature `log`).
  - **Changed calls:** `terminal.supports_kitty_graphics()` becomes `tv_extensions::graphics::kitty::supports_kitty_graphics()`, `terminal.clear_kitty_images()` becomes `clear_kitty_images(&mut terminal)`, and `TV_REMOTE_KEYS` is now read by `tv_extensions::remote_input::enable_from_env(&mut app.terminal)`.
  - **The tv-extensions dependency line** for 4.0: `tv-extensions = { git = "https://github.com/aovestdipaperino/tv-extensions" }`, with the features to enable.
  - **The `screenshot` feature:** on by default; crates using `default-features = false` add it.
  - **The 3.1 accessor change**, for anyone jumping from 3.0: owned strings, `.as_deref()`.
  - **Extras users:** `ComboBox`, `SpinControl`, `Notebook` and `Gauge` map to core `ComboBox`, `Spinner`, `TabbedPane` and `ProgressBar`; `Slider` is now in core; `GridView` maps to `Table` + `RowProvider`; `VirtualListBox` maps to `ListBox` + `ListProvider`; `ScrollPane` and `popup_menu` live in tv-extensions.
- [ ] **Step 2: `CHANGELOG.md`.** Add `## [Unreleased]` with `### Removed`: one bullet per group naming its new home, linking `UPGRADING-TO-4.0.md`.
- [ ] **Step 3: README.**
  - Remove the removed items from the feature checklist.
  - Add a short "Extensions" paragraph pointing at tv-extensions, naming its features.
  - Update the version line to 4.0.0.
  - Update the SSH section, if any, to point at tv-extensions.
- [ ] **Step 4: `docs/*.md`.**
  - Edit `RUST-API-CATALOG.md`, `MORE-CONTROLS.md`, `TURBO-VISION-DESIGN.md`, `MISSING-INHERITANCE.md` and the user guide chapters: remove the API entries for removed items, or replace them with one line pointing to tv-extensions. Don't rewrite unrelated text.
  - Delete `docs/MISSING-INHERITANCE.md.bak` (a stray backup file).
- [ ] **Step 5: Website.**
  - In `website/sync_docs.py`, map `UPGRADING-TO-4.0.md` to a new `reference/upgrading-4.0.md`, and add it to the nav in `website/mkdocs.yml` next to the 3.0 guide.
  - Run `python3 website/sync_docs.py`.
  - Add a `## 4.0.0` section at the top of `whats-new.md`, saying what moved where and linking the guide.
  - Fix hand-written pages (`index.md`, `getting-started.md`, `examples/index.md`) that list removed features or examples: point them at tv-extensions or drop them. The `ssh` feature line in `getting-started.md` becomes tv-extensions.
  - Build the site as `docs/HOW-TO-BUILD-AND-DEPLOY-WEBSITE.md` describes, then remove `website/.venv` and `website/site`. No new warnings.
- [ ] **Step 6: Commit.** Run `git status`, then commit with the message `docs: the 4.0 upgrade guide; core docs and site without the moved features`.

---

### Task 3: tv-extensions on core 4.0 (pre-release check, then repin)

**Files (tv-extensions):** `Cargo.toml`, `docs/index.md`, `README.md` and `CHANGELOG.md`.

- [ ] **Step 1: Prove it builds against the branch before release.**
  - In tv-extensions, run every check with a path override: `cargo check --all-targets --all-features --config 'patch."https://github.com/aovestdipaperino/turbo-vision-4-rust".turbo-vision.path="/Users/enzol/Code/turbo-vision-4-rust"'`. Also run `cargo test --all-features`, the per-feature loop, and the wasm check with the same override.
  - Report the result. Any failure means core removed something tv-extensions needs. Stop and report; don't paper over it.
- [ ] **Step 2: After the controller publishes 4.0.0**, change both the dependency and the dev-dependency to `turbo-vision = { version = "4.0", default-features = false }` (dev-dependency with `features = ["test-util"]`).
  - Add `screenshot = ["turbo-vision/screenshot"]` with a comment, and enable it in the native examples where useful. At minimum, `csv_edit`'s `required-features` stay `csv,native`; `screenshot` is optional.
  - Update the dependency snippet in `docs/index.md` and README to `version = "4.0"`.
  - In the CHANGELOG, record "tracks turbo-vision 4.0 from crates.io".
  - Re-run every check without an override, then commit: `build: depend on turbo-vision 4.0 from crates.io; forward the screenshot feature`.

---

### Task 4: Release (controller)

- [ ] Merge `feat/core-4.0` into `main`, and run the full checks on `main`.
- [ ] Set `Cargo.toml` to `version = "4.0.0"`, and change `## [Unreleased]` to `## [4.0.0] - <date>`. Update the README version line and the `whats-new` heading.
- [ ] Run `cargo publish --dry-run`. Commit `Release 4.0.0`, tag `v4.0.0`, and push `main` and the tag.
- [ ] Run `cargo publish`. Create the GitHub release with notes that link `UPGRADING-TO-4.0.md`.
- [ ] Run Task 3 Step 2, then push tv-extensions.

---

### Task 5: Publish tv-extensions (user request, 2026-10-01)

**Files (tv-extensions):** `logo.png` (new), `README.md`, `Cargo.toml`, `src/lib.rs`

- [ ] **Step 1: Logo.**
  - Make `logo.png` in the style of turbo-vision's own `logo.png`: a text-mode look, drawn with the real PNG renderer.
  - A small Rust example or test in tv-extensions (`examples/make_logo.rs`, `required-features = ["screenshot"]`) builds a cell buffer and writes the PNG with `turbo_vision::core::screenshot::render_to_png` at scale 2. The buffer holds a framed Turbo Vision window titled "tv-extensions" on the classic blue desktop, with the name in large block letters and a one-line tagline ("extensions for turbo-vision"). Use the CP437 glyphs the renderer now supports.
  - Keep the PNG under 100 KB. Commit the generated file and the example.
- [ ] **Step 2: README header.** Put the logo at the top the way turbo-vision's README does: `<img src="https://raw.githubusercontent.com/aovestdipaperino/tv-extensions/main/logo.png" ... align="right" />`. Use an absolute URL, because crates.io renders README images only from absolute URLs. Add crates.io and docs badges matching turbo-vision's style.
- [ ] **Step 3: Publishing metadata.**
  - In `Cargo.toml`: version `0.2.0`; `description`, `license`, `repository`, `readme`, `keywords` (at most 5) and `categories` all set; `homepage` set if a docs site exists, otherwise left out; `exclude` keeps out `website/*`, `.superpowers/*` and `logo.png`.
  - Add `[package.metadata.docs.rs] all-features = true` plus `rustdoc-args = ["--cfg", "docsrs"]`, and put `#![cfg_attr(docsrs, feature(doc_cfg))]` in `src/lib.rs` so feature-gated items are labelled on docs.rs.
  - Run `cargo package --list` (check no stray files) and `cargo publish --dry-run`.
- [ ] **Step 4 (controller): publish.**
  - Check that the crate name `tv-extensions` is free with `cargo search tv-extensions` and a crates.io API lookup. If it is taken, stop and ask the user.
  - Then `cargo publish`, tag `v0.2.0`, push, and create the GitHub release.
