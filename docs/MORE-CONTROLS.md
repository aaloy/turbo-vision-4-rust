# More Controls — gap analysis and roadmap

Survey of the widget set in `src/views/` against both the Borland Turbo Vision
reference and the expectations of a modern text UI toolkit. Work top to bottom
within each section.

## What already exists

Containers: `Group`, `Window`, `Dialog`, `Desktop`, `Frame`, `Background`.
(`AnsiBackground` moved to `tv-extensions`' `graphics` feature in 4.0.0.)

Input: `InputLine` (with `Validator`, `PictureValidator`, `LookupValidator`),
`Button`, `CheckBox`, `RadioButton`, `Cluster`, `Editor` / `EditWindow` /
`FileEditor`.

Display: `StaticText`, `Label`, `ParamText`, `Memo`, `TextViewer`, `ListBox`,
`SortedListBox`, `DirListBox`, `FileList`, `Outline` (tree), `Indicator`,
`Scroller`, `ScrollBar`. (`KittyImage`, `TerminalWidget` and `LogWindow` moved
to `tv-extensions`' `graphics` and `log` features in 4.0.0; see
[UPGRADING-TO-4.0.md](../UPGRADING-TO-4.0.md).)

Chrome and canned dialogs: `MenuBar`, `MenuBox`, `MenuViewer`, `StatusLine`,
`msgbox`, `FileDialog`, `ChDirDialog`, `ColorDialog`, `ColorSelector`,
`HistoryWindow`, the `help_*` family.

## New controls

Ordered by benefit-to-effort. Each one composes existing views wherever it can.

- [x] **ProgressBar / gauge** — done, `src/views/progress_bar.rs`. Determinate
      and marquee modes; `Smooth` / `Blocks` / `Ascii` glyph styles; centred
      percentage overlay that can be toggled off or replaced with fixed text;
      self-animating marquee via `IdleView`; `CP_PROGRESS_BAR` palette reusing
      the scrollbar gauge colours. Demo in `examples/progress_bar.rs`.
      Still open: a compact status-line variant.
- [x] **ComboBox / dropdown list** — done, `src/views/combo_box.rs`. Read-only
      flavour: a field with a drop arrow, F4 or a click drops the list, arrows
      cycle without opening. The popup runs modally through the same two-step
      command the history button uses, since a control cannot reach the terminal
      from `handle_event`. Still open: the editable flavour, where the field is
      a real `InputLine`.
- [x] **Spinner / numeric up-down** — done, `src/views/spinner.rs`. Holds one
      integer inside a range, so typed input is clamped rather than rejected.
      Arrows step, PgUp and PgDn step ten times as far, Home and End jump to the
      ends, digits edit the number, and the steppers are clickable. Optional
      wrap-around and a unit suffix.
- [x] **TabbedPane / notebook** — done, `src/views/tabbed_pane.rs`. Each page is
      a `Group`, so it holds ordinary controls and runs its own focus traversal.
      Drawn as enclosed tab boxes over a framed page, the active tab's floor
      open. F6 and Shift+F6 switch, as do Ctrl+PgUp/PgDn where the terminal
      sends them, tilde hotkeys, and clicking a tab. Tab cycles within the
      active page rather than escaping it.
- [x] **Table / grid view** — done, `src/views/table.rs`. Header row, per-column
      widths and alignment, and a focused cell rather than a focused row: Up and
      Down move rows, Left and Right move columns, and the grid scrolls in both
      directions by whole columns. Ragged rows draw blank instead of panicking.
- [x] **Splitter** — done as `SplitPane`, `src/views/split_pane.rs`. A bare
      divider cannot resize siblings it does not own, so the control owns both
      halves, each a `Group`, the way `TabbedPane` owns its pages. Vertical or
      horizontal, draggable, with a minimum size per half; F8 moves focus between
      them. Keyboard divider movement is left to the host through `grow_first`
      and `shrink_first`, rather than stealing a key from the panes.
- [x] **Tooltip / hint popup** — done, `src/views/tooltip.rs`. One tooltip serves
      a whole dialog: register a rect and a line of text per control, and the
      pointer resting on one raises the hint beside it. Add it last, since it
      draws over its neighbours. The hover delay runs off the new `CM_IDLE_TICK`
      broadcast.
- [x] **Slider** — done, `src/views/slider.rs`. Moved in from
      turbo-vision-extras; the dragging counterpart of `Spinner`. A horizontal
      track with a thumb over an `i64` range.

  | Key | Action |
  |-----|--------|
  | Left, Right | One step down or up |
  | Home, End | Minimum or maximum |

  Clicking or dragging the track moves the thumb under the pointer; a drag
  keeps tracking the pointer even once it leaves the track. `set_on_change`
  broadcasts a command on every user-driven change.

  ```rust
  use turbo_vision::views::Slider;
  use turbo_vision::core::geometry::Rect;

  let mut slider = Slider::new(Rect::new(2, 2, 30, 3), 0, 100);
  slider.set_step(5);
  slider.set_on_change(1);
  assert_eq!(slider.value(), 0);
  ```

## Completing existing controls

- [x] **`CheckBoxes` / `RadioButtons` as true clusters** — done,
      `src/views/cluster_group.rs`. Each holds its items in one focusable control
      with a single bitmask value, as Borland does. Arrows move within the
      cluster, Space toggles or selects, Tab leaves it, and a tilde-marked letter
      is an item's Alt hotkey; items can be disabled individually. The existing
      one-label `CheckBox` and `RadioButton` are untouched and still supported.
- [x] **Multi-select in `ListBox`** — done. `set_multi_select` adds marks that
      are independent of the focus: Space marks the focused item, Shift+click
      marks a run from the anchor, and `marked_items` / `marked_text` report them
      in list order. Marked rows carry a check glyph in a two-cell column, and
      `is_selected` follows the marks in that mode. Off by default, so existing
      single-selection lists are untouched.
- [x] **ScrollBar mouse auto-repeat** — done. `Application` tracks whether a
      button is held and, only then, broadcasts `CM_MOUSE_AUTO_REPEAT` from its
      idle pass; the scrollbar repeats the press it is holding after a 400 ms
      delay, every 80 ms, and stops at the end of the range. Nothing is sent
      while no button is down, so an idle app stays idle.
- [x] **Frame zoom-icon rendering** — done. A resizable window's title bar now
      carries `[\u{25B2}]` beside the close box, turning into `[\u{25BC}]` once
      zoomed. It tracks press and release like the close box, so a press that
      slides off cancels. Dialogs show none, since Borland pairs wfZoom with
      wfGrow and a dialog has neither.
- [x] **Table column separators** — done. `Table::set_separators` /
      `TableBuilder::separators` draw `table::SEPARATOR` (`│`) in the
      one-cell gap between each pair of visible columns, in each line's own
      colour. Off by default; columns stay where they are, so a click lands
      on the same cell either way.
- [x] **Table frozen panes** — done. `Table::set_frozen_cols` /
      `TableBuilder::frozen_cols` keep the first columns at the left while
      the rest scroll sideways, marked by `table::FROZEN_SEPARATOR` (`║`)
      whether separators are on or not; `Table::set_frozen_rows` /
      `TableBuilder::frozen_rows` keep the first rows under the header while
      the rest scroll, the last one underlined. Focusing a frozen cell
      scrolls nothing. `examples/table_frozen.rs` freezes a region column
      and a totals row.
- [x] **Lazy rows and items** — done. `table::RowProvider` (`rows()`,
      `cell(row, col)`) and `listbox::ListProvider` (`len()`, `item(index)`,
      a default `is_empty`) let a `Table` or `ListBox` read its content only
      as it draws it, instead of holding every row or item as an owned
      string. Give one to `Table::set_provider` / `ListBox::set_provider`;
      call `refresh_rows` / `refresh_items` after the source's length
      changes — drawing and cell access always read the provider's current
      length, but navigation and the `row_count` / `item_count` accessors use
      the count from the last refresh. Because content can now come from a
      provider, `Table::selected_cell`, `ListBox::get_selected_item` and
      `ListBox::marked_text` return owned `String`s instead of borrowing.

## Demo

`examples/new_controls.rs` runs all five new controls in one dialog. A tabbed
pane holds two pages: the first wires three combo boxes and a spinner to a
progress bar, the second is a table.

## What is left

Nothing on this list; the next round is under
[Next controls](#next-controls-2026-10-review). Two things worth knowing about what shipped:

- `CM_IDLE_TICK`, added for the tooltip's hover delay, is a general timer any
  view can use. `Application::idle` broadcasts it whenever the event poll times
  out. Views must not consume it, since a broadcast stops travelling once it is.
- The clusters take their colours from `CP_CLUSTER`, so they look exactly like
  the existing one-label `CheckBox` and `RadioButton`.

## Next controls (2026-10 review)

What a data-entry or business application still has to build by hand. Ordered
by benefit; the first five are what a Django-admin-like layer (list window,
generated edit dialog, model menu) would need, so they come first.

### Forms and data

- [ ] **Form layout helper** — every view is placed with an absolute `Rect`,
      the main source of effort and of off-by-one bugs in hand-built dialogs.
      A helper that stacks label/field rows in two columns (and pages long
      forms with `TabbedPane`) would shrink dialogs to a list of fields and
      let them follow a resize through the grow modes.
- [ ] **Form data transfer** — Borland's `TView::getData` / `setData` /
      `dataSize` filled a whole dialog from a record and read it back in one
      call; the port has no equivalent, so every field is wired by hand
      through its own `Rc<RefCell<…>>`. A per-control value trait, a dialog
      that gathers them, and dirty tracking ("discard changes?") would
      replace that wiring.
- [ ] **Date and time input** — a field with a drop-down calendar. A calendar
      exists only inside `examples/showcase.rs`; `chrono` is already a
      dependency, and the drop-down can reuse `ComboBox`'s two-step popup
      command.
- [ ] **Editable `ComboBox` / autocomplete** — the open item above: the field
      is a real `InputLine` and the list narrows to the matches as one types.
      Needed to pick from long lists such as a related record.
- [ ] **`Table` sorting and filtering** — sort by clicking a column header
      (with an indicator), an optional filter row, and resizable columns.
      Sorting a `RowProvider` table means asking the provider, so the trait
      needs an optional sort hook.

### Navigation and feedback

- [ ] **Command palette** — Ctrl+P, type part of a name, run any menu
      command. The menu data is already structured (`menu_data`), so it is
      mostly a filtered `ListBox` in a popup that skips disabled commands.
- [ ] **Toast notifications** — short, non-modal messages that go away on
      their own ("Saved", "3 rows deleted"). `core::timed_event` (added for
      the button press animation) or `CM_IDLE_TICK` drives the timeout.
- [ ] **Wizard dialog** — steps with Back / Next / Finish and validation
      before each step, built on `TabbedPane` with the tab strip hidden.
- [ ] **Tree with columns** — `Outline` with `Table`'s columns, for
      hierarchical data: folders with sizes and dates, accounts with balances.

### Smaller

- [ ] **Status-line progress bar** — the compact variant still open under
      ProgressBar above.
- [ ] **Toolbar** — a row of buttons under the menu bar. Less usual in text
      UIs, so optional.

### Not controls, but as important

- [ ] **Scripted UI tests** — a supported way to drive a running application
      with keys and mouse events and assert on the screen. The showcase's
      mouse routing broke silently when views moved to owner-relative
      coordinates; such a test would have caught it. `test_util` and the
      remote-input listener in `tv-extensions` are the starting points.
- [ ] **Wide characters** — CJK and emoji cannot be typed or shown yet, since
      a screen cell holds one character, one column wide.

## Notes

New views follow the established shape: a struct holding `bounds` and a
`palette_chain`, an `impl View`, a `*Builder` with a fluent API and
`build()` / `build_boxed()`, unit tests in the same file, and a `pub mod` entry
plus doc-comment listing in `src/views/mod.rs`.
