# Upgrading to 4.0.0

4.0.0 removes SSH, remote input, Kitty/ANSI graphics, the log window and
terminal widget, and the `turbo-vision-extras` crate from core. Apart from the
removed methods listed below, nothing in core's own `View`, `Application` or
`Terminal` API changed shape — every item either moved verbatim into the new
[`tv-extensions`](https://github.com/aovestdipaperino/tv-extensions) crate or
was folded into a core control that already existed. This guide is the ordered
list of what to change. The [changelog](changelog.md) records the same ground
as release notes.

## Do you need this?

If your program does not use SSH, remote input, Kitty images, ANSI-art
backgrounds, `LogWindow`/`TerminalWidget`, or `turbo-vision-extras`, nothing
else changed in 4.0.0 and you can stop reading. If you are jumping straight
from 3.0 rather than 3.1, also read the [3.1 note](#the-31-accessor-change)
near the end.

## Where each item went

| Removed from core | Now in tv-extensions | Feature |
|---|---|---|
| `turbo_vision::ssh::*` (`SshServer`, `SshServerConfig`, `SshAuthPolicy`, `run_ssh_server`, `AppFactory`, `PasswordAuthFn`, `PublicKeyAuthFn`, `TuiHandler`, `TuiSession`) | `tv_extensions::ssh::*` | `ssh` |
| `turbo_vision::terminal::{SshBackend, SshSessionBuilder, SshSessionHandle}` | `tv_extensions::ssh::{SshBackend, SshSessionBuilder, SshSessionHandle}` | `ssh` |
| `turbo_vision::terminal::remote_input`, `Terminal::enable_remote_input`, `Application::enable_remote_input` | `tv_extensions::remote_input::{spawn, enable, enable_from_env}` | `remote-input` |
| the `TV_REMOTE_KEYS` env var read in `Application::new` | `tv_extensions::remote_input::enable_from_env` (call it yourself) | `remote-input` |
| `turbo_vision::views::kitty_image::{KittyImage, KittyImageBuilder}` | `tv_extensions::graphics::{KittyImage, KittyImageBuilder}` | `graphics` |
| `turbo_vision::views::ansi_background::{AnsiBackground, AnsiBackgroundBuilder}` | `tv_extensions::graphics::{AnsiBackground, AnsiBackgroundBuilder}` | `graphics` |
| `turbo_vision::core::ansi` | `tv_extensions::graphics::ansi` | `graphics` |
| `Terminal::{write_kitty_graphics, supports_kitty_graphics, delete_kitty_image, clear_kitty_images}` | `tv_extensions::graphics::kitty::{supports_kitty_graphics, delete_kitty_image, clear_kitty_images}` (free functions; `write_kitty_graphics` is inlined at the call site with `Terminal::write_raw`) | `graphics` |
| `turbo_vision::views::log_window::{LogWindow, LogWindowBuilder, LogSubscriber}` | `tv_extensions::log::{LogWindow, LogWindowBuilder, LogSubscriber}` | `log` |
| `turbo_vision::views::terminal_widget::{TerminalWidget, TerminalWidgetBuilder, Span, OutputLine}` | `tv_extensions::log::{TerminalWidget, TerminalWidgetBuilder, Span, OutputLine}` | `log` |
| the `ssh` cargo feature on `turbo-vision` itself | gone; `ssh`/`graphics`/`log`/`remote-input`/`csv` are now features on `tv-extensions` | n/a |
| the `turbo-vision-extras` crate | `tv-extensions`, plus a few items absorbed into core — see [Extras users](#extras-users) below | varies |

Example, for the `LogWindowBuilder` row:

```rust
// 3.1.0
use turbo_vision::views::log_window::LogWindowBuilder;

// 4.0.0
use tv_extensions::log::LogWindowBuilder;
```

## Changed calls

A few call sites change shape because the functions they call are no longer
inherent methods on `Terminal` — they are free functions in `tv-extensions`
that take the terminal (or nothing) as an argument:

```rust
// 3.1.0
if terminal.supports_kitty_graphics() { /* ... */ }
terminal.clear_kitty_images()?;
terminal.delete_kitty_image(id)?;

// 4.0.0
use tv_extensions::graphics::kitty;

if kitty::supports_kitty_graphics() { /* ... */ }
kitty::clear_kitty_images(&mut terminal)?;
kitty::delete_kitty_image(&mut terminal, id)?;
```

`TV_REMOTE_KEYS` used to be read automatically inside `Application::new`.
Nothing reads it for you now; call the equivalent yourself once you have a
`Terminal`:

```rust
// 3.1.0
let mut app = Application::new()?; // read TV_REMOTE_KEYS for you

// 4.0.0
let mut app = Application::new()?;
tv_extensions::remote_input::enable_from_env(&mut app.terminal)?; // feature "remote-input"
```

## The tv-extensions dependency

Add the crate alongside `turbo-vision`, with only the features you use:

```toml
[dependencies]
turbo-vision = { version = "4.0" }
tv-extensions = { version = "0.2", features = ["ssh", "graphics", "log", "remote-input"] }
```

`tv-extensions` itself builds without turbo-vision's `native` feature by
default, so a crate that doesn't need a real terminal (e.g. a WASM guest)
still builds; `native` is pulled in automatically by the `ssh` and
`remote-input` features, since both need a real terminal to attach to.

## The `screenshot` feature

The PNG screen-capture feature (Ctrl+F12) is on by default in core, as it was
in 3.1.0; this did not change in 4.0.0. If your crate depends on
`turbo-vision` with `default-features = false`, add `features =
["screenshot"]` to keep Ctrl+F12 PNG captures — otherwise Ctrl+F12 and
`CM_SCREENSHOT` only run the capture hook, if one is installed, and otherwise
do nothing. F12's ANSI dump is unaffected either way.

## The 3.1 accessor change

If you are upgrading from 3.0.0 rather than 3.1.0: three accessors now return
owned strings instead of borrowing. `Table::selected_cell` and
`ListBox::get_selected_item` return `Option<String>`, and
`ListBox::marked_text` returns `Vec<String>`. Add `.as_deref()` where a `&str`
is still needed. See the [3.1.0 changelog entry](changelog.md) for the full
list of 3.1 changes.

## Extras users

`turbo-vision-extras` is gone. Everything it held either moved into core, or
into `tv-extensions`, in the 3.1.0/4.0.0 cycle:

| extras type | Replacement |
|---|---|
| `ComboBox` | core `ComboBox` (`turbo_vision::views::combo_box::ComboBox`) |
| `SpinControl` | core `Spinner` (`turbo_vision::views::spinner::Spinner`) |
| `Notebook` | core `TabbedPane` (`turbo_vision::views::tabbed_pane::TabbedPane`) |
| `Gauge` | core `ProgressBar` (`turbo_vision::views::progress_bar::ProgressBar`) |
| `Slider` | core `Slider` (`turbo_vision::views::slider::Slider`), moved in during 3.1.0 |
| `GridView` | core `Table` + `table::RowProvider` (`Table::set_separators` gives the column separators `Grid` used to draw); extras' `GridColumn` maps to core `table::Column`, and extras' `VecRowProvider` to a simple `RowProvider` impl; extras' `RowProvider` trait has the same methods as core's `table::RowProvider` (`rows()`, `cell(row, col)`) |
| `VirtualListBox` | core `ListBox` + `listbox::ListProvider`; extras' `ListProvider` trait has the same methods as core's `listbox::ListProvider` (`len()`, `item(i)`) |
| `ScrollPane` | `tv_extensions::ScrollPane` (no feature flag) |
| `popup_menu` | `tv_extensions::popup_menu` (no feature flag) |

## When something goes wrong

**`use turbo_vision::ssh::...` / `use turbo_vision::terminal::SshBackend`
no longer resolves.** The whole module moved; see the table above and add
`tv-extensions` with the `ssh` feature.

**A build with `default-features = false` lost Ctrl+F12 PNG captures.** Add
`features = ["screenshot"]`; see [above](#the-screenshot-feature).

**`cargo build` can't find `turbo-vision-extras`.** Remove it from
`Cargo.toml` and depend on `tv-extensions` instead; see
[Extras users](#extras-users).

**`Terminal::supports_kitty_graphics` / `clear_kitty_images` /
`delete_kitty_image` don't exist any more.** They are free functions in
`tv_extensions::graphics::kitty` now; see [Changed calls](#changed-calls).
