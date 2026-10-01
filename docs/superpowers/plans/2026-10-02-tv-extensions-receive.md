# tv-extensions receives the moved code — Implementation Plan (2 of 4)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give `tv-extensions` working copies of every niche feature that will leave core — ScrollPane, popup menus, the CSV editor, the logging window, Kitty and ANSI-art graphics, PNG/ANSI capture, remote key input and the SSH server — each behind its own feature and built only on turbo-vision's public API and the plan-1 hooks. Retire `Grid`.

**Architecture:** One library crate, `tv-extensions` (repo `~/Code/tv-extensions`), with one module per group and one Cargo feature per group. It depends on turbo-vision `main` with `default-features = false`. Code moves as a **copy**: core keeps its originals until plan 3 deletes them. Each copied file gets its `crate::`/`super::` paths rewritten to `turbo_vision::` paths. Where it used to reach into `Terminal` or `Application`, it is rebuilt on the plan-1 hooks (`event_injector`, `set_capture_hook`, `write_raw`, public `InputParser`). The CSV editor comes from `~/Code/plank-csvedit` without its plank parts.

**Tech Stack:** Rust 2024, Cargo features, `turbo-vision` git dependency, `wasm32-wasip1` check script, `tempfile` for tests.

**Spec:** `docs/superpowers/specs/2026-10-01-core-extensions-split-design.md` (in the turbo-vision repo). Plan 1 (`docs/superpowers/plans/2026-10-01-core-absorb.md`) is merged on turbo-vision `main`.

## Global Constraints

- All work is in `~/Code/tv-extensions` unless a task says otherwise. Do not edit turbo-vision, and do not edit plank-csvedit (it is read only as the source of the copy). Do not push either repo; Task 13 asks first.
- turbo-vision dependency: `turbo-vision = { git = "https://github.com/aovestdipaperino/turbo-vision-4-rust", rev = "<REV>", default-features = false }`, where `<REV>` is `git -C ~/Code/turbo-vision-4-rust rev-parse origin/main` at the time Task 1 runs. Dev-dependency: the same git/rev with `features = ["test-util"]`.
- Feature names, exactly: `native`, `log`, `graphics`, `capture`, `remote-input`, `ssh`, `csv`. `default = []`. `remote-input` and `ssh` imply `native`. `native = ["turbo-vision/native"]`.
- `cargo build` with default features must still compile for `wasm32-wasip1` (`sh scripts/check-wasm.sh`), and so must `--features csv`.
- Dependency versions are the ones core uses today: `tracing = "0.1"`, `base64 = "0.22"`, `chrono = "0.4.45"`, `log = "0.4.33"`, `russh = "0.48"`, `russh-keys = "0.48"`, `async-trait = "0.1"`, `parking_lot = "0.12"`, `tokio = { version = "1", features = ["full"] }`, `rand = "0.8"`, `ssh-key = "0.6"`. All optional, enabled only by their feature.
- Copied files keep their behaviour and their tests. Change only what the move needs: paths, the hook-based rewrites the task names, and names the task gives.
- Every new or copied file keeps or gets the `// (C) 2026 - Enzo Lombardi` header. Copied doc comments that name core paths (`turbo_vision::views::log_window::...`) are updated to the new `tv_extensions::...` paths.
- The crate keeps `missing_docs = "warn"`, `unsafe_code = "forbid"` and clippy `pedantic` as warnings. `cargo build --all-features --all-targets` must finish with 0 warnings.
- Commits end with a `Co-Authored-By:` trailer naming the model that wrote them.

## Review Focus

1. **Children of a `ScrollPane` inside a `Dialog` draw in the dialog's colours.** Today they come out green on green, because the pane never hands its palette chain to its inner group. Pinned in Task 3 (`children_draw_in_the_owner_dialogs_colours`).
2. **A CSV file name containing `/`, `\` or `..` must not let `FsDisk` read or write outside its root directory.** Pinned in Task 5 (`fs_disk_refuses_paths_outside_its_root`).
3. **Capture with a directory that does not exist, or a second `install`.** No panic; the newest hook wins, and a failed write is logged, not fatal. Pinned in Task 9 (`a_capture_into_a_missing_directory_does_not_panic`, `installing_twice_keeps_one_hook`).
4. **Remote input on a port already in use, or a garbage `TV_REMOTE_KEYS`.** `enable` returns the bind error, and `enable_from_env` returns `Ok(false)` for an unset or unparsable value. Pinned in Task 10 (`enable_reports_a_port_in_use`, `enable_from_env_ignores_unset_and_garbage`).
5. **The editor's screen after the move matches what plank showed.** `Session::buffer()` gives the same cells `cells()` used to, so the copied editor tests that read the screen still pass unchanged. Pinned in Task 5 (all copied `editor` tests).

---

## File Structure

| File (in ~/Code/tv-extensions) | Change | Responsibility |
|---|---|---|
| `Cargo.toml` | Modify | rev pin, features, optional deps, dev-deps, `[[example]]` required-features |
| `scripts/check-wasm.sh` | Modify | also check `--features csv` |
| `src/lib.rs` | Modify | module list with `#[cfg(feature = ...)]` gates, crate docs |
| `src/grid.rs`, `tests/grid.rs`, `docs/grid.md` | Delete | Grid retired (core `Table::set_separators`) |
| `src/scroll_pane.rs` | Create (copy) | `ScrollPane` from extras, palette fix |
| `src/popup_menu.rs` | Create (copy) | `popup_menu`, check-mark helpers from extras |
| `src/keys.rs` | Create (copy) | `translate`, `is_ctrl` key-name helpers from plank-csvedit |
| `src/csv/mod.rs`, `src/csv/{format,doc,disk,commands,dialogs,editor}.rs` | Create (copy) | CSV editor from plank-csvedit, plank-free |
| `src/log/mod.rs`, `src/log/{terminal_widget,log_window}.rs` | Create (copy) | logging window from core |
| `src/graphics/mod.rs`, `src/graphics/{ansi,ansi_background,kitty,kitty_image}.rs` | Create (copy) | ANSI art and Kitty images from core |
| `src/capture/mod.rs`, `src/capture/{png,ansi_dump}.rs`, `src/capture/font8x16.bin`, `LICENSE-Spleen` | Create (copy) | PNG and ANSI capture from core, installed on the capture hook |
| `src/remote_input.rs` | Create (copy) | TCP key injection from core, built on `event_injector` |
| `src/ssh/mod.rs`, `src/ssh/{backend,handler,server}.rs` | Create (copy) | SSH server and backend from core |
| `examples/*.rs` | Create (copy) | the examples that move (Task 12) |
| `README.md`, `docs/*.md`, `CHANGELOG.md` | Modify/Create | docs for every module |

Path rewrite table used by every copy task (apply to `use` lines, doc links and fully qualified paths):

