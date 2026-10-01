# Core: merge and absorb — Implementation Plan (1 of 4)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring `main` up to `wasm-host-driven`, then give core everything it absorbs from the extensions: `Table` separators and a lazy row source, a lazy `ListBox` item source, `Slider`, and the four extension hooks that `tv-extensions` will build on.

**Architecture:** Additions only, except for three accessor signatures that have to return owned text once rows can come from a provider. `Table` and `ListBox` each store their data in a private `Rows`/`Items` enum (owned vectors, or a boxed provider). The hooks live on `Terminal`: an event-injection channel, a capture callback that `Application` calls on Ctrl+F12 and F12, a raw-bytes passthrough, and an always-public `InputParser`. Every built-in behaviour (remote input, built-in screenshots) keeps working on top of the hooks until plan 3 deletes it.

**Tech Stack:** Rust 2024, `cargo test`, the crate's `test_util::test_terminal`, `wasm32-wasip1` check script.

**Spec:** `docs/superpowers/specs/2026-10-01-core-extensions-split-design.md`

**Later plans** (written once this one lands): 2. tv-extensions receives the moved code. 3. Core removal and 4.0 release. 4. plank-tv and plank.

## Global Constraints

- Core version stays 3.x in this plan. The 4.0.0 bump happens in plan 3.
- Each separator character is `│` (U+2502). It takes the colour already drawn in its gap.
- Separators are **off** by default on `Table`. (tv-extensions' `Grid` defaulted them on. Its users opt in.)
- A provider-backed `Table`/`ListBox`: `set_rows`/`add_row`/`set_items`/`add_item` switch back to in-memory data. `add_row`/`add_item` on a provider-backed view start a new in-memory list containing just that row or item.
- Changing the data source (`set_rows`, `set_provider`, `set_items`) drops `ListBox` marks, as `set_items` already does.
- No new dependencies. `default-features = false` must still build for `wasm32-wasip1` (`scripts/check-wasm.sh`).
- New public items carry doc comments, in the existing style: plain sentences, Borland references where relevant.
- New files start with the `// (C) 2026 - Enzo Lombardi` header used across `src/`.
- Do not push to `origin` without asking the user first.

## Review Focus

1. **A provider swapped for a shorter one while a late row is focused.** Focus clamps to the last row and drawing does not panic. Pinned in Task 3 (`a_shorter_provider_clamps_the_focus`) and Task 4 (`a_shorter_provider_clamps_the_selection_and_drops_marks`).
2. **Separators on a horizontally scrolled table.** Separators appear only between the columns actually on screen, never after the last visible one or past the right edge. Pinned in Task 2 (`separators_follow_horizontal_scrolling`).
3. **Degenerate slider ranges** (`min == max`, reversed bounds, a one-cell-wide slider, `i64` extremes). No panic or overflow, and the thumb stays inside the view. Pinned in Task 5 (`degenerate_ranges_do_not_panic`).
4. **Events injected from another thread through cloned senders.** They arrive in order through `poll_event`, and repeated `event_injector()` calls share one channel. Pinned in Task 6 (`injected_events_arrive_in_order_from_any_sender`).
5. **Capture keys with no hook installed.** Behaviour is unchanged (built-in capture). With a hook installed, no file is written and the hook sees the terminal. Pinned in Task 7 (`without_a_hook_run_capture_hook_reports_false`, `capture_keys_run_the_capture_hook`).

---

## File Structure

| File | Change | Responsibility |
|---|---|---|
| `src/views/table.rs` | Modify | `SEPARATOR`, separators flag, `RowProvider`, private `Rows` enum, `set_provider`, `refresh_rows`, `selected_cell -> Option<String>` |
| `src/views/listbox.rs` | Modify | `ListProvider`, private `Items` enum, `set_provider`, `refresh_items`, owned-text accessors |
| `src/views/slider.rs` | Create | `Slider` control |
| `src/views/mod.rs` | Modify | `pub mod slider;`, re-exports |
| `src/terminal/mod.rs` | Modify | `event_injector`, `CaptureKind`, `CaptureHook`, `set_capture_hook`, `clear_capture_hook`, `run_capture_hook`, `write_raw`, unconditional `InputParser` |
| `src/app/application.rs` | Modify | Ctrl+F12, F12 and `CM_SCREENSHOT` try the capture hook first |
| `examples/list_components.rs` | Modify | Compiles against `get_selected_item -> Option<String>` (no logic change) |
| `CHANGELOG.md` | Modify | `[Unreleased]` section listing the additions and signature changes |
| `docs/superpowers/specs/2026-10-01-core-extensions-split-design.md` | Modify | Bring the hook and provider details in line with this plan |

---

### Task 1: Fast-forward main and set up the working branch

**Files:** none (git only)

**Interfaces:**
- Produces: branch `feat/core-absorb` containing `origin/wasm-host-driven` plus the spec and this plan.

- [ ] **Step 1: Confirm main has nothing the branch lacks**

Run: `git fetch origin && git log --oneline origin/wasm-host-driven..main`
Expected: no output. If there is output, stop and ask the user: the fast-forward assumption no longer holds.

- [ ] **Step 2: Fast-forward main**

```bash
git switch main
git merge --ff-only origin/wasm-host-driven
```
Expected: `Fast-forward`, with HEAD at `eae2cc8`.

- [ ] **Step 3: Run the suite on the merged main**

Run: `cargo test 2>&1 | tail -5`
Expected: `test result: ok` on every binary, 0 failed.

- [ ] **Step 4: Move the spec and plan branch onto main and rename it**

```bash
git switch docs/core-extensions-split
git rebase main
git branch -m docs/core-extensions-split feat/core-absorb
git log --oneline -3
```
Expected: the spec commit, then this plan's commit, on top of `eae2cc8`.

- [ ] **Step 5: Install the wasm target if it is missing**

Run: `rustup target list --installed | grep -q wasm32-wasip1 || rustup target add wasm32-wasip1`
Then: `sh scripts/check-wasm.sh`
Expected: `Finished` with no errors.

- [ ] **Step 6: Ask the user whether to push `main`**

Ask: "main is fast-forwarded to wasm-host-driven locally. Push it to origin now?" Push only on a yes (`git push origin main`).

---

### Task 2: Table column separators

**Files:**
- Modify: `src/views/table.rs` (struct `Table`, `Table::new`, `draw`, `TableBuilder`)
- Test: `src/views/table.rs` (`mod tests`)

**Interfaces:**
- Produces: `pub const SEPARATOR: char`, `Table::set_separators(&mut self, on: bool)`, `Table::separators(&self) -> bool`, `TableBuilder::separators(self, on: bool) -> Self`.

- [ ] **Step 1: Write the failing tests**

Append inside `mod tests` in `src/views/table.rs`:

```rust
    fn draw(t: &mut Table, w: u16, h: u16) -> crate::terminal::Terminal {
        let mut term = crate::test_util::test_terminal(w, h);
        t.draw(&mut term);
        term
    }

    fn ch(term: &crate::terminal::Terminal, x: i16, y: i16) -> char {
        term.read_cell(x, y).unwrap().ch
    }

    #[test]
    fn separators_are_off_by_default() {
        let mut t = table(3);
        assert!(!t.separators());
        let term = draw(&mut t, 30, 6);
        assert_eq!(ch(&term, 10, 1), ' ');
    }

    #[test]
    fn separators_fill_the_gaps_between_visible_columns() {
        // Columns: Name 0..10, gap 10, Size 11..17, gap 17, Kind 18..26.
        let mut t = table(3);
        t.set_separators(true);
        let term = draw(&mut t, 30, 6);
        for y in 0..6 {
            assert_eq!(ch(&term, 10, y), SEPARATOR, "first gap, line {y}");
            assert_eq!(ch(&term, 17, y), SEPARATOR, "second gap, line {y}");
            assert_ne!(ch(&term, 26, y), SEPARATOR, "after the last column, line {y}");
        }
        // Text is untouched.
        assert_eq!(ch(&term, 0, 1), 'f');
    }

    #[test]
    fn a_separator_takes_the_colour_of_its_line() {
        let mut t = table(3);
        t.set_separators(true);
        let term = draw(&mut t, 30, 6);
        let attr = |x, y| term.read_cell(x, y).unwrap().attr;
        assert_eq!(attr(10, 0), attr(0, 0), "header");
        assert_eq!(attr(10, 1), attr(11, 1), "selected row bar");
        assert_eq!(attr(10, 2), attr(11, 2), "normal row");
    }

    #[test]
    fn separators_follow_horizontal_scrolling() {
        // 20 wide: focusing Kind scrolls Name off, leaving Size at 0..6 and
        // Kind at 7..15. The only gap between visible columns is x = 6.
        let mut t = table(3);
        t.set_bounds(Rect::new(0, 0, 20, 6));
        t.set_separators(true);
        t.set_selected_col(2);
        let term = draw(&mut t, 20, 6);
        assert_eq!(ch(&term, 6, 1), SEPARATOR);
        assert_ne!(ch(&term, 15, 1), SEPARATOR);
        assert_ne!(ch(&term, 19, 1), SEPARATOR);
    }

    #[test]
    fn the_builder_sets_separators() {
        let t = TableBuilder::new()
            .bounds(Rect::new(0, 0, 10, 3))
            .separators(true)
            .build();
        assert!(t.separators());
    }
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test --lib views::table::tests::separator 2>&1 | tail -5`
Expected: compile error, `cannot find value SEPARATOR` / `no method named set_separators`.

- [ ] **Step 3: Implement**

In `src/views/table.rs`, below `const COLUMN_GAP`:

```rust
/// Drawn in each gap between two visible columns when separators are on.
pub const SEPARATOR: char = '│';
```

Add a field to `struct Table`, after `show_header`:

```rust
    /// Whether a `SEPARATOR` is drawn between columns.
    separators: bool,
```

Initialise it in `Table::new` with `separators: false,`.

Add the public methods after `set_show_header`:

```rust
    /// Draw a [`SEPARATOR`] in the one-cell gap between each pair of visible
    /// columns. Off by default. Columns stay where they are, so clicks land
    /// on the same cells either way.
    pub fn set_separators(&mut self, on: bool) {
        self.separators = on;
    }

    /// Whether separators are drawn.
    pub fn separators(&self) -> bool {
        self.separators
    }
```

Add a private helper after `write_row`:

```rust
    /// Put a separator into each gap between two visible columns of one
    /// drawn line, in the colour the line already has there. The gap after
    /// the last visible column is not between two columns and stays blank.
    fn write_separators(&self, buf: &mut DrawBuffer, width: usize) {
        if !self.separators {
            return;
        }
        let offsets = self.column_offsets();
        for &(x, w) in offsets.iter().take(offsets.len().saturating_sub(1)) {
            let gap = x + usize::from(w);
            if gap < width {
                let attr = buf.data[gap].attr;
                buf.put_char(gap, SEPARATOR, attr);
            }
        }
    }
```

In `draw`, call it just before each `write_line_to_terminal`. Header: after `self.write_row(...)` add `self.write_separators(&mut buf, width);`. Body: after the `if let Some(row) = ... { self.write_row(...); }` block, and before `write_line_to_terminal(terminal, 0, y + screen_row as i16, &buf);`, add `self.write_separators(&mut buf, width);`. Blank lines below the data get separators too, so the column lines run the full height, as `Grid` drew them.

In `TableBuilder`, add the field `separators: bool,` (initialised `false` in `TableBuilder::new`), the method

```rust
    #[must_use]
    pub fn separators(mut self, on: bool) -> Self {
        self.separators = on;
        self
    }
```

and in `build`, after `table.set_show_header(self.show_header);`, add `table.set_separators(self.separators);`.

- [ ] **Step 4: Run the tests**

Run: `cargo test --lib views::table 2>&1 | tail -5`
Expected: all `views::table` tests pass, old and new.

- [ ] **Step 5: Commit**

```bash
git add src/views/table.rs
git commit -m "feat(table): optional column separators

Moves tv-extensions' Grid separator into Table as a flag, off by default.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Table lazy row source

**Files:**
- Modify: `src/views/table.rs`
- Test: `src/views/table.rs` (`mod tests`)

**Interfaces:**
- Consumes: Task 2's `Table` (unchanged API).
- Produces: `pub trait RowProvider { fn rows(&self) -> usize; fn cell(&self, row: usize, col: usize) -> String; }`, `Table::set_provider(&mut self, provider: Box<dyn RowProvider>)`, `Table::refresh_rows(&mut self)`, and the **changed** `Table::selected_cell(&self) -> Option<String>` (was `Option<&str>`).

- [ ] **Step 1: Write the failing tests**

Append inside `mod tests`:

```rust
    use std::cell::Cell as Counter;
    use std::rc::Rc;

    /// Row `n` is `[n, n²]`; counts how many cells were asked for.
    struct Squares {
        rows: usize,
        calls: Rc<Counter<usize>>,
    }

    impl RowProvider for Squares {
        fn rows(&self) -> usize {
            self.rows
        }
        fn cell(&self, row: usize, col: usize) -> String {
            self.calls.set(self.calls.get() + 1);
            match col {
                0 => row.to_string(),
                1 => (row * row).to_string(),
                _ => String::new(),
            }
        }
    }

    fn squares(rows: usize) -> (Table, Rc<Counter<usize>>) {
        let calls = Rc::new(Counter::new(0));
        let mut t = Table::new(Rect::new(0, 0, 30, 6), 0);
        t.set_columns(vec![Column::new("N", 10), Column::right("N2", 12)]);
        t.set_provider(Box::new(Squares { rows, calls: Rc::clone(&calls) }));
        t.set_state(State::FOCUSED);
        (t, calls)
    }

    #[test]
    fn a_provider_supplies_rows_lazily() {
        let (mut t, calls) = squares(1_000_000);
        assert_eq!(t.row_count(), 1_000_000);
        t.set_selected_row(999_999);
        assert_eq!(t.selected_cell().as_deref(), Some("999999"));

        calls.set(0);
        let term = draw(&mut t, 30, 6);
        // Five visible rows times two columns, never the whole source.
        assert!(calls.get() <= 10, "asked for {} cells", calls.get());
        let shown = (1..6).any(|y| {
            let line: String = (0..10).map(|x| ch(&term, x, y)).collect();
            line.trim_end() == "999999"
        });
        assert!(shown, "the focused last row is on screen");
    }

    #[test]
    fn a_shorter_provider_clamps_the_focus() {
        let (mut t, _) = squares(100);
        t.set_selected_row(99);
        t.set_provider(Box::new(Squares { rows: 3, calls: Rc::new(Counter::new(0)) }));
        assert_eq!(t.selected_row(), Some(2));
        // Drawing after the swap must not index past the new end.
        let term = draw(&mut t, 30, 6);
        assert!(ch(&term, 0, 1).is_ascii_digit());
    }

    #[test]
    fn refresh_rows_picks_up_a_grown_source() {
        struct Growing(Rc<Counter<usize>>);
        impl RowProvider for Growing {
            fn rows(&self) -> usize {
                self.0.get()
            }
            fn cell(&self, row: usize, _col: usize) -> String {
                row.to_string()
            }
        }
        let len = Rc::new(Counter::new(2));
        let mut t = Table::new(Rect::new(0, 0, 30, 6), 0);
        t.set_columns(vec![Column::new("N", 10)]);
        t.set_provider(Box::new(Growing(Rc::clone(&len))));
        len.set(5);
        assert_eq!(t.row_count(), 2, "cached until refreshed");
        t.refresh_rows();
        assert_eq!(t.row_count(), 5);
    }

    #[test]
    fn set_rows_and_add_row_replace_a_provider() {
        let (mut t, _) = squares(10);
        t.add_row(vec!["only".into(), "row".into()]);
        assert_eq!(t.row_count(), 1);
        assert_eq!(t.selected_cell().as_deref(), Some("only"));

        let (mut t, _) = squares(10);
        t.set_rows(vec![vec!["a".into()], vec!["b".into()]]);
        assert_eq!(t.row_count(), 2);
    }

    #[test]
    fn a_ragged_owned_row_still_reports_no_cell() {
        let mut t = table(1);
        t.set_rows(vec![vec!["short".into()]]);
        t.set_selected_col(2);
        assert_eq!(t.selected_cell(), None);
    }
```

Change the existing assertion in `first_row_and_column_start_focused` from `assert_eq!(t.selected_cell(), Some("file0"));` to `assert_eq!(t.selected_cell().as_deref(), Some("file0"));`, and do the same for every other `selected_cell()` assertion in this module (`grep -n "selected_cell()" src/views/table.rs`).

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test --lib views::table 2>&1 | tail -5`
Expected: compile error, `cannot find trait RowProvider`.

- [ ] **Step 3: Implement**

Above `pub struct Table`, add:

```rust
/// A source of rows a [`Table`] reads only as it draws them, so a table can
/// browse more rows than would fit in memory as strings.
///
/// `cell` is only asked for rows below `rows()` and columns the table has.
/// A provider whose length changes must be followed by
/// [`Table::refresh_rows`].
pub trait RowProvider {
    /// How many rows there are.
    fn rows(&self) -> usize;
    /// The text of one cell.
    fn cell(&self, row: usize, col: usize) -> String;
}

/// Where a table's rows come from.
enum Rows {
    /// Rows held by the table, as `set_rows` and `add_row` give them.
    Owned(Vec<Vec<String>>),
    /// Rows read on demand.
    Provided(Box<dyn RowProvider>),
}

impl Rows {
    fn len(&self) -> usize {
        match self {
            Rows::Owned(rows) => rows.len(),
            Rows::Provided(p) => p.rows(),
        }
    }

    /// The cell, or `None` past the end of the row (an owned ragged row) or
    /// of the table.
    fn get(&self, row: usize, col: usize) -> Option<String> {
        match self {
            Rows::Owned(rows) => rows.get(row)?.get(col).cloned(),
            Rows::Provided(p) => (row < p.rows()).then(|| p.cell(row, col)),
        }
    }
}
```

In `struct Table`, change `rows: Vec<Vec<String>>,` to `rows: Rows,`, and in `Table::new` use `rows: Rows::Owned(Vec::new()),`.

Replace the row-data methods:

```rust
    pub fn set_rows(&mut self, rows: Vec<Vec<String>>) {
        self.rows = Rows::Owned(rows);
        self.list_state.set_range(self.rows.len());
        self.scroll_row_into_view();
    }

    /// Append one row. On a provider-backed table this starts a new
    /// in-memory list holding just this row.
    pub fn add_row(&mut self, row: Vec<String>) {
        match &mut self.rows {
            Rows::Owned(rows) => rows.push(row),
            Rows::Provided(_) => self.rows = Rows::Owned(vec![row]),
        }
        self.list_state.set_range(self.rows.len());
    }

    pub fn clear_rows(&mut self) {
        self.rows = Rows::Owned(Vec::new());
        self.list_state.set_range(0);
        self.first_col = 0;
    }

    /// Read rows from `provider` instead of an in-memory list. The focus is
    /// kept where it was, clamped to the new length.
    pub fn set_provider(&mut self, provider: Box<dyn RowProvider>) {
        self.rows = Rows::Provided(provider);
        self.refresh_rows();
    }

    /// Re-read the row count, after a provider's source has grown or shrunk.
    pub fn refresh_rows(&mut self) {
        self.list_state.set_range(self.rows.len());
        self.scroll_row_into_view();
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Text of the focused cell, if there is one.
    pub fn selected_cell(&self) -> Option<String> {
        self.rows.get(self.list_state.focused?, self.focused_col)
    }
```

Keep each method's existing doc comment where it already had one (`set_rows`, `clear_rows`, `row_count`).

Update the remaining uses:
- `set_selected_row`: `if self.rows.is_empty()` → `if self.rows.len() == 0`. `self.rows.len() - 1` is unchanged.
- `move_row`: `if self.rows.is_empty()` → `if self.rows.len() == 0`. `self.rows.len()` is unchanged.
- `cell_at`: `if row >= self.rows.len()` is unchanged.
- `get_text` (the `ListViewer` impl):

```rust
    fn get_text(&self, item: usize, max_len: usize) -> String {
        if item >= self.rows.len() {
            return String::new();
        }
        let joined = (0..self.columns.len())
            .filter_map(|col| self.rows.get(item, col))
            .collect::<Vec<_>>()
            .join(" ");
        joined.chars().take(max_len).collect()
    }
```

- `draw`, the body loop: replace `if let Some(row) = self.rows.get(row_index) { self.write_row(&mut buf, width, |i| row.get(i).cloned().unwrap_or_default(), ...` with `if row_index < self.rows.len() { self.write_row(&mut buf, width, |i| self.rows.get(row_index, i).unwrap_or_default(), ...`. The attribute closure is unchanged.
- `TableBuilder` keeps its own `rows: Vec<Vec<String>>` field and calls `table.set_rows(self.rows)`. No change.

If `#[derive(...)]` on `Table` or any `impl Debug for Table` exists, add a manual `Debug` for `Rows` printing `Owned(len)` / `Provided(len)`. Run `grep -n "Debug" src/views/table.rs` to check.

Update the module docs at the top of `src/views/table.rs`: after the paragraph about focus being a cell, add:

```rust
//! Rows come from `set_rows`, or lazily from a [`RowProvider`] given to
//! [`Table::set_provider`], which asks only for the rows on screen.
```

- [ ] **Step 4: Run the tests**

Run: `cargo test --lib views::table 2>&1 | tail -5`
Expected: all pass.

Run: `cargo build --all-targets 2>&1 | grep -E "^error" -A6 | head -30`
Expected: no errors. If anything outside `table.rs` called `selected_cell()` and broke, append `.as_deref()` at that call site.

- [ ] **Step 5: Commit**

```bash
git add -A src/views/table.rs
git commit -m "feat(table): rows from a lazy RowProvider

Table::set_provider reads only the rows on screen, absorbing extras'
GridView. selected_cell now returns Option<String>.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: ListBox lazy item source

**Files:**
- Modify: `src/views/listbox.rs`
- Modify: `examples/list_components.rs:202` (only if it fails to build)
- Test: `src/views/listbox.rs` (`mod tests`)

**Interfaces:**
- Produces: `pub trait ListProvider { fn len(&self) -> usize; fn item(&self, index: usize) -> String; fn is_empty(&self) -> bool { self.len() == 0 } }`, `ListBox::set_provider(&mut self, provider: Box<dyn ListProvider>)`, `ListBox::refresh_items(&mut self)`, and the **changed** `ListBox::get_selected_item(&self) -> Option<String>` (was `Option<&str>`) and `ListBox::marked_text(&self) -> Vec<String>` (was `Vec<&str>`).

- [ ] **Step 1: Write the failing tests**

Append inside `mod tests` in `src/views/listbox.rs`:

```rust
    struct Numbers(usize);

    impl ListProvider for Numbers {
        fn len(&self) -> usize {
            self.0
        }
        fn item(&self, index: usize) -> String {
            format!("Item {index}")
        }
    }

    #[test]
    fn a_provider_supplies_items() {
        let mut lb = ListBox::new(Rect::new(0, 0, 20, 5), 0);
        lb.set_provider(Box::new(Numbers(1_000_000)));
        assert_eq!(lb.item_count(), 1_000_000);
        lb.set_selection(999_999);
        assert_eq!(lb.get_selected_item().as_deref(), Some("Item 999999"));

        let mut term = crate::test_util::test_terminal(20, 5);
        lb.draw(&mut term);
        let shown = (0..5).any(|y| {
            let line: String = (0..20).map(|x| term.read_cell(x, y).unwrap().ch).collect();
            line.trim_end() == "Item 999999"
        });
        assert!(shown);
    }

    #[test]
    fn a_shorter_provider_clamps_the_selection_and_drops_marks() {
        let mut lb = ListBox::new(Rect::new(0, 0, 20, 5), 0);
        lb.set_provider(Box::new(Numbers(50)));
        lb.set_selection(49);
        lb.set_marked(10, true);
        lb.set_provider(Box::new(Numbers(3)));
        assert_eq!(lb.get_selection(), Some(2));
        assert_eq!(lb.marked_count(), 0);
        let mut term = crate::test_util::test_terminal(20, 5);
        lb.draw(&mut term);
    }

    #[test]
    fn marked_text_reads_through_a_provider() {
        let mut lb = ListBox::new(Rect::new(0, 0, 20, 5), 0);
        lb.set_provider(Box::new(Numbers(10)));
        lb.set_marked(2, true);
        lb.set_marked(7, true);
        assert_eq!(lb.marked_text(), vec!["Item 2", "Item 7"]);
    }

    #[test]
    fn add_item_on_a_provider_starts_a_new_list() {
        let mut lb = ListBox::new(Rect::new(0, 0, 20, 5), 0);
        lb.set_provider(Box::new(Numbers(10)));
        lb.add_item("solo".into());
        assert_eq!(lb.item_count(), 1);
        assert_eq!(lb.get_selected_item().as_deref(), Some("solo"));
    }
```

Change the existing assertions that compare `get_selected_item()` with `Some("...")` (around lines 504 and 559) to use `.as_deref()`. The `marked_text()` assertion around line 634 compiles unchanged, because `Vec<String> == Vec<&str>` is implemented.

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test --lib views::listbox 2>&1 | tail -5`
Expected: compile error, `cannot find trait ListProvider`.

- [ ] **Step 3: Implement**

Above `pub struct ListBox`, add:

```rust
/// A source of items a [`ListBox`] reads only as it draws them.
///
/// `item` is only asked for indices below `len()`. A provider whose length
/// changes must be followed by [`ListBox::refresh_items`].
pub trait ListProvider {
    /// How many items there are.
    fn len(&self) -> usize;
    /// The text of one item.
    fn item(&self, index: usize) -> String;
    /// Whether there are no items.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Where a list box's items come from.
enum Items {
    Owned(Vec<String>),
    Provided(Box<dyn ListProvider>),
}

impl Items {
    fn len(&self) -> usize {
        match self {
            Items::Owned(items) => items.len(),
            Items::Provided(p) => p.len(),
        }
    }

    fn get(&self, index: usize) -> Option<String> {
        match self {
            Items::Owned(items) => items.get(index).cloned(),
            Items::Provided(p) => (index < p.len()).then(|| p.item(index)),
        }
    }
}
```

In `struct ListBox`, `items: Vec<String>,` becomes `items: Items,`. In `ListBox::new`, use `items: Items::Owned(Vec::new()),`.

Rewrite every `self.items` use with these rules (`grep -n "self.items" src/views/listbox.rs` lists them all):
- `self.items.len()` stays as written (`Items::len`).
- `self.items.is_empty()` becomes `self.items.len() == 0`.
- `self.items.get(i)` returning `Option<&String>` becomes `self.items.get(i)` returning `Option<String>`. Drop `.cloned()`, `.map(|s| s.as_str())` and `.map(|s| &**s)`.
- `self.items[i]` (in `draw`) becomes `self.items.get(i).unwrap_or_default()`.

The methods that change shape:

```rust
    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = Items::Owned(items);
        // keep the existing body after the assignment: marks dropped, range set
    }

    /// Append one item. On a provider-backed list this starts a new
    /// in-memory list holding just this item.
    pub fn add_item(&mut self, item: String) {
        match &mut self.items {
            Items::Owned(items) => items.push(item),
            Items::Provided(_) => {
                self.items = Items::Owned(vec![item]);
                self.marked.clear();
            }
        }
        self.list_state.set_range(self.items.len());
    }

    pub fn clear(&mut self) {
        self.items = Items::Owned(Vec::new());
        // keep the existing body after the assignment
    }

    /// Read items from `provider` instead of an in-memory list. Marks are
    /// dropped, because their indices refer to the old items; the focus is
    /// clamped to the new length.
    pub fn set_provider(&mut self, provider: Box<dyn ListProvider>) {
        self.items = Items::Provided(provider);
        self.marked.clear();
        self.refresh_items();
    }

    /// Re-read the item count after a provider's source changed length.
    /// Marks past the new end are dropped.
    pub fn refresh_items(&mut self) {
        let len = self.items.len();
        self.marked.retain(|&i| i < len);
        self.list_state.set_range(len);
    }

    /// The marked items' text, in list order.
    pub fn marked_text(&self) -> Vec<String> {
        self.marked.iter().filter_map(|&i| self.items.get(i)).collect()
    }

    /// Get the currently selected item text
    pub fn get_selected_item(&self) -> Option<String> {
        self.items.get(self.list_state.focused?)
    }
```

`set_items` and `clear` keep their current bodies after the first line. Only the assignment changes. The `ListViewer::get_text` impl becomes `self.items.get(item).unwrap_or_default()`.

- [ ] **Step 4: Run the tests and build everything**

Run: `cargo test --lib views::listbox 2>&1 | tail -5`
Expected: all pass.

Run: `cargo build --all-targets 2>&1 | grep -E "^error" -A6 | head -30`
Expected: no errors. If `examples/list_components.rs:202` fails, `format!("Selected: {}", selected)` already accepts a `String`, so any error there is a borrow of the old `&str`; bind the value with `if let Some(selected) = listbox.get_selected_item()` as-is and fix only the reported line. `SortedListBox` and `HistoryViewer` have their own `get_selected_item` and are not affected.

- [ ] **Step 5: Commit**

```bash
git add -A src/views/listbox.rs examples/list_components.rs
git commit -m "feat(listbox): items from a lazy ListProvider

ListBox::set_provider absorbs extras' VirtualListBox. get_selected_item
and marked_text now return owned strings.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Slider in core

**Files:**
- Create: `src/views/slider.rs`
- Modify: `src/views/mod.rs` (add `pub mod slider;` next to `pub mod spinner;`, and `pub use slider::Slider;` next to `pub use spinner::Spinner;`)
- Test: `src/views/slider.rs` (`mod tests`)

**Interfaces:**
- Produces: `Slider::new(bounds: Rect, min: i64, max: i64) -> Slider`, `value(&self) -> i64`, `set_value(&mut self, v: i64) -> bool` (true when changed), `range(&self) -> (i64, i64)`, `set_step(&mut self, step: i64)`, `set_on_change(&mut self, command: CommandId)` (0 = none). Exported as `turbo_vision::views::Slider`.

- [ ] **Step 1: Write the file with its tests first**

Create `src/views/slider.rs` holding the tests below and a stub `impl` (only `new`, `value`, `range` returning defaults), so the tests compile and fail on behaviour:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::{KB_END, KB_HOME, KB_LEFT, KB_RIGHT, MB_LEFT_BUTTON};
    use crate::core::geometry::Point;

    fn focused(min: i64, max: i64, width: i16) -> Slider {
        let mut s = Slider::new(Rect::new(0, 0, width, 1), min, max);
        s.set_focus(true);
        s
    }

    fn key(s: &mut Slider, code: u16) -> Event {
        let mut e = Event::keyboard(code);
        s.handle_event(&mut e);
        e
    }

    fn click(s: &mut Slider, x: i16) -> Event {
        let mut e = Event::nothing();
        e.what = EventType::MouseDown;
        e.mouse.pos = Point::new(x, 0);
        e.mouse.buttons = MB_LEFT_BUTTON;
        s.handle_event(&mut e);
        e
    }

    #[test]
    fn keys_step_and_clamp() {
        let mut s = focused(0, 10, 11);
        s.set_step(3);
        assert_eq!(key(&mut s, KB_RIGHT).what, EventType::Nothing);
        assert_eq!(s.value(), 3);
        key(&mut s, KB_END);
        assert_eq!(s.value(), 10);
        key(&mut s, KB_RIGHT);
        assert_eq!(s.value(), 10);
        key(&mut s, KB_HOME);
        key(&mut s, KB_LEFT);
        assert_eq!(s.value(), 0);
    }

    #[test]
    fn a_click_sets_the_value_under_the_mouse() {
        // 11 cells over 0..=10: one value per cell.
        let mut s = focused(0, 10, 11);
        click(&mut s, 7);
        assert_eq!(s.value(), 7);
        click(&mut s, 50);
        assert_eq!(s.value(), 7, "clicks outside the view are ignored");
    }

    #[test]
    fn on_change_broadcasts_only_real_changes() {
        let mut s = focused(0, 10, 11);
        s.set_on_change(900);
        let e = key(&mut s, KB_RIGHT);
        assert_eq!((e.what, e.command), (EventType::Broadcast, 900));
        key(&mut s, KB_END);
        let e = key(&mut s, KB_RIGHT);
        assert_eq!(e.what, EventType::Nothing, "already at max");
    }

    #[test]
    fn the_thumb_tracks_the_value() {
        let mut s = focused(0, 10, 11);
        s.set_value(10);
        let mut term = crate::test_util::test_terminal(11, 1);
        s.draw(&mut term);
        assert_eq!(term.read_cell(10, 0).unwrap().ch, THUMB);
        assert_eq!(term.read_cell(0, 0).unwrap().ch, TRACK);
    }

    #[test]
    fn degenerate_ranges_do_not_panic() {
        let s = Slider::new(Rect::new(0, 0, 10, 1), 9, 2);
        assert_eq!(s.range(), (2, 9), "reversed bounds are swapped");

        for (min, max, width) in [(5, 5, 10), (0, 100, 1), (i64::MIN, i64::MAX, 20)] {
            let mut s = focused(min, max, width);
            key(&mut s, KB_END);
            key(&mut s, KB_LEFT);
            click(&mut s, width - 1);
            let mut term = crate::test_util::test_terminal(width as u16, 1);
            s.draw(&mut term);
            let thumbs = (0..width).filter(|&x| term.read_cell(x, 0).unwrap().ch == THUMB).count();
            assert_eq!(thumbs, 1, "one thumb inside the view for {min}..={max} in {width}");
        }
    }
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test --lib views::slider 2>&1 | tail -8`
Expected: failures (or compile errors for the missing `THUMB`, `TRACK`, `set_step`, `set_on_change`, `set_value`).

- [ ] **Step 3: Implement**

The full file above the tests:

```rust
// (C) 2026 - Enzo Lombardi

//! Slider view - a horizontal track with a thumb that picks one integer.
//!
//! Not part of the Borland Turbo Vision widget set; the control comes from the
//! TV Tool Box add-on library. It is the dragging counterpart of `Spinner`.
//!
//! # Keys
//!
//! | Key | Action |
//! |-----|--------|
//! | Left, Right | One step down or up |
//! | Home, End | Minimum or maximum |
//!
//! Clicking or dragging on the track moves the thumb under the mouse.

use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::command::CommandId;
use crate::core::draw::DrawBuffer;
use crate::core::event::{Event, EventType, KB_END, KB_HOME, KB_LEFT, KB_RIGHT, MB_LEFT_BUTTON};
use crate::core::geometry::Rect;
use crate::core::palette::{INPUT_ARROWS, INPUT_NORMAL, INPUT_SELECTED};
use crate::core::state::{State, StateFlags};
use crate::terminal::Terminal;

/// The track character.
pub const TRACK: char = '─';
/// The thumb character. A CP437 glyph, so it also shows in PNG captures.
pub const THUMB: char = '■';

/// A horizontal value slider over `min..=max`.
pub struct Slider {
    core: ViewCore,
    view_state: StateFlags,
    min: i64,
    max: i64,
    value: i64,
    step: i64,
    /// Broadcast on every user change; 0 sends nothing.
    on_change: CommandId,
}

impl Slider {
    /// A slider over `min..=max`, starting at `min`. Reversed bounds are
    /// swapped.
    pub fn new(bounds: Rect, min: i64, max: i64) -> Self {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        Self {
            core: ViewCore {
                bounds,
                palette_chain: None,
                ..ViewCore::default()
            },
            view_state: State::empty(),
            min,
            max,
            value: min,
            step: 1,
            on_change: 0,
        }
    }

    /// The current value.
    pub fn value(&self) -> i64 {
        self.value
    }

    /// Set the value, clamped to the range. Returns whether it changed.
    pub fn set_value(&mut self, value: i64) -> bool {
        let value = value.clamp(self.min, self.max);
        let changed = value != self.value;
        self.value = value;
        changed
    }

    /// The bounds, smallest first.
    pub fn range(&self) -> (i64, i64) {
        (self.min, self.max)
    }

    /// How far Left and Right move. At least 1.
    pub fn set_step(&mut self, step: i64) {
        self.step = step.max(1);
    }

    /// Broadcast `command` whenever the user changes the value; 0 turns it off.
    pub fn set_on_change(&mut self, command: CommandId) {
        self.on_change = command;
    }

    /// The span of the range, in a type that cannot overflow.
    fn span(&self) -> i128 {
        i128::from(self.max) - i128::from(self.min)
    }

    /// The column the thumb sits in, for a track `width` cells wide.
    fn thumb_col(&self, width: usize) -> usize {
        let track = i128::try_from(width.saturating_sub(1)).unwrap_or(0);
        let span = self.span();
        if track == 0 || span == 0 {
            return 0;
        }
        let offset = i128::from(self.value) - i128::from(self.min);
        usize::try_from(track * offset / span).unwrap_or(0)
    }

    /// The value under column `col` of a track `width` cells wide.
    fn value_at(&self, col: i16, width: usize) -> i64 {
        let track = i128::try_from(width.saturating_sub(1)).unwrap_or(0);
        if track == 0 {
            return self.min;
        }
        let col = i128::from(col).clamp(0, track);
        let value = i128::from(self.min) + self.span() * col / track;
        i64::try_from(value).unwrap_or(self.max)
    }

    fn report(&self, event: &mut Event, changed: bool) {
        if changed && self.on_change != 0 {
            *event = Event::broadcast(self.on_change);
        } else {
            event.clear();
        }
    }
}

impl View for Slider {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn state(&self) -> StateFlags {
        self.view_state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.view_state = state;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = usize::try_from(self.core.bounds.width_clamped()).unwrap_or(0);
        if width == 0 {
            return;
        }
        // Like Spinner: no cursor, so focus borrows the selected-text colour.
        let track_attr = if self.is_focused() {
            self.map_color(INPUT_SELECTED)
        } else {
            self.map_color(INPUT_NORMAL)
        };
        let thumb_attr = self.map_color(INPUT_ARROWS);

        let mut buf = DrawBuffer::new(width);
        buf.move_char(0, TRACK, track_attr, width);
        buf.put_char(self.thumb_col(width), THUMB, thumb_attr);
        write_line_to_terminal(terminal, 0, 0, &buf);
    }

    fn handle_event(&mut self, event: &mut Event) {
        let width = usize::try_from(self.core.bounds.width_clamped()).unwrap_or(0);
        match event.what {
            EventType::MouseDown | EventType::MouseMove
                if event.mouse.buttons & MB_LEFT_BUTTON != 0
                    && self.extent().contains(event.mouse.pos) =>
            {
                let changed = self.set_value(self.value_at(event.mouse.pos.x, width));
                self.report(event, changed);
            }
            EventType::Keyboard if self.is_focused() => {
                let target = match event.key_code {
                    KB_LEFT => self.value.saturating_sub(self.step),
                    KB_RIGHT => self.value.saturating_add(self.step),
                    KB_HOME => self.min,
                    KB_END => self.max,
                    _ => return,
                };
                let changed = self.set_value(target);
                self.report(event, changed);
            }
            _ => {}
        }
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_INPUT_LINE))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
```

Check `Spinner`'s `impl View` for any other required trait method it overrides (`grep -n "    fn " src/views/spinner.rs`) and mirror it if the compiler asks for it.

Register the module in `src/views/mod.rs`: `pub mod slider;` in alphabetical order next to `pub mod spinner;`, and `pub use slider::Slider;` next to `pub use spinner::Spinner;`.

- [ ] **Step 4: Run the tests**

Run: `cargo test --lib views::slider 2>&1 | tail -5`
Expected: 5 passed.

- [ ] **Step 5: Commit**

```bash
git add src/views/slider.rs src/views/mod.rs
git commit -m "feat(views): Slider joins the core controls

Moved from turbo-vision-extras; uses the input-line palette like Spinner
and a CP437 thumb.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: Terminal event injector

**Files:**
- Modify: `src/terminal/mod.rs` (struct `Terminal` fields near line 145, its constructor near line 238, `enable_remote_input` near line 624)
- Test: `src/terminal/mod.rs` (`mod tests`)

**Interfaces:**
- Produces: `Terminal::event_injector(&mut self) -> std::sync::mpsc::Sender<Event>`. `enable_remote_input` becomes a user of it.

- [ ] **Step 1: Write the failing test**

Append inside `mod tests` in `src/terminal/mod.rs`:

```rust
    #[test]
    fn injected_events_arrive_in_order_from_any_sender() {
        use crate::core::event::KB_ENTER;
        let mut t = crate::test_util::test_terminal(20, 10);
        let first = t.event_injector();
        let second = t.event_injector();
        std::thread::spawn(move || first.send(Event::keyboard(KB_ENTER)).unwrap())
            .join()
            .unwrap();
        second.send(Event::command(1234)).unwrap();

        let a = t.poll_event(std::time::Duration::ZERO).unwrap().unwrap();
        let b = t.poll_event(std::time::Duration::ZERO).unwrap().unwrap();
        assert_eq!(a.key_code, KB_ENTER);
        assert_eq!(b.command, 1234);
        assert!(t.poll_event(std::time::Duration::ZERO).unwrap().is_none());
    }
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test --lib terminal::tests::injected 2>&1 | tail -5`
Expected: compile error, `no method named event_injector`.

- [ ] **Step 3: Implement**

Next to the `injected_rx` field, add:

```rust
    /// The sending half of `injected_rx`, kept so every `event_injector`
    /// call hands out a clone of one channel.
    injected_tx: Option<std::sync::mpsc::Sender<Event>>,
```

Initialise it as `injected_tx: None,` wherever `injected_rx: None,` is set.

Add the method above `enable_remote_input`, and rewrite `enable_remote_input` to use it:

```rust
    /// A sender that queues events for [`poll_event`](Self::poll_event), as
    /// if they had been typed. Safe to use from another thread; every call
    /// returns a clone of the same channel, and injected events are served
    /// after a [`put_event`](Self::put_event) one and before the backend's.
    ///
    /// This is the hook automation and remote-input listeners build on.
    pub fn event_injector(&mut self) -> std::sync::mpsc::Sender<Event> {
        if let Some(tx) = &self.injected_tx {
            return tx.clone();
        }
        let (tx, rx) = std::sync::mpsc::channel();
        self.injected_rx = Some(rx);
        self.injected_tx = Some(tx.clone());
        tx
    }

    pub fn enable_remote_input(&mut self, port: u16) -> io::Result<()> {
        let tx = self.event_injector();
        remote_input::spawn(port, tx)
    }
```

Keep the existing doc comment on `enable_remote_input`. Update the `injected_rx` field comment to: `/// Receiver for events queued through [`event_injector`](Self::event_injector).`

- [ ] **Step 4: Run the tests**

Run: `cargo test --lib terminal 2>&1 | tail -5`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add src/terminal/mod.rs
git commit -m "feat(terminal): event_injector, a public event-injection hook

Remote input now builds on it, as tv-extensions will.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: Capture hook

**Files:**
- Modify: `src/terminal/mod.rs` (new types near the `Terminal` struct, a new field, `capture_injected`)
- Modify: `src/app/application.rs` (the `KB_F12` and `KB_CTRL_F12` arms near line 870, the `CM_SCREENSHOT` arm near line 943)
- Modify: `src/terminal/mod.rs` exports and `src/lib.rs` prelude only if `Terminal` types are re-exported there (`grep -n "pub use" src/terminal/mod.rs`)
- Test: `src/terminal/mod.rs` (`mod tests`) and a new `#[cfg(test)] mod capture_tests` at the end of `src/app/application.rs`

**Interfaces:**
- Consumes: Task 6's `event_injector`.
- Produces: `pub enum CaptureKind { Png, Ansi }`, `pub type CaptureHook = Box<dyn FnMut(CaptureKind, &Terminal)>`, `Terminal::set_capture_hook(&mut self, hook: CaptureHook)`, `Terminal::clear_capture_hook(&mut self) -> Option<CaptureHook>`, `Terminal::run_capture_hook(&mut self, kind: CaptureKind) -> bool`. All exported from `turbo_vision::terminal`.

- [ ] **Step 1: Write the failing tests**

In `src/terminal/mod.rs` `mod tests`:

```rust
    #[test]
    fn without_a_hook_run_capture_hook_reports_false() {
        let mut t = crate::test_util::test_terminal(20, 10);
        assert!(!t.run_capture_hook(CaptureKind::Png));
    }

    #[test]
    fn injected_capture_chords_go_to_the_hook() {
        use crate::core::event::{KB_CTRL_F12, KB_F12};
        use std::cell::RefCell;
        use std::rc::Rc;

        let mut t = crate::test_util::test_terminal(20, 10);
        let seen = Rc::new(RefCell::new(Vec::new()));
        let log = Rc::clone(&seen);
        t.set_capture_hook(Box::new(move |kind, term| log.borrow_mut().push((kind, term.size()))));
        let tx = t.event_injector();
        tx.send(Event::keyboard(KB_CTRL_F12)).unwrap();
        tx.send(Event::keyboard(KB_F12)).unwrap();

        assert!(t.poll_event(std::time::Duration::ZERO).unwrap().is_none());
        assert!(t.poll_event(std::time::Duration::ZERO).unwrap().is_none());
        assert_eq!(
            *seen.borrow(),
            vec![(CaptureKind::Png, (20, 10)), (CaptureKind::Ansi, (20, 10))]
        );
        assert!(t.clear_capture_hook().is_some());
        assert!(!t.run_capture_hook(CaptureKind::Png));
    }
```

At the end of `src/app/application.rs`:

```rust
#[cfg(test)]
mod capture_tests {
    use super::*;
    use crate::core::command::CM_SCREENSHOT;
    use crate::terminal::CaptureKind;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn capture_keys_run_the_capture_hook() {
        let mut app = Application::with_terminal(crate::test_util::test_terminal(80, 25));
        let seen = Rc::new(RefCell::new(Vec::new()));
        let log = Rc::clone(&seen);
        app.terminal
            .set_capture_hook(Box::new(move |kind, _| log.borrow_mut().push(kind)));

        for mut event in [
            Event::keyboard(KB_CTRL_F12),
            Event::keyboard(KB_F12),
            Event::command(CM_SCREENSHOT),
        ] {
            app.handle_event(&mut event);
        }
        assert_eq!(*seen.borrow(), vec![CaptureKind::Png, CaptureKind::Ansi, CaptureKind::Png]);
    }
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test --lib capture 2>&1 | tail -5`
Expected: compile error, `cannot find type CaptureKind`.

- [ ] **Step 3: Implement in the terminal**

Above `pub struct Terminal`:

```rust
/// Which capture the user asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureKind {
    /// Ctrl+F12: an image of the screen.
    Png,
    /// F12: a text dump of the screen with ANSI colours.
    Ansi,
}

/// Called with the kind of capture and the terminal to capture. Install one
/// with [`Terminal::set_capture_hook`].
pub type CaptureHook = Box<dyn FnMut(CaptureKind, &Terminal)>;
```

Add a field after `injected_tx`: `capture_hook: Option<CaptureHook>,`, initialised to `None` alongside the others.

Add the methods after `event_injector`:

```rust
    /// Handle Ctrl+F12 and F12 with `hook` instead of the built-in capture.
    /// The hook gets the whole terminal, so it can read `buffer()` and
    /// `size()`. Replaces any earlier hook.
    pub fn set_capture_hook(&mut self, hook: CaptureHook) {
        self.capture_hook = Some(hook);
    }

    /// Remove the capture hook, returning it.
    pub fn clear_capture_hook(&mut self) -> Option<CaptureHook> {
        self.capture_hook.take()
    }

    /// Run the capture hook for `kind`. Returns `false`, doing nothing,
    /// when no hook is installed.
    pub fn run_capture_hook(&mut self, kind: CaptureKind) -> bool {
        let Some(mut hook) = self.capture_hook.take() else {
            return false;
        };
        hook(kind, self);
        self.capture_hook = Some(hook);
        true
    }
```

Change `fn capture_injected(&self, png: bool)` to `fn capture_injected(&mut self, png: bool)`, and make its first lines:

```rust
        let kind = if png { CaptureKind::Png } else { CaptureKind::Ansi };
        if self.run_capture_hook(kind) {
            return;
        }
```

Leave the rest of its body (the built-in capture) unchanged. Plan 3 deletes it. If the borrow checker rejects the call inside `poll_event`'s `if let Some(rx) = &self.injected_rx` block, receive first and then act:

```rust
        let injected = self.injected_rx.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some(event) = injected {
            match event.key_code {
                crate::core::event::KB_CTRL_F12 => {
                    self.capture_injected(true);
                    return Ok(None);
                }
                crate::core::event::KB_F12 => {
                    self.capture_injected(false);
                    return Ok(None);
                }
                _ => return Ok(Some(event)),
            }
        }
```

Export the new types where `Terminal` is exported. If `src/terminal/mod.rs` defines `Terminal` itself, `pub enum` and `pub type` there are already reachable as `turbo_vision::terminal::{CaptureKind, CaptureHook}`. Confirm with `grep -n "pub struct Terminal" src/terminal/mod.rs`.

- [ ] **Step 4: Implement in the application**

In `src/app/application.rs`, add `use crate::terminal::CaptureKind;` to the imports. Then change the three arms:

```rust
                KB_F12 => {
                    if !self.terminal.run_capture_hook(CaptureKind::Ansi) {
                        self.dump_screen_ansi();
                    }
                    event.clear();
                    return;
                }
                KB_CTRL_F12 => {
                    if !self.terminal.run_capture_hook(CaptureKind::Png) {
                        self.take_screenshot();
                    }
                    event.clear();
                    return;
                }
```

and in the command arm:

```rust
                CM_SCREENSHOT => {
                    if !self.terminal.run_capture_hook(CaptureKind::Png) {
                        self.take_screenshot();
                    }
                    // keep whatever the arm did after take_screenshot (event.clear())
                }
```

Read each arm before editing and keep its existing trailing statements. Only the `take_screenshot()`/`dump_screen_ansi()` call gets wrapped.

- [ ] **Step 5: Run the tests**

Run: `cargo test --lib capture 2>&1 | tail -5` and `cargo test --lib terminal 2>&1 | tail -3`
Expected: all pass, and no `screenshot-*.png` or `screen-*.ans` file appears in the working directory (`ls screenshot-* screen-*.ans 2>/dev/null` prints nothing).

- [ ] **Step 6: Commit**

```bash
git add src/terminal/mod.rs src/app/application.rs
git commit -m "feat(terminal): capture hook for Ctrl+F12 and F12

Application and injected chords try the hook before the built-in capture,
which plan 3 moves to tv-extensions.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 8: Raw output passthrough and a public InputParser

**Files:**
- Modify: `src/terminal/mod.rs` (`write_kitty_graphics` near line 856; `mod input_parser` and `pub use input_parser::InputParser` near lines 59–68)
- Test: `src/terminal/mod.rs` (`mod tests`, reusing `RecordingBackend`)

**Interfaces:**
- Produces: `Terminal::write_raw(&mut self, data: &[u8]) -> io::Result<()>`, and `turbo_vision::terminal::InputParser` available without the `ssh` feature.

- [ ] **Step 1: Write the failing tests**

In `mod tests` (look at the existing test that builds `Terminal::with_backend(Box::new(RecordingBackend { ... }))` around line 983, and copy its construction):

```rust
    #[test]
    fn write_raw_reaches_the_backend_unchanged() {
        let written = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut terminal = Terminal::with_backend(Box::new(RecordingBackend {
            written: std::sync::Arc::clone(&written),
        }))
        .unwrap();
        written.lock().unwrap().clear();
        terminal.write_raw(b"\x1b_Ga=d\x1b\\").unwrap();
        assert_eq!(&*written.lock().unwrap(), b"\x1b_Ga=d\x1b\\");
    }

    #[test]
    fn input_parser_is_available_without_ssh() {
        let mut parser = InputParser::new();
        let events = parser.parse(b"a");
        assert_eq!(events.len(), 1);
    }
```

If `RecordingBackend` has more fields than `written`, fill them the way the existing test does.

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test --lib -- terminal::tests::write_raw terminal::tests::input_parser 2>&1 | tail -5`
Expected: compile errors, `no method named write_raw` and `cannot find type InputParser` (without the `ssh` feature).

- [ ] **Step 3: Implement**

Add above `write_kitty_graphics`:

```rust
    /// Send bytes straight to the terminal, bypassing the cell buffer, and
    /// flush. For protocols drawn outside the cells, such as Kitty graphics.
    ///
    /// # Errors
    ///
    /// Returns an error if the backend cannot write or flush.
    pub fn write_raw(&mut self, data: &[u8]) -> io::Result<()> {
        self.backend.write_raw(data)?;
        self.backend.flush()
    }
```

and make `write_kitty_graphics` call `self.write_raw(data)` instead of its two backend calls.

Remove the `#[cfg(feature = "ssh")]` line above `mod input_parser;` and the one above `pub use input_parser::InputParser;`. Leave the `ssh_backend` gates alone.

- [ ] **Step 4: Run the tests and the feature builds**

Run: `cargo test --lib terminal 2>&1 | tail -3`
Expected: all pass, including `input_parser`'s own tests, which now run in the default build.

Run: `cargo build --features ssh 2>&1 | grep -E "^(error|warning: unused)" -A4 | head`
Expected: nothing.

- [ ] **Step 5: Commit**

```bash
git add src/terminal/mod.rs
git commit -m "feat(terminal): write_raw passthrough; InputParser always public

The hooks Kitty graphics and the SSH backend need once they move out.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 9: Changelog, spec sync, full verification

**Files:**
- Modify: `CHANGELOG.md` (new section above `## [3.0.1]`)
- Modify: `docs/superpowers/specs/2026-10-01-core-extensions-split-design.md` ("Additions" bullets under "turbo-vision core 4.0")

- [ ] **Step 1: Add the changelog section**

Insert above `## [3.0.1] - 2026-09-17`:

```markdown
## [Unreleased]

### Added
- `Table::set_separators` / `TableBuilder::separators`: a `│` between visible
  columns, in each line's own colour. Off by default.
- `table::RowProvider` and `Table::set_provider` / `refresh_rows`: rows read
  only as they are drawn.
- `listbox::ListProvider` and `ListBox::set_provider` / `refresh_items`.
- `views::Slider`, moved in from turbo-vision-extras.
- Extension hooks on `Terminal`: `event_injector`, `set_capture_hook` /
  `clear_capture_hook` / `run_capture_hook` with `CaptureKind`, and
  `write_raw`. `terminal::InputParser` no longer needs the `ssh` feature.

### Changed
- `Table::selected_cell` returns `Option<String>`, and
  `ListBox::get_selected_item` / `marked_text` return owned strings, because
  rows and items can now come from a provider. Add `.as_deref()` where a
  `&str` is needed.
- Ctrl+F12, F12 and `CM_SCREENSHOT` run the capture hook when one is set,
  instead of the built-in capture.
```

- [ ] **Step 2: Bring the spec in line**

In the spec's "Additions" list:
- Replace "`set_rows`/`add_row` replace it with an in-memory `VecRowProvider`" with "`set_rows`/`add_row` switch back to in-memory rows (`add_row` on a provider-backed table starts a new list)". Add `refresh_rows()`, and note that `selected_cell` now returns `Option<String>`.
- For `ListBox`, add `refresh_items()`, and note that `get_selected_item`/`marked_text` return owned strings.
- Replace the capture hook signature `Box<dyn FnMut(CaptureKind, &[Vec<Cell>])>` with `Box<dyn FnMut(CaptureKind, &Terminal)>` (the PNG renderer needs the font-size query as well as the cells), and add `clear_capture_hook` and `run_capture_hook`.
- Note that the `Slider` thumb is `■`, a CP437 glyph.

- [ ] **Step 3: Full verification**

Run each command and check the result:
- `cargo test 2>&1 | grep -E "test result|FAILED|panicked" | sort | uniq -c`: every line says `ok`, none says `FAILED`.
- `cargo test --no-default-features --lib 2>&1 | tail -3`: `ok`.
- `cargo test --features ssh --lib 2>&1 | tail -3`: `ok`.
- `sh scripts/check-wasm.sh`: `Finished`.
- `cargo clippy --all-targets 2>&1 | grep -c "^warning"`, compared against the same count on `main` (`git stash; git switch main; ...; git switch -; git stash pop`, or a second worktree): no more warnings than on main.
- `cargo doc --no-deps 2>&1 | grep -E "^(warning|error)" | head`: no new broken intra-doc links.

- [ ] **Step 4: Commit**

```bash
git add CHANGELOG.md docs/superpowers/specs/2026-10-01-core-extensions-split-design.md
git commit -m "docs: changelog for the absorbed widgets and hooks; sync the spec

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

- [ ] **Step 5: Hand back**

Report to the user: the branch name `feat/core-absorb`, the commit list (`git log --oneline main..`), the verification results from Step 3, and that `main` has not been pushed unless they approved it in Task 1. Ask whether to open a PR or merge into `main`.
