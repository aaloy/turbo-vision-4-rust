# Core / extensions split — design

Date: 2026-10-01
Repos: `turbo-vision-4-rust` (core), `tv-extensions`, `plank-csvedit` (renamed `plank-tv`), `plank`

## Goal

Core keeps Borland Turbo Vision plus the widgets most applications need, and
nothing else. Niche features move to `tv-extensions`, which versions on its
own. Duplicate widgets collapse to one implementation. csvedit's CSV editing
becomes usable by any turbo-vision app, with its plank glue in a separate
`plank-tv` crate.

Success means:

- One implementation per widget across all three repos.
- Core no longer depends on `tracing`, `chrono`, `base64`, `simplelog` or the
  SSH stack, and no longer embeds the PNG font.
- Every feature that leaves core still works from `tv-extensions`, with its
  tests and examples.
- `tv-extensions` and `plank-tv` depend on a core `main` commit or release,
  never on a side branch.
- A native app can edit a CSV file with `tv-extensions` alone, without
  pulling in any plank dependency.

## Decisions

| Question | Decision |
|---|---|
| What is core? | Borland Turbo Vision 2.0 equivalents, plus the common widgets: `Table`, `ListBox`, `ComboBox`, `ProgressBar`, `Spinner`, `Slider`, `TabbedPane`, `SplitPane`, `Tooltip`, syntax highlighting and markdown help. |
| Duplicates | Core wins. The extras versions of `ComboBox`, `SpinControl`, `Notebook` and `Gauge` are deleted with no deprecation period, because `turbo-vision-extras` was never published. |
| The three grids | One core `Table`. It gains a `separators` flag and an optional `RowProvider` source. Extras' `GridView` and tv-extensions' `Grid` are deleted. This reverses the separator move in core commit `eae2cc8`. |
| Lists | Core `ListBox` gains an optional `ListProvider` source. Extras' `VirtualListBox` is deleted. |
| csvedit | The generic CSV editor goes into `tv-extensions` under a `csv` feature. The plank glue goes into `plank-tv`. |
| Where plank-tv lives | The GitHub repo `plank-csvedit` is renamed to `plank-tv`. |
| How code moves | A plain copy. The commit message names the source repo and commit. No history surgery. |
| Core version | 4.0.0, a breaking release with an `UPGRADING-TO-4.0.md`. |

Out of scope: publishing `tv-extensions` or `plank-tv` to crates.io, and any
new widget from the component survey (ToolBar, progress dialog and so on).

## Target layout

### turbo-vision core 4.0

Keeps everything in `src/` except the modules listed under tv-extensions
below.

Additions:

- `Table::set_separators(bool)` and `TableBuilder::separators(bool)`. These
  draw `│` in the one-cell gap after each column, using the colour the table
  drew in that gap. This is the logic of `tv_extensions::Grid`, moved in.
- `table::RowProvider` trait (`rows()`, `cell(row, col)`) and
  `Table::set_provider(Box<dyn RowProvider>)`. When a provider is set, rows
  are fetched lazily; `set_rows`/`add_row` switch back to in-memory rows
  (`add_row` on a provider-backed table starts a new list). `refresh_rows()`
  re-reads a provider whose length changed. The existing API keeps working
  unchanged, except `selected_cell` now returns `Option<String>` instead of
  `Option<&str>`, since a selected cell can come from a provider.
- `listbox::ListProvider` trait (`len()`, `item(i)`) and
  `ListBox::set_provider(Box<dyn ListProvider>)`. Same rules as `Table`,
  plus `refresh_items()`. `get_selected_item`/`marked_text` return owned
  strings for the same reason `selected_cell` does.
- `views::slider::Slider`, moved from extras. It uses palette mapping like
  the other core controls, so it draws correctly in a `Dialog`. Its thumb
  is `■`, a CP437 glyph.