| In the core/extras/csvedit original | In tv-extensions |
|---|---|
| `crate::core::…` | `turbo_vision::core::…` |
| `crate::terminal::…` | `turbo_vision::terminal::…` |
| `crate::app::…` | `turbo_vision::app::…` |
| `crate::views::…` (core files) | `turbo_vision::views::…` |
| `super::view::…` / `super::group::…` / `super::window::…` / `super::scrollbar::…` / `super::shared::…` (core view files) | `turbo_vision::views::view::…` / `…::group::…` / `…::window::…` / `…::scrollbar::…` / `…::shared::…` |
| `crate::test_util::test_terminal` | `turbo_vision::test_util::test_terminal` |
| `crate::impl_view_for_window!` | `turbo_vision::impl_view_for_window!` |
| `super::draw::Cell`, `super::palette::…` (core `core/` files) | `turbo_vision::core::draw::Cell`, `turbo_vision::core::palette::…` |
| `tv_extensions::…` (csvedit) | `crate::…` |

---

### Task 1: Pin core main, declare features, keep the crate green

**Files:**
- Modify: `Cargo.toml`, `scripts/check-wasm.sh`, `src/lib.rs`

**Interfaces:**
- Produces: the features listed in Global Constraints, and dev-dependency access to `turbo_vision::test_util`.

- [ ] **Step 1: Record the rev and update the manifest**

Run: `REV=$(git -C ~/Code/turbo-vision-4-rust rev-parse origin/main); echo $REV`

Replace the `[dependencies]` section of `Cargo.toml` and add the rest, so it reads (with `<REV>` replaced by the printed value):

```toml
[dependencies]
# Turbo Vision without its terminal by default (`native` off), so the crate
# builds for wasm32-wasip1. Pinned to a core `main` commit that carries the
# plan-1 hooks until a release does.
turbo-vision = { git = "https://github.com/aovestdipaperino/turbo-vision-4-rust", rev = "<REV>", default-features = false }
tracing = { version = "0.1", optional = true }
base64 = { version = "0.22", optional = true }
chrono = { version = "0.4.45", optional = true }
log = { version = "0.4.33", optional = true }
russh = { version = "0.48", optional = true }
russh-keys = { version = "0.48", optional = true }
async-trait = { version = "0.1", optional = true }
parking_lot = { version = "0.12", optional = true }
tokio = { version = "1", features = ["full"], optional = true }
rand = { version = "0.8", optional = true }
ssh-key = { version = "0.6", optional = true }

[dev-dependencies]
turbo-vision = { git = "https://github.com/aovestdipaperino/turbo-vision-4-rust", rev = "<REV>", default-features = false, features = ["test-util"] }
tempfile = "3"

[features]
default = []
# A real terminal (crossterm, OS clipboard) in turbo-vision.
native = ["turbo-vision/native"]
# LogWindow and TerminalWidget, with a tracing subscriber.
log = ["dep:tracing"]
# Kitty graphics images and ANSI-art backgrounds.
graphics = ["dep:base64"]
# Ctrl+F12 PNG and F12 ANSI screen captures.
capture = ["dep:chrono", "dep:log"]
# Key chords injected over TCP, for automation.
remote-input = ["native", "dep:log"]
# Serving an application over SSH.
ssh = ["native", "dep:russh", "dep:russh-keys", "dep:async-trait", "dep:parking_lot", "dep:tokio", "dep:rand", "dep:ssh-key"]
# The CSV table editor.
csv = []
```

- [ ] **Step 2: Extend the wasm check**

Append to `scripts/check-wasm.sh`, after the existing `cargo check` line:

```sh
cargo check --lib --features csv --target wasm32-wasip1
```

- [ ] **Step 3: Verify**

Run: `cargo test 2>&1 | tail -3`, then `cargo build --all-targets 2>&1 | grep -c warning`, then `sh scripts/check-wasm.sh`.
Expected: the existing host and grid tests pass, 0 warnings, and the wasm check finishes. (The optional deps are unused until later tasks, so they compile only when their feature is on.)

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock scripts/check-wasm.sh
git commit -m "build: track turbo-vision main; declare one feature per extension group"
```

---

### Task 2: Retire Grid

**Files:**
- Delete: `src/grid.rs`, `tests/grid.rs`, `docs/grid.md`
- Modify: `src/lib.rs` (drop `pub mod grid;` and `pub use grid::Grid;`; update the crate doc list), `README.md` (replace the "A grid with separators" section with two sentences: Grid was retired; use core `Table::set_separators(true)`, which draws the same separators in each line's own colour), `docs/index.md` (drop the Grid link if present)

**Interfaces:**
- Produces: no `tv_extensions::Grid`. Later tasks use `turbo_vision::views::table::Table` with `set_separators(true)`.

- [ ] **Step 1: Delete and edit as listed**

- [ ] **Step 2: Verify**

Run: `cargo test 2>&1 | tail -3` and `cargo build --all-targets 2>&1 | grep -E "^(warning|error)" | head`
Expected: host tests pass; nothing references `grid`. Run `grep -rn "Grid" src docs README.md` and expect only the README retirement note.

- [ ] **Step 3: Commit**

```bash
git add -A src docs README.md tests
git commit -m "refactor: retire Grid; core Table draws separators itself"
```

---

### Task 3: ScrollPane and popup menus

**Files:**
- Create: `src/scroll_pane.rs` (copy of `~/Code/turbo-vision-4-rust/extras/src/scroll_pane.rs`), `src/popup_menu.rs` (copy of `…/extras/src/popup_menu.rs`)
- Modify: `src/lib.rs`

**Interfaces:**
- Produces: `tv_extensions::scroll_pane::ScrollPane` (same public API as extras: `new(bounds, virtual_height: i16)`, `add(Box<dyn View>, virtual_rect: Rect) -> ViewId`, `group()`, `group_mut()`, plus whatever else extras exposes), and `tv_extensions::popup_menu::{popup_menu, set_menu_item_checked, is_menu_item_checked}`. Re-exported at the crate root as `tv_extensions::{ScrollPane, popup_menu, set_menu_item_checked, is_menu_item_checked}`. Both are always built (no feature).

- [ ] **Step 1: Copy both files and register them**

The extras files already use `turbo_vision::` paths, so no rewrite is needed. Change the file header to the `(C) 2026` line if it differs. In `src/lib.rs` add `pub mod popup_menu;`, `pub mod scroll_pane;`, the re-exports, and a bullet for each in the crate docs.

- [ ] **Step 2: Write the failing palette test**

Append to the copied tests in `src/scroll_pane.rs` (create `#[cfg(test)] mod tests` if extras had none):

```rust
    #[test]
    fn children_draw_in_the_owner_dialogs_colours() {
        use turbo_vision::views::GroupLike;
        use turbo_vision::views::dialog::Dialog;
        use turbo_vision::views::static_text::StaticText;

        // The same label drawn straight in a dialog and inside a ScrollPane
        // in that dialog must come out in the same colours.
        let mut plain = Dialog::new(Rect::new(0, 0, 30, 8), "");
        plain.add(StaticText::new(Rect::new(2, 2, 12, 3), "Label"));
        let mut term = turbo_vision::test_util::test_terminal(30, 8);
        plain.draw(&mut term);
        let expected = term.read_cell(2, 2).unwrap().attr;

        let mut dialog = Dialog::new(Rect::new(0, 0, 30, 8), "");
        let mut pane = ScrollPane::new(Rect::new(2, 2, 20, 6), 10);
        pane.add(
            Box::new(StaticText::new(Rect::new(0, 0, 10, 1), "Label")),
            Rect::new(0, 0, 10, 1),
        );
        dialog.add(pane);
        let mut term = turbo_vision::test_util::test_terminal(30, 8);
        dialog.draw(&mut term);
        assert_eq!(term.read_cell(2, 2).unwrap().ch, 'L');
        assert_eq!(term.read_cell(2, 2).unwrap().attr, expected);
    }
```