- Extension hooks, the only new public surface the extensions need:
  - `Terminal::event_injector() -> Sender<Event>`, a public way to queue
    events from another thread. This replaces the private remote-input
    channel.
  - `Terminal::set_capture_hook(Box<dyn FnMut(CaptureKind, &Terminal)>)`,
    with `CaptureKind::{Png, Ansi}` (the PNG renderer needs the font-size
    query as well as the cells), plus `clear_capture_hook` and
    `run_capture_hook`. `Application` calls it on Ctrl+F12 and F12 instead
    of `take_screenshot`/`dump_screen_ansi`. With no hook set, the built-in
    capture still runs.
  - `Terminal::write_raw(&[u8])`, a passthrough to the backend for protocols
    such as Kitty graphics.
  - `terminal::InputParser` exported unconditionally instead of only under
    `feature = "ssh"`. It is a generic byte-stream parser.

Already on `wasm-host-driven`, and arriving with the fast-forward: the
`native` feature, `Backend::is_host_driven`, `Application::step`,
`core::keys`, and `Table::column_offsets`.

Removed from core:

- `src/views/{kitty_image,ansi_background,log_window,terminal_widget}.rs`
- `src/core/{ansi,ansi_dump,screenshot}.rs`, the embedded
  `src/core/font8x16.bin` and `fonts/Spleen-LICENSE` (all of them move to
  `capture`)
- `src/terminal/{remote_input,ssh_backend}.rs` and `src/ssh/`
- `Terminal::{enable_remote_input, save_screenshot_png, dump_screen,
  dump_region, write_kitty_graphics, supports_kitty_graphics,
  delete_kitty_image, clear_kitty_images}` and
  `Application::{enable_remote_input, take_screenshot, dump_screen_ansi}`.
  The `TV_REMOTE_KEYS` env handling moves with `remote_input`.
- The `ssh` feature and its dependencies, plus `tracing`, `chrono`,
  `base64` and `simplelog`. `log` stays, because about a dozen core views
  log through it.
- The `extras/` workspace member.
- The examples that move (see Examples and docs).

### tv-extensions

A single library crate with one feature per group, so the wasm build stays
lean:

| Module | Feature | Contents | Source |
|---|---|---|---|
| `host` | default | `HostBackend`, `HostInput`, `pump` | already here |
| `scroll_pane` | default | `ScrollPane` | extras |
| `popup_menu` | default | `popup_menu`, check-mark helpers | extras |
| `log` | `log` | `LogWindow`, `LogSubscriber`, `TerminalWidget`, `Span` | core; brings `tracing` |
| `graphics` | `graphics` | `KittyImage`, `AnsiBackground`, ANSI parser | core; brings `base64`; built on `Terminal::write_raw` |
| `capture` | `capture` | PNG renderer and font, ANSI dump, `install(&mut Application)` that registers the capture hook | core; brings `chrono` |
| `remote_input` | `remote-input` (needs `native`) | TCP key injection, `enable(&mut Application, port)` built on `event_injector` | core |
| `ssh` | `ssh` (needs `native`) | SSH server, `SshBackend`, auth policy | core; brings tokio and russh |
| `csv` | `csv` | `csv`, `doc::CsvDoc`, `editor::Session`, dialogs, commands, `Disk`, `MemDisk`, new `FsDisk` | plank-csvedit |

`grid.rs` (`Grid`) is deleted, and its tests move to core `Table`.

`turbo-vision` is a git dependency on core `main` (path dependency for local
development) until 4.0 is released, then `4.0`.

### plank-tv (renamed from plank-csvedit)

Depends on `tv-extensions` (feature `csv`), `plank-guest-support` and
`extism-pdk`.

- Reusable glue for any turbo-vision app in plank: `paint` (cells to
  `CellGlyph`), `keys` (plank key payload to `Event`), and `PlankDisk`
  (implements `tv_extensions::csv::Disk`).
- The csvedit plugin, behind a default-on `csvedit` feature so that a second
  plugin can depend on plank-tv without clashing Extism exports: `frame`
  exports, `summary`, `plugin.json`, `package.sh` and CI.
- `Session::cells()` is removed from the editor. plank-tv calls
  `paint::cells(session.app().terminal.buffer())` instead, which means
  `Session` needs an `app()` accessor or an equivalent buffer accessor.