Run: `cargo test --lib scroll_pane 2>&1 | tail -5`
Expected: FAIL on the attribute assertion (the pane's children resolve colours without the dialog in their chain). If it already passes, report that, keep the test, and skip Step 3.

- [ ] **Step 3: Fix the palette chain**

In `ScrollPane::draw`, before it draws its inner group, hand the group the pane's own chain node, the way `Group::draw` hands one to each child (`turbo_vision/src/views/group.rs`, the `PaletteChainNode::new(self.get_palette(), self.get_palette_chain().cloned())` block):

```rust
        let node = turbo_vision::core::palette_chain::PaletteChainNode::new(
            self.get_palette(),
            self.palette_chain.clone(),
        );
        self.group.set_palette_chain(Some(node));
```

Read `draw` first: put this immediately before the line that calls `self.group.draw(terminal)`. If `ScrollPane::get_palette` returns `None`, that is correct, because the pane is transparent and carries the parent link.

- [ ] **Step 4: Run all tests and the build**

Run: `cargo test 2>&1 | tail -3` and `cargo build --all-targets 2>&1 | grep -c warning`
Expected: the copied extras tests and the new test pass; 0 warnings.

- [ ] **Step 5: Commit**

```bash
git add src/scroll_pane.rs src/popup_menu.rs src/lib.rs
git commit -m "feat: ScrollPane and popup menus, from turbo-vision-extras

ScrollPane now hands its palette chain to its inner group, so children
draw in their owner's colours.

Source: turbo-vision-4-rust extras/src/{scroll_pane,popup_menu}.rs at <REV>."
```

---

### Task 4: Key-name helpers

**Files:**
- Create: `src/keys.rs` (from `~/Code/plank-csvedit/src/keys.rs`: copy `translate`, `is_ctrl` and their tests; leave out `payload_text` and its test, which are plank's JSON payload format)
- Modify: `src/lib.rs` (`pub mod keys;`, docs bullet)

**Interfaces:**
- Produces: `tv_extensions::keys::translate(code: &str, text: Option<char>) -> Option<Event>` and `tv_extensions::keys::is_ctrl(ev: &Event) -> bool`, unchanged from csvedit. Always built. Task 5's editor uses `is_ctrl`, and its tests use `translate`. plank-tv (plan 4) uses both.

- [ ] **Step 1: Copy, rewrite the module doc**

The module doc becomes: `//! Key events from key names such as "ctrl-s" or "enter", the form a host that is not a terminal (a web page, a WASM host) reports keys in.` Keep the rest of the doc that describes the `code`/`text` contract.

- [ ] **Step 2: Run the copied tests**

Run: `cargo test --lib keys 2>&1 | tail -3`
Expected: the copied tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/keys.rs src/lib.rs
git commit -m "feat: key events from key names, from plank-csvedit"
```

---

### Task 5: The CSV editor

**Files:**
- Create: `src/csv/mod.rs`; `src/csv/format.rs` (copy of csvedit `src/csv.rs`); `src/csv/doc.rs` (copy of `src/doc.rs`); `src/csv/disk.rs` (copy of `src/disk.rs` without `PlankDisk` and its `use extism_pdk`, plus a new `FsDisk`); `src/csv/commands.rs`; `src/csv/dialogs.rs`; `src/csv/editor.rs` (copy of `src/editor.rs`, changes below)
- Modify: `src/lib.rs` (`#[cfg(feature = "csv")] pub mod csv;`)

**Interfaces:**
- Consumes: Task 4's `crate::keys::{is_ctrl, translate}`; turbo-vision `Table::set_separators`, `Table` replacing `Grid`.
- Produces:
  - `tv_extensions::csv::format::{Parsed, parse, write}`
  - `tv_extensions::csv::doc::CsvDoc`
  - `tv_extensions::csv::disk::{Disk, MemDisk, FsDisk}` with `FsDisk::new(root: impl Into<PathBuf>) -> FsDisk`
  - `tv_extensions::csv::commands::*` (the `CMD_*` constants)
  - `tv_extensions::csv::editor::{Session, OPEN_DIALOG_ARG}`, where `Session::open(w, h, arg, disk)`, `key(Event) -> Option<String>`, `step(w, h)`, `close_line()`, `doc()` and `into_disk()` are unchanged, and the new `Session::buffer(&self) -> &[Vec<turbo_vision::core::draw::Cell>]` replaces `cells()`.
  - Re-export in `csv/mod.rs`: `pub use editor::{Session, OPEN_DIALOG_ARG}; pub use doc::CsvDoc; pub use disk::{Disk, MemDisk, FsDisk};`

- [ ] **Step 1: Copy the pure modules and run their tests**

Copy `csv.rs` → `csv/format.rs`, `doc.rs`, `commands.rs` and `dialogs.rs` with `crate::csv::` → `crate::csv::format::`, other `crate::X` → `crate::csv::X`, and `crate::keys` → `crate::keys`. Write `src/csv/mod.rs`:

```rust
// (C) 2026 - Enzo Lombardi

//! A CSV table editor: a [`Session`] holds one document in a window with a
//! menu bar, a status line and dialogs for editing cells, columns and rows.
//!
//! The session is host-driven (see [`crate::host`]): the caller pushes one key
//! with [`Session::key`], asks for a frame with [`Session::step`], and reads
//! the finished cells from [`Session::buffer`]. Documents live on a
//! [`Disk`]: [`FsDisk`] for a directory, [`MemDisk`] for tests.

pub mod commands;
mod dialogs;
pub mod disk;
pub mod doc;
pub mod editor;
pub mod format;

pub use disk::{Disk, FsDisk, MemDisk};
pub use doc::CsvDoc;
pub use editor::{OPEN_DIALOG_ARG, Session};
```

Run: `cargo test --lib --features csv csv::format csv::doc csv::commands 2>&1 | tail -3`
Expected: the copied tests pass. (Comment out `pub mod editor;` and its re-export until Step 3 if they don't compile yet.)

- [ ] **Step 2: Disk, FsDisk and its tests**

In `src/csv/disk.rs`, drop `PlankDisk` and the `extism_pdk` import, change the module doc's first line to `//! Where documents are saved: a directory, or a map in tests.`, and add:

```rust
/// A directory on the local file system. Paths are file names at its root;
/// a name that is empty, contains a path separator, or is `..` is refused,
/// so the editor can never reach outside `root`.
#[derive(Debug, Clone)]
pub struct FsDisk {
    root: std::path::PathBuf,
}

impl FsDisk {
    /// A disk over the directory `root`, which must already exist.
    pub fn new(root: impl Into<std::path::PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn file(&self, path: &str) -> Result<std::path::PathBuf, String> {
        let name = path.trim_start_matches('/');
        if name.is_empty() || name == ".." || name.contains(['/', '\\']) {
            return Err(format!("not a file name: {path}"));
        }
        Ok(self.root.join(name))
    }
}

impl Disk for FsDisk {
    fn read(&self, path: &str) -> Result<String, String> {
        std::fs::read_to_string(self.file(path)?).map_err(|e| format!("{path}: {e}"))
    }
    fn write(&mut self, path: &str, text: &str) -> Result<(), String> {
        std::fs::write(self.file(path)?, text).map_err(|e| format!("{path}: {e}"))
    }
    fn list(&self) -> Result<Vec<String>, String> {
        let mut names: Vec<String> = std::fs::read_dir(&self.root)
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .filter(|e| e.path().is_file())
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|n| n.ends_with(".csv"))
            .collect();
        names.sort();
        Ok(names)
    }
}
```

Read csvedit's `MemDisk::read` error text and `list` contract first. If `MemDisk` strips a leading `/` or formats errors differently, match it so the editor's messages read the same for both disks.

Add tests in `disk.rs`:

```rust
    #[test]
    fn fs_disk_round_trips_and_lists_only_csv_files() {
        let dir = tempfile::tempdir().unwrap();
        let mut disk = FsDisk::new(dir.path());
        disk.write("b.csv", "x\n").unwrap();
        disk.write("a.csv", "y\n").unwrap();
        std::fs::write(dir.path().join("notes.txt"), "z").unwrap();
        assert_eq!(disk.read("a.csv").unwrap(), "y\n");
        assert_eq!(disk.list().unwrap(), vec!["a.csv", "b.csv"]);
    }

    #[test]
    fn fs_disk_refuses_paths_outside_its_root() {
        let dir = tempfile::tempdir().unwrap();
        let inner = dir.path().join("inner");
        std::fs::create_dir(&inner).unwrap();
        std::fs::write(dir.path().join("secret.csv"), "s").unwrap();
        let mut disk = FsDisk::new(&inner);
        for bad in ["../secret.csv", "..", "a/b.csv", "a\\b.csv", ""] {
            assert!(disk.read(bad).is_err(), "read {bad:?}");
            assert!(disk.write(bad, "x").is_err(), "write {bad:?}");
        }
        assert_eq!(std::fs::read_to_string(dir.path().join("secret.csv")).unwrap(), "s");
    }
```

Run: `cargo test --lib --features csv csv::disk 2>&1 | tail -3`
Expected: pass.

- [ ] **Step 3: The editor**

Copy `editor.rs` with these changes:
- `use tv_extensions::grid::Grid;` → `use turbo_vision::views::table::Table;`. Every `Grid` type → `Table`. `Grid::new(table_r, CMD_EDIT_CELL)` → `{ let mut t = Table::new(table_r, CMD_EDIT_CELL); t.set_separators(true); t }`. `Handle<Grid>` → `Handle<Table>`.
- `tv_extensions::host::…` → `crate::host::…`, and other `crate::X` → `crate::csv::X` (except `crate::keys`).
- Replace `pub fn cells(&self) -> Vec<plank_guest_support::CellGlyph>` with:

```rust
    /// The last frame, every cell, row by row.
    #[must_use]
    pub fn buffer(&self) -> &[Vec<turbo_vision::core::draw::Cell>] {
        self.app.terminal.buffer()
    }
```

- The module doc's mention of plank becomes host-neutral: "A host pushes one key per call and asks for a screen per step, so…". Keep the rest.
- `Grid` forwarded `selected_cell()` as `Option<&str>`; core `Table::selected_cell` returns `Option<String>`. Adjust the call sites (`.as_deref()` where a `&str` is compared, or use the `String`).
- Tests: the copied `screen(&s)` helper builds rows from `s.cells()` glyphs. Rewrite it over `s.buffer()`:

```rust
    fn screen(s: &Session) -> String {
        s.buffer()
            .iter()
            .map(|row| row.iter().map(|c| c.ch).collect::<String>().trim_end().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }
```

Check how the original helper joined rows and trimmed, and match it, so the copied assertions that search the screen keep their meaning. Keep `use crate::keys::translate;` in the tests. Copy every editor test.

Run: `cargo test --lib --features csv csv 2>&1 | tail -3`
Expected: every copied editor test passes, plus the format/doc/disk tests.

- [ ] **Step 4: Full checks**

Run: `cargo test --all-features 2>&1 | tail -3`, `cargo build --all-features --all-targets 2>&1 | grep -c warning`, `sh scripts/check-wasm.sh`.
Expected: pass, 0 warnings, wasm ok for both default and `csv`.

- [ ] **Step 5: Commit**

```bash
git add src/csv src/lib.rs
git commit -m "feat(csv): the CSV table editor, from plank-csvedit without plank

Session::buffer replaces the plank-specific cells(); Grid becomes core
Table with separators; FsDisk edits a directory.

Source: plank-csvedit src/{csv,doc,disk,commands,dialogs,editor}.rs at 0ad4e59."
```

---

### Task 6: A native CSV editor example

**Files:**
- Create: `examples/csv_edit.rs`
- Modify: `Cargo.toml` (add `[[example]] name = "csv_edit" required-features = ["csv", "native"]`)

**Interfaces:**
- Consumes: `tv_extensions::csv::{Session, FsDisk}`, `turbo_vision::terminal::Terminal` (crossterm backend via `Terminal::init()`), `Terminal::poll_event`, `Terminal::write_cell`, `Terminal::flush`, `Terminal::size`.

- [ ] **Step 1: Write the example**

```rust
// (C) 2026 - Enzo Lombardi
// CSV editor on the local terminal.
//
// Drives a host-driven csv::Session from a real terminal: each key goes to
// the session, each frame is copied cell by cell to the screen.
//
// Run with:
//   cargo run --example csv_edit --features csv,native -- [DIR] [FILE.csv]
// DIR defaults to the current directory; with no FILE a new document opens.

use std::time::Duration;
use turbo_vision::terminal::Terminal;
use tv_extensions::csv::{FsDisk, Session};

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let dir = args.next().unwrap_or_else(|| ".".into());
    let file = args.next().unwrap_or_default();

    let mut terminal = Terminal::init().map_err(std::io::Error::other)?;
    let (w, h) = terminal.size();
    let mut session = Session::open(w, h, &file, Box::new(FsDisk::new(dir)));

    let result = (|| -> std::io::Result<()> {
        loop {
            let (w, h) = terminal.size();
            session.step(w, h);
            for (y, row) in session.buffer().iter().enumerate() {
                for (x, cell) in row.iter().enumerate() {
                    terminal.write_cell(x as i16, y as i16, *cell);
                }
            }
            terminal.flush()?;
            if let Some(event) = terminal.poll_event(Duration::from_millis(50))? {
                if let Some(line) = session.key(event) {
                    println!("{line}");
                    return Ok(());
                }
            }
        }
    })();
    terminal.shutdown().map_err(std::io::Error::other)?;
    result
}
```

Check the real signatures before compiling: `Terminal::init()` and `shutdown()` may return the crate's own `Result`, `write_cell` takes `(i16, i16, Cell)`, and `poll_event` returns `io::Result<Option<Event>>`. Adjust the error conversions to the real types; keep the structure. If `shutdown` is named differently, use the method `examples/` in turbo-vision use to restore the terminal. The `println!` must come after the terminal is restored: move it after `shutdown` if needed.

- [ ] **Step 2: Build it**

Run: `cargo build --example csv_edit --features csv,native 2>&1 | grep -E "^(warning|error)" | head`
Expected: nothing.

- [ ] **Step 3: Commit**

```bash
git add examples/csv_edit.rs Cargo.toml
git commit -m "example: csv_edit, the CSV editor on a real terminal"
```

---

### Task 7: The logging window

**Files:**
- Create: `src/log/mod.rs`, `src/log/terminal_widget.rs` (copy of `~/Code/turbo-vision-4-rust/src/views/terminal_widget.rs`), `src/log/log_window.rs` (copy of `…/src/views/log_window.rs`)
- Modify: `src/lib.rs` (`#[cfg(feature = "log")] pub mod log;`)

**Interfaces:**
- Produces: `tv_extensions::log::{TerminalWidget, TerminalWidgetBuilder, Span, OutputLine, LogWindow, LogWindowBuilder, LogSubscriber}`. These are the same public items as the core files; re-export whatever else they made public.

- [ ] **Step 1: Copy with the rewrite table**

`super::terminal_widget::TerminalWidget` → `super::terminal_widget::TerminalWidget` (now a sibling in `log/`). The `crate::impl_view_for_window` macro → `turbo_vision::impl_view_for_window`. Read the macro (`turbo-vision src/views/window.rs`, `macro_rules! impl_view_for_window`). If it expands to `crate::` paths rather than `$crate::`, it will not work from outside turbo-vision. In that case, write the `impl View` for `LogWindow` by hand, delegating exactly as the macro does, and note it in the report. `src/log/mod.rs`:

```rust
// (C) 2026 - Enzo Lombardi

//! A scrolling output pane ([`TerminalWidget`]) and a window that shows
//! `tracing` events in it ([`LogWindow`], [`LogSubscriber`]).

mod log_window;
mod terminal_widget;

pub use log_window::*;
pub use terminal_widget::*;
```

Replace the `*` re-exports with explicit lists if clippy pedantic (`wildcard_imports`) warns.

- [ ] **Step 2: Run the copied tests**

Run: `cargo test --lib --features log log:: 2>&1 | tail -3`
Expected: pass.

- [ ] **Step 3: Full checks and commit**

Run: `cargo build --all-features --all-targets 2>&1 | grep -c warning` → 0.

```bash
git add src/log src/lib.rs
git commit -m "feat(log): LogWindow and TerminalWidget, from turbo-vision core

Source: turbo-vision-4-rust src/views/{log_window,terminal_widget}.rs at <REV>."
```

---

### Task 8: Graphics — ANSI art and Kitty images

**Files:**
- Create: `src/graphics/mod.rs`; `src/graphics/ansi.rs` (copy of core `src/core/ansi.rs`); `src/graphics/ansi_background.rs` (copy of core `src/views/ansi_background.rs`); `src/graphics/kitty.rs` (new: protocol helpers); `src/graphics/kitty_image.rs` (copy of core `src/views/kitty_image.rs`)
- Modify: `src/lib.rs` (`#[cfg(feature = "graphics")] pub mod graphics;`)

**Interfaces:**
- Consumes: `Terminal::write_raw(&mut self, &[u8]) -> io::Result<()>`.
- Produces: `tv_extensions::graphics::{ansi::{AnsiImage, AnsiParser /* and the rest of ansi.rs's public items */}, AnsiBackground, KittyImage, kitty::{supports_kitty_graphics, delete_kitty_image, clear_kitty_images}}`, where `supports_kitty_graphics() -> bool`, `delete_kitty_image(terminal: &mut Terminal, image_id: u32) -> io::Result<()>` and `clear_kitty_images(terminal: &mut Terminal) -> io::Result<()>`.

- [ ] **Step 1: Kitty helpers**

Create `src/graphics/kitty.rs` by porting core `Terminal::supports_kitty_graphics`, `delete_kitty_image` and `clear_kitty_images` (turbo-vision `src/terminal/mod.rs`, near lines 940-1000) to free functions. Each `self.write_kitty_graphics(x)` becomes `terminal.write_raw(x)`. `supports_kitty_graphics` reads only environment variables, so it takes no terminal. Copy their doc comments, and add tests for the escape sequences using the core terminal tests' `RecordingBackend` pattern (a `Backend` impl in the test module that records `write_raw` bytes; write your own small one):

```rust
    #[test]
    fn delete_kitty_image_sends_the_delete_command() {
        let (mut terminal, written) = recording_terminal();
        delete_kitty_image(&mut terminal, 7).unwrap();
        let sent = String::from_utf8(written.lock().unwrap().clone()).unwrap();
        assert!(sent.contains("a=d") && sent.contains("i=7"), "{sent:?}");
    }
```

Match the assertion to the exact sequence the core function sends.

- [ ] **Step 2: Copy ansi, AnsiBackground, KittyImage**

Apply the rewrite table. In `kitty_image.rs`, `terminal.write_kitty_graphics(&seq)` becomes `terminal.write_raw(&seq)` (three call sites). `ansi_background.rs`'s `crate::core::ansi::AnsiImage` becomes `super::ansi::AnsiImage`. `src/graphics/mod.rs`:

```rust
// (C) 2026 - Enzo Lombardi

//! Pictures in a text UI: ANSI-art backgrounds ([`AnsiBackground`], parsed by
//! [`ansi`]) and bitmap images over the Kitty graphics protocol
//! ([`KittyImage`], with the protocol helpers in [`kitty`]).

pub mod ansi;
mod ansi_background;
pub mod kitty;
mod kitty_image;

pub use ansi_background::AnsiBackground;
pub use kitty_image::KittyImage;
```

If either view file exposes more public items, re-export them too.

- [ ] **Step 3: Run tests, checks, commit**

Run: `cargo test --lib --features graphics graphics 2>&1 | tail -3` (pass) and `cargo build --all-features --all-targets 2>&1 | grep -c warning` (0).

```bash
git add src/graphics src/lib.rs
git commit -m "feat(graphics): ANSI art and Kitty images, on Terminal::write_raw

Source: turbo-vision-4-rust src/core/ansi.rs, src/views/{ansi_background,kitty_image}.rs at <REV>."
```

---

### Task 9: Screen capture on the capture hook

**Files:**
- Create: `src/capture/mod.rs`; `src/capture/png.rs` (copy of core `src/core/screenshot.rs`); `src/capture/font8x16.bin` (copy of core `src/core/font8x16.bin`); `src/capture/ansi_dump.rs` (copy of core `src/core/ansi_dump.rs`); `LICENSE-Spleen` (copy of core `fonts/Spleen-LICENSE`)
- Modify: `src/lib.rs` (`#[cfg(feature = "capture")] pub mod capture;`), `Cargo.toml` (`include` or nothing; `include_bytes!` reaches the font as long as it is in the package)

**Interfaces:**
- Consumes: `Terminal::set_capture_hook(Box<dyn FnMut(CaptureKind, &Terminal) + Send>)`, `CaptureKind::{Png, Ansi}`, `Terminal::buffer()`, `Terminal::size()`, `Terminal::query_font_pixel_size()` (associated fn, returns `None` without `native`).
- Produces: `tv_extensions::capture::{install(terminal: &mut Terminal), install_in(terminal: &mut Terminal, dir: impl Into<PathBuf>), save_png(terminal: &Terminal, path: &Path) -> io::Result<()>, save_ansi(terminal: &Terminal, path: &Path) -> io::Result<()>, png::{render_to_png, GLYPH_WIDTH, GLYPH_HEIGHT}, ansi_dump::*}`.

- [ ] **Step 1: Copy the renderer and dump with their tests**

Apply the rewrite table. `screenshot.rs`'s `include_bytes!("font8x16.bin")` keeps working because the font sits next to `png.rs`; fix the doc pointer to the licence (`LICENSE-Spleen` at the crate root).

Run: `cargo test --lib --features capture capture 2>&1 | tail -3`
Expected: the copied renderer and dump tests pass.

- [ ] **Step 2: Write the failing install tests**

In `src/capture/mod.rs` tests:

```rust
    #[test]
    fn a_png_capture_writes_a_png_of_the_screen_size() {
        let dir = tempfile::tempdir().unwrap();
        let mut terminal = turbo_vision::test_util::test_terminal(20, 5);
        install_in(&mut terminal, dir.path());
        assert!(terminal.run_capture_hook(CaptureKind::Png));
        let pngs: Vec<_> = std::fs::read_dir(dir.path()).unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().ends_with(".png"))
            .collect();
        assert_eq!(pngs.len(), 1);
        let bytes = std::fs::read(pngs[0].path()).unwrap();
        assert_eq!(&bytes[1..4], b"PNG");
        // IHDR width and height, big-endian, at bytes 16..24.
        let w = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
        let h = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
        assert_eq!((w, h), (20 * png::GLYPH_WIDTH as u32, 5 * png::GLYPH_HEIGHT as u32));
    }

    #[test]
    fn an_ansi_capture_writes_an_ans_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut terminal = turbo_vision::test_util::test_terminal(20, 5);
        install_in(&mut terminal, dir.path());
        assert!(terminal.run_capture_hook(CaptureKind::Ansi));
        let names: Vec<String> = std::fs::read_dir(dir.path()).unwrap()
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert!(names.iter().any(|n| n.starts_with("screen-") && n.ends_with(".ans")), "{names:?}");
    }

    #[test]
    fn a_capture_into_a_missing_directory_does_not_panic() {
        let mut terminal = turbo_vision::test_util::test_terminal(20, 5);
        install_in(&mut terminal, "/nonexistent/tv-extensions-capture-test");
        assert!(terminal.run_capture_hook(CaptureKind::Png));
    }

    #[test]
    fn installing_twice_keeps_one_hook() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        let mut terminal = turbo_vision::test_util::test_terminal(20, 5);
        install_in(&mut terminal, first.path());
        install_in(&mut terminal, second.path());
        terminal.run_capture_hook(CaptureKind::Png);
        assert_eq!(std::fs::read_dir(first.path()).unwrap().count(), 0);
        assert_eq!(std::fs::read_dir(second.path()).unwrap().count(), 1);
    }
```

Use the real names of the glyph-size constants in `png.rs` (core's `screenshot.rs` exports `GLYPH_HEIGHT`; check whether a width constant is public, and if not, compare against the `FONT_W`-based value the module exposes, or make the width `pub const GLYPH_WIDTH` in the copy and say so in the report). The test terminal reports no font pixel size, so the scale is 1.

Run: `cargo test --lib --features capture capture 2>&1 | tail -5`
Expected: FAIL, `install_in` not found.

- [ ] **Step 3: Implement install**

```rust
// (C) 2026 - Enzo Lombardi

//! Screen captures: Ctrl+F12 saves a PNG of the screen and F12 an ANSI text
//! dump, once [`install`] puts the capture hook on a terminal. Files are named
//! `screenshot-YYYYMMDD-HHMMSS.png` and `screen-YYYYMMDD-HHMMSS.ans`.

pub mod ansi_dump;
pub mod png;

use std::io;
use std::path::{Path, PathBuf};
use turbo_vision::terminal::{CaptureKind, Terminal};

/// Save captures in the current working directory.
pub fn install(terminal: &mut Terminal) {
    install_in(terminal, ".");
}

/// Save captures in `dir`. Replaces any capture hook already installed. A
/// capture that cannot be written is logged and otherwise ignored.
pub fn install_in(terminal: &mut Terminal, dir: impl Into<PathBuf>) {
    let dir = dir.into();
    terminal.set_capture_hook(Box::new(move |kind, terminal| {
        let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let (path, result) = match kind {
            CaptureKind::Png => {
                let path = dir.join(format!("screenshot-{stamp}.png"));
                let result = save_png(terminal, &path);
                (path, result)
            }
            CaptureKind::Ansi => {
                let path = dir.join(format!("screen-{stamp}.ans"));
                let result = save_ansi(terminal, &path);
                (path, result)
            }
        };
        match result {
            Ok(()) => log::info!("capture saved to {}", path.display()),
            Err(e) => log::warn!("capture to {} failed: {e}", path.display()),
        }
    }));
}

/// Render the terminal's screen to a PNG at `path`, scaled to the terminal's
/// font cell height when the terminal reports one.
///
/// # Errors
///
/// Returns an error if the file cannot be created or written.
pub fn save_png(terminal: &Terminal, path: &Path) -> io::Result<()> {
    // Port of core Terminal::save_screenshot_png: pick the integer scale whose
    // glyph height is closest to the real cell height, clamped to 1..=8.
    let scale = match Terminal::query_font_pixel_size() {
        Some((_, ch)) if ch > 0 => {
            ((ch as usize + png::GLYPH_HEIGHT / 2) / png::GLYPH_HEIGHT).clamp(1, 8)
        }
        _ => 1,
    };
    let (w, h) = terminal.size();
    png::render_to_png(terminal.buffer(), w as usize, h as usize, scale, path)
}

/// Write the terminal's screen as ANSI-coloured text at `path`.
///
/// # Errors
///
/// Returns an error if the file cannot be created or written.
pub fn save_ansi(terminal: &Terminal, path: &Path) -> io::Result<()> {
    // Port of core Terminal::dump_screen.
    let (w, h) = terminal.size();
    ansi_dump::dump_buffer_to_file(terminal.buffer(), w as usize, h as usize, &path.to_string_lossy())
}
```

Check `ansi_dump::dump_buffer_to_file`'s real signature in the copied file, and core `Terminal::dump_screen` for how it is called, and match them. Fix any clippy pedantic cast warnings with `usize::from`.

- [ ] **Step 4: Run tests, checks, commit**

Run: `cargo test --lib --features capture capture 2>&1 | tail -3` (pass) and `cargo build --all-features --all-targets 2>&1 | grep -c warning` (0).

```bash
git add src/capture LICENSE-Spleen src/lib.rs Cargo.toml
git commit -m "feat(capture): PNG and ANSI screen captures, installed on the capture hook

Source: turbo-vision-4-rust src/core/{screenshot,ansi_dump}.rs, font8x16.bin, fonts/Spleen-LICENSE at <REV>."
```

---

### Task 10: Remote key input

**Files:**
- Create: `src/remote_input.rs` (copy of core `src/terminal/remote_input.rs`, rebuilt on `event_injector`)
- Modify: `src/lib.rs` (`#[cfg(feature = "remote-input")] pub mod remote_input;`)

**Interfaces:**
- Consumes: `Terminal::event_injector(&mut self) -> std::sync::mpsc::Sender<Event>`; `turbo_vision::core::event::parse_key_chord(&str) -> Option<Event>`.
- Produces: `tv_extensions::remote_input::{spawn(port: u16, tx: Sender<Event>) -> io::Result<()>, enable(terminal: &mut Terminal, port: u16) -> io::Result<()>, enable_from_env(terminal: &mut Terminal) -> io::Result<bool>}`. `enable_from_env` reads `TV_REMOTE_KEYS`: unset or not a `u16` gives `Ok(false)`; it binds and gives `Ok(true)`, or returns the bind error.

- [ ] **Step 1: Copy and rebuild**

Copy `remote_input.rs` with the rewrite table; `spawn` and the wire-format docs stay as they are. Add:

```rust
/// Listen on `127.0.0.1:port` and type every received chord into
/// `terminal`, as [`Terminal::event_injector`] does. Ctrl+F12 and F12
/// chords become captures if a capture hook is installed (see
/// `tv_extensions::capture`).
///
/// # Errors
///
/// Returns an error if the port cannot be bound.
pub fn enable(terminal: &mut Terminal, port: u16) -> std::io::Result<()> {
    spawn(port, terminal.event_injector())
}

/// [`enable`] on the port in the `TV_REMOTE_KEYS` environment variable.
/// Returns `Ok(false)`, doing nothing, when the variable is unset or not a
/// port number.
///
/// # Errors
///
/// Returns an error if the port cannot be bound.
pub fn enable_from_env(terminal: &mut Terminal) -> std::io::Result<bool> {
    let Some(port) = std::env::var("TV_REMOTE_KEYS").ok().and_then(|v| v.trim().parse::<u16>().ok()) else {
        return Ok(false);
    };
    enable(terminal, port)?;
    Ok(true)
}
```

Update the module doc: replace references to `Terminal::enable_remote_input` / `Application::enable_remote_input` with `enable` / `enable_from_env`.

- [ ] **Step 2: Tests**

Keep the copied tests. Add:

```rust
    #[test]
    fn enable_reports_a_port_in_use() {
        let taken = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = taken.local_addr().unwrap().port();
        let mut terminal = turbo_vision::test_util::test_terminal(10, 5);
        assert!(enable(&mut terminal, port).is_err());
    }

    #[test]
    fn a_chord_sent_over_tcp_arrives_through_poll_event() {
        use std::io::Write;
        let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = probe.local_addr().unwrap().port();
        drop(probe);
        let mut terminal = turbo_vision::test_util::test_terminal(10, 5);
        enable(&mut terminal, port).unwrap();
        let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream.write_all(b"ENTER\n").unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        let mut got = None;
        while got.is_none() && std::time::Instant::now() < deadline {
            got = terminal.poll_event(std::time::Duration::from_millis(20)).unwrap();
        }
        assert_eq!(got.map(|e| e.key_code), Some(turbo_vision::core::event::KB_ENTER));
    }
```

Check the copied module's wire format (the chord syntax `parse_key_chord` accepts, and whether lines end in `\n`) and use a chord it accepts for Enter.

For `enable_from_env`, environment variables are process-global, so test it under a lock with set/remove of `TV_REMOTE_KEYS` covering unset and `"not a port"` → `Ok(false)`. Name the test `enable_from_env_ignores_unset_and_garbage`. Mark the env mutation `unsafe` if the edition requires it; that is the one place `unsafe_code = "forbid"` would bite. If it does, test only the parsing by extracting `fn port_from(value: Option<String>) -> Option<u16>` and testing that instead, and say so in the report.

Run: `cargo test --lib --features remote-input remote_input 2>&1 | tail -3`
Expected: pass.

- [ ] **Step 3: Checks and commit**

`cargo build --all-features --all-targets 2>&1 | grep -c warning` → 0.

```bash
git add src/remote_input.rs src/lib.rs
git commit -m "feat(remote-input): TCP key injection on Terminal::event_injector

Source: turbo-vision-4-rust src/terminal/remote_input.rs at <REV>."
```

---

### Task 11: The SSH server

**Files:**
- Create: `src/ssh/mod.rs` (copy of core `src/ssh/mod.rs`), `src/ssh/handler.rs`, `src/ssh/server.rs` (copies), `src/ssh/backend.rs` (copy of core `src/terminal/ssh_backend.rs`)
- Modify: `src/lib.rs` (`#[cfg(feature = "ssh")] pub mod ssh;`)

**Interfaces:**
- Consumes: `turbo_vision::terminal::{Backend, Capabilities, InputParser, Terminal}`, `turbo_vision::core::command::CM_REDRAW`.
- Produces: `tv_extensions::ssh::{SshServer, SshServerConfig, SshAuthPolicy, run_ssh_server, SshBackend, SshSessionBuilder, SshSessionHandle}`, plus whatever core's `ssh/mod.rs` re-exports, with the same signatures.

- [ ] **Step 1: Copy with the rewrite table**

`crate::terminal::{SshBackend, SshSessionHandle}` → `super::backend::{SshBackend, SshSessionHandle}`. `super::backend::{Backend, Capabilities}` and `super::input_parser::InputParser` in `ssh_backend.rs` → `turbo_vision::terminal::{Backend, Capabilities, InputParser}`. Add `mod backend;` to `ssh/mod.rs` and re-export `SshBackend, SshSessionBuilder, SshSessionHandle`. Doc examples that use `turbo_vision::ssh::…` become `tv_extensions::ssh::…`.

- [ ] **Step 2: Run the copied tests and build**

Run: `cargo test --lib --features ssh ssh 2>&1 | tail -3` (pass), then `cargo build --features ssh 2>&1 | grep -E "^(warning|error)" | head`.
Expected: no errors, and no warnings from these files. (Core has 3 pre-existing warnings in `ssh/handler.rs` and `ssh/server.rs`; fix them in the copy, since this crate's bar is 0 warnings, and list the fixes in the report.)

- [ ] **Step 3: Commit**

```bash
git add src/ssh src/lib.rs
git commit -m "feat(ssh): serving an application over SSH, from turbo-vision core

Source: turbo-vision-4-rust src/ssh/*.rs and src/terminal/ssh_backend.rs at <REV>."
```

---

### Task 12: Examples

**Files:**
- Create in `examples/`: `kitty_image.rs`, `kitty_background.rs`, `kitty_biorhythm.rs`, `log_window.rs`, `terminal_widget.rs`, `ssh_server.rs`, `screenshot.rs`, `desktop_logo.rs` (copies of the turbo-vision examples with the same names, plus any data files they `include_str!`, such as `logo.txt` and `tv-logo.txt`), and `extras_controls.rs`, `extras_data.rs`, `extras_notebook.rs` (from turbo-vision `extras/examples/`)
- Modify: `Cargo.toml` (`[[example]]` entries with `required-features`)

**Interfaces:**
- Consumes: every module above; core `Slider`, `ComboBox`, `Spinner`, `ProgressBar`, `TabbedPane`, `Table` (with `set_provider`), `ListBox` (with `set_provider`).

- [ ] **Step 1: Copy and port**

For each copied example: rewrite `turbo_vision::views::{kitty_image, ansi_background, log_window, terminal_widget}` and `turbo_vision::ssh` to the `tv_extensions` paths. Where the example relied on built-in captures or `TV_REMOTE_KEYS` (`screenshot.rs`), call `tv_extensions::capture::install(&mut app.terminal)` and `tv_extensions::remote_input::enable_from_env(&mut app.terminal)?` right after building the app, and keep its header comment accurate. `required-features`: kitty/desktop_logo → `["graphics", "native"]`; log_window/terminal_widget → `["log", "native"]`; ssh_server → `["ssh"]`; screenshot → `["capture", "remote-input"]`.

For the three extras examples:
- `extras_controls`: `turbo_vision_extras::{ComboBox, Gauge, Slider, SpinControl}` → core `turbo_vision::views::{combo_box::ComboBox, progress_bar::ProgressBar, Slider, spinner::Spinner}`. Use the core constructors (read their signatures). Rename the file to `controls.rs` and give it `required-features = ["native"]`.
- `extras_data`: `GridView` over a `RowProvider` → core `Table` with `set_provider`; `VirtualListBox` over a `ListProvider` → core `ListBox` with `set_provider`. Rename to `lazy_data.rs`, `["native"]`.
- `extras_notebook`: `Notebook` → core `TabbedPane` (pages are `Group`s; see core `examples/new_controls.rs` for the pattern); `ScrollPane` and `popup_menu` → `tv_extensions::{ScrollPane, popup_menu}`. Rename to `scroll_and_popup.rs`, `["native"]`.

- [ ] **Step 2: Build every example**

Run: `cargo build --examples --all-features 2>&1 | grep -E "^(warning|error)" | head`
Expected: nothing. Then run one quickly to make sure it starts and quits: `timeout 3 cargo run --example lazy_data --features native </dev/null; echo exit=$?` may fail without a TTY; if so, note that it builds and skip running.

- [ ] **Step 3: Commit**

```bash
git add examples Cargo.toml
git commit -m "examples: the graphics, logging, capture and SSH examples, and the extras demos on core widgets

Source: turbo-vision-4-rust examples/ and extras/examples/ at <REV>."
```

---

### Task 13: Documentation and hand-off

**Files:**
- Modify: `README.md`, `docs/index.md`, `docs/host.md` (if it mentions Grid), `src/lib.rs` (crate docs listing every module and its feature)
- Create: `docs/scroll-pane.md`, `docs/csv.md`, `docs/log.md`, `docs/graphics.md`, `docs/capture.md`, `docs/remote-input.md`, `docs/ssh.md`, `CHANGELOG.md`

- [ ] **Step 1: Write the docs**

- **README:** one section per module, in the existing style (a sentence on what it is, the feature to enable, a short example taken from a test or an example file), plus a feature table.
- **`docs/*.md`:** one page per module, each linked from `docs/index.md`, covering the API, the feature, and for capture and remote input how they plug into the core hooks.
- **`CHANGELOG.md`:** an `[Unreleased]` section listing what arrived, where it came from, that Grid was retired, and that `tv-extensions` now tracks turbo-vision `main`.
- **Keep the plain style; no marketing words.**

- [ ] **Step 2: Final verification**

Run each command and check its result:
- `cargo test 2>&1 | tail -2`: ok.
- `cargo test --all-features 2>&1 | tail -2`: ok.
- `cargo build --all-features --all-targets 2>&1 | grep -c "^warning"`: 0.
- `cargo clippy --all-features --all-targets -- -D warnings 2>&1 | tail -3`: clean (fix anything it reports in this crate's files).
- `cargo doc --no-deps --all-features 2>&1 | grep -E "^(warning|error)" | head`: nothing.
- `sh scripts/check-wasm.sh`: ok.
- `for f in log graphics capture remote-input ssh csv native; do cargo check --features $f 2>&1 | grep -E "^error" | head -2; done`: no errors (each feature builds on its own).

- [ ] **Step 3: Commit, then ask**

```bash
git add README.md CHANGELOG.md docs src/lib.rs
git commit -m "docs: every extension module, its feature, and where it came from"
```

Report the commit list (`git log --oneline origin/main..`) and the verification results, then ask the user whether to push tv-extensions to origin.