- The plugin id `dev.plank.csvedit` and the module name `csvedit.wasm` stay
  the same, so installed plugins and trust entries keep working until the
  bytes change.

### plank

Update `docs/WASM-PLUGINS.md`, `tests/wasm_csvedit.rs`, `tests/grid_bridge.rs`
and anything else that names `plank-csvedit` or its paths.

## Order of work

Every step leaves all repos building and their tests passing.

1. **Core: merge.** Fast-forward `main` to `origin/wasm-host-driven`
   (`main` has no commits the branch lacks).
2. **Core: absorb.** Add `Table` separators, `RowProvider`, `ListProvider`,
   `Slider`, and the four extension hooks. Port the tests from
   `tv-extensions/tests/grid.rs`, `extras/src/grid.rs`,
   `extras/src/virtual_listbox.rs` and `extras/src/slider.rs`. Removed APIs
   are still present at this step, so nothing breaks yet.
3. **tv-extensions: receive.** Point it at core `main`. Add the modules in
   the table, each with its tests and examples, re-implemented on the hooks
   where they used to reach into `Terminal` or `Application`. Delete
   `Grid`. Add the `csv` module from plank-csvedit, without the plank
   parts, plus `FsDisk` and a native example that edits a CSV file. Keep
   `scripts/check-wasm.sh` green with default features and with `csv`.
4. **Core: remove.** Delete the moved modules, APIs, features,
   dependencies, examples and `extras/`. Write `UPGRADING-TO-4.0.md` with a
   moved-to table. Update `README.md`, `CHANGELOG.md`, `docs/` and
   `website/`. Bump to 4.0.0. Release.
5. **plank-tv.** Rename the GitHub repo. Remove the modules that moved to
   tv-extensions, depend on tv-extensions, and add the `csvedit` feature
   gate. Run `package.sh` and confirm the module still loads in plank.
6. **plank.** Update the references and run its csvedit tests against the
   new module.

## Examples and docs

These examples move to `tv-extensions/examples/`: `kitty_image`,
`kitty_background`, `kitty_biorhythm`, `log_window`, `terminal_widget`,
`ssh_server`, `screenshot`, `desktop_logo` (it uses a moved module), plus
the three `extras/examples/*` with the deleted duplicates swapped for core
widgets. No other core example or demo uses a moved module.

The docs site keeps its pages for core widgets. Pages for moved features
link to the tv-extensions docs (`tv-extensions/docs/`).

## Testing

- Core: all existing tests that are not about moved code, plus new tests for
  separators, both providers, Slider and each hook. `cargo test`,
  `cargo test --no-default-features` and `scripts/check-wasm.sh`.
- tv-extensions: moved tests per feature, with `cargo test --all-features`,
  `cargo test` (defaults) and the wasm check with `--features csv`. The
  capture tests render a PNG and compare against the glyphs drawn.
- plank-tv: native `cargo test --lib`, plus the wasm build via `package.sh`.
- plank: `tests/wasm_csvedit.rs` and `tests/grid_bridge.rs` against the
  rebuilt module.

## Risks

- **Hidden internals.** The moved modules were checked for crate-private
  imports: `log_window` uses `views::shared::Shared`, which is already
  public, and `ssh` uses `InputParser`, which becomes public. Anything else
  found during step 3 gets the smallest public accessor in core or moves
  with the module. That decision is recorded in the commit.
- **Palette regressions.** `ScrollPane` and `Notebook` children currently
  render with wrong colours, and `ProgressBar`/`ListBox` blend into blue
  windows (see the component report). Moving `ScrollPane` is a chance to
  fix its palette chain. The `ProgressBar`/`ListBox` issue is out of scope,
  but a test must not lock the bad colours in.
- **Screenshot font gaps.** The PNG renderer draws `?` for `◆ √ ► ◄ ▏–▉`.
  It moves to `capture` unchanged. Fixing the gaps is a separate follow-up.
- **wasm build.** `tracing`, `chrono` and tokio must stay behind their
  features, so a default tv-extensions build still compiles for
  `wasm32-wasip1`.
