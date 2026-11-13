# To do: the user guide walk-through does not compile

Chapters 1 to 6 of the [user guide](user-guide/) are a walk-through: one
application built in twelve steps, from an empty desktop to a data-entry
system with validation and a custom view. Most of its code no longer
compiles against the current API. A reader who copies Listing 1.1, the first
program, gets a compile error.

This file records what was found, on 2026-11-13, and the work needed. Nothing
in the chapters has been changed yet.

## What was found

The six chapters have 81 Rust code blocks:

| Result | Blocks | Meaning |
|--------|-------:|---------|
| Compile | 11 | Correct as written. |
| Use API that has changed | 41 | They call methods, constructors or constants that no longer exist, or pass the wrong arguments. These are the blocks to fix. |
| Need the earlier listings | 27 | They use variables, types or functions that an earlier listing defines (`app`, `Order`, `create_order_dialog`), or crates they never declare (`serde_json`, `dirs`, `ron`). They could not be checked on their own, so they may hide more changed API. |
| Not compilable as written | 2 | A bare `mod commands;` and a lone `match` arm. |

No chapter is close to compiling: Chapter 4 has no block that compiles, and
the best, Chapter 6, has 4 of 21.

The rest of the user guide was not checked. `AGENTS.md` does not have this
problem because it is compiled as doctests (`src/lib.rs`), so `cargo test`
fails when its code stops matching the API.

## How it was checked

Each block was compiled as a doctest, compile only, with every module of the
crate glob-imported, so that a missing `use` line could not hide a real
error. To repeat the check:

1. In a scratch copy of the repository, mark every Rust block of the six
   chapters as `rust,no_run`. A plain `rust` block runs as a doctest, and
   these programs would take over the terminal.
2. Add the hidden imports to each block, as `# use ...;` lines:
   `turbo_vision::prelude::*`, `turbo_vision::app::*`, and every module
   under `turbo_vision::views` and `turbo_vision::core`.
3. Include the edited chapters in `src/lib.rs`, the way `AGENTS.md` is
   included:

   ```rust,ignore
   #[cfg(doctest)]
   #[doc = include_str!("../doccheck/Chapter-01.md")]
   pub struct Chapter01;
   ```

4. Run `cargo test --doc Chapter0`.

## Tasks

Each task names the current replacement. The block checklist further down
says which tasks each block needs.

- [ ] **T1. `main` returns the crate's `Result`.** Several listings declare
  `fn main() -> std::io::Result<()>` and then call `Application::new()?`.
  The crate's error type, `TurboVisionError`, does not convert to
  `std::io::Error`. Use `fn main() -> turbo_vision::core::error::Result<()>`,
  as `AGENTS.md` does.
- [ ] **T2. Menus are built with `MenuItemBuilder`.** `MenuBuilder::item_with_shortcut`,
  `MenuItem::new` and `MenuItem::with_shortcut` no longer exist.
  `MenuBuilder::item` takes two arguments, not three, and `MenuBar::new` takes
  only the rectangle, with menus added by `add_submenu(SubMenu::new(..))`.
  Use `MenuItemBuilder::new().text(..).command(..).key("Ctrl+O").build()`,
  or `MenuBuilder::item_key(text, command, "Ctrl+O")`.
- [ ] **T3. Status lines are built with `StatusItemBuilder`.** `StatusItem::new`,
  `StatusLine::single` and `StatusLineView` no longer exist. Use
  `StatusLine::new(rect, vec![StatusItemBuilder::new().text(..).key(..).command(..).build()])`.
- [ ] **T4. Builders end in `build()`.** `WindowBuilder` and `ButtonBuilder`
  have no `build_boxed()`. Use `build()` and add the view itself:
  `desktop.add(window)` and `dialog.add(button)` take the view, not a `Box`.
- [ ] **T5. Input lines and centring.** `InputLineBuilder::data` no longer
  exists: give the text with `.text(..)` and read it back with
  `input_line.text()`, through the `Handle` that `add_typed` returns.
  `Dialog::set_centered` no longer exists: set `Options::CENTERED` in the
  dialog's options (`set_options`).
- [ ] **T6. Views are reached through handles.** `Desktop::windows`,
  `Desktop::children`, `Desktop::select_view`, `Dialog::insert`,
  `Dialog::find_view_by_id` and `TextViewer::load_text` no longer exist, and
  `Desktop::child_at` returns the view, not an `Option`. Use `add` or
  `add_typed`, `get` and `get_mut` with a `Handle`, `child_by_id`,
  `bring_to_front`, `select_next`, and `TextViewer::set_text` or `load_file`.
  `Desktop::new` takes a `Rect`.
- [ ] **T7. Constants that do not exist.** `CM_HELP`, `CM_DELETE_LINE`,
  `CM_HELP_CONTENTS`, `CM_HELP_KEYBOARD`, `CM_HELP_COMMANDS`, `CM_HELP_ABOUT`,
  and `CM_ABOUT` imported from `core::command`, plus the keys `KB_SHIFT_DEL`,
  `KB_CTRL_INS` and `KB_SHIFT_INS`. Define the application's own commands from
  `CM_USER`, use `CM_HELP_INDEX` for help, and give key chords as strings in
  the builders (`.key("Shift+Del")`).
- [ ] **T8. Message boxes take the application.** `confirmation_box(&mut app.terminal, title, message)`
  is now `confirmation_box(app, message)`, and it returns the command that
  closed it (`CM_YES`, `CM_OK`, ...), not a `bool`. The module
  `views::dialogs` does not exist: the message boxes are in `views::msgbox`.
- [ ] **T9. Step 12, the custom view, uses the old `View` trait.** The modules
  `core::view` and `core::draw_buffer` do not exist: use `views::view` and
  `core::draw::DrawBuffer`. `DrawBuffer::move_char` and `move_str` take an
  `Attr`, not an integer: take it from the palette with `self.map_color(..)`.
  A `View` must implement `core`, `core_mut`, `as_any`, `as_any_mut` and
  `get_palette`. The recipe "Your own view" in `AGENTS.md` has the current
  shape.
- [ ] **T10. Code that cannot compile as written.** A bare `mod commands;`, a
  lone `match` arm, methods with `self` written outside an `impl`, a
  `SortedListBox` that calls a `find_insertion_point` it never defines, and
  `i16` and `i32` mixed in arithmetic in "Adding Controls to the Dialog".
  Make each one complete, or mark it `rust,ignore` when it is only an
  outline.
- [ ] **T11. The walk-through as one program.** Many blocks build on earlier
  listings. Write the complete order-entry application, one example per step
  if it helps, and show the parts in the chapters, with the rest of each
  listing as hidden `#` lines so that every block compiles. Add the
  declarations the blocks leave out (`use std::rc::Rc;`, `serde`, ...).
- [ ] **T12. Keep it compiling.** Include Chapters 1 to 6 in `src/lib.rs` as
  doctests (`#[cfg(doctest)]`, as for `AGENTS.md`), so that
  `cargo test` catches the next change of API. Before that, mark the view
  tree diagram in Chapter 3 (line 265) as ```` ```text ````: an unmarked
  fence is compiled as Rust.
- [ ] **T13. Check the rest of the user guide** (Chapters 7 to 18) the same
  way.

## Block checklist

Line numbers are those of the chapter files when this was written. A ticked
box compiles now.

### Chapter 1: [Stepping into Turbo Vision](user-guide/Chapter-01-Stepping-into-Turbo-Vision.md)

5 blocks: 2 compile, 2 use API that has changed, 1 need the earlier listings or are not code.

- [ ] Line 68, Listing 1.1 — The Simplest Turbo Vision Program: changed API: T1
- [ ] Line 88, Listing 1.2 — An Extensible Application: changed API: T1
- [x] Line 120, Creating a Command Module: compiles
- [x] Line 134, Listing 1.3 — Custom Command Constants: compiles
- [ ] Line 155, Listing 1.3 — Custom Command Constants: not compilable as written: T10

### Chapter 2: [Responding to Commands](user-guide/Chapter-02-Responding-to-Commands.md)

11 blocks: 2 compile, 6 use API that has changed, 3 need the earlier listings or are not code.

- [x] Line 30, Event Types: compiles
- [x] Line 49, Command Identifiers: compiles
- [ ] Line 83, Listing 2.1 — Handling Commands in Main Event Loop: changed API: T1, T7
- [ ] Line 167, Listing 2.2 — Adding a Menu Bar and Status Line: changed API: T1, T2, T3, T7
- [ ] Line 267, Breaking Down the Menu Construction: changed API: T2
- [ ] Line 277, Breaking Down the Menu Construction: needs the earlier listings: T11
- [ ] Line 287, Breaking Down the Menu Construction: changed API: T3
- [ ] Line 322, How Command Sets Work: needs the earlier listings: T11
- [ ] Line 346, Example: Dynamic Command Management: changed API: T1
- [ ] Line 380, Context-Sensitive Status Lines: changed API: T7
- [ ] Line 427, Event Handling Pattern: needs the earlier listings: T11

### Chapter 3: [Adding Windows](user-guide/Chapter-03-Adding-Windows.md)

11 blocks: 2 compile, 7 use API that has changed, 2 need the earlier listings or are not code.

- [ ] Line 44, Listing 3.1 — Adding a Simple Window: changed API: T1, T4
- [x] Line 110, 1. Assigning the Window Boundaries: compiles
- [ ] Line 120, 2. Constructing the Window Object: changed API: T4
- [ ] Line 141, 3. Inserting the Window: needs the earlier listings: T11
- [ ] Line 189, Listing 3.2 — Creating a Modal Dialog: changed API: T4
- [x] Line 309, Window Architecture Details: compiles
- [ ] Line 339, Listing 3.3 — Window with Multiple Views: changed API: T4, T6
- [ ] Line 414, Window Positioning: changed API: T4
- [ ] Line 433, Cascading Windows: changed API: T4
- [ ] Line 455, Checking for Window Closure: needs the earlier listings: T11
- [ ] Line 531, Window Construction: changed API: T4

### Chapter 4: [Persistence and Configuration](user-guide/Chapter-04-Persistence-and-Configuration.md)

15 blocks: 0 compile, 10 use API that has changed, 5 need the earlier listings or are not code.

- [ ] Line 71, Basic Menu Structure: changed API: T2
- [ ] Line 101, Creating Nested Menus (Submenus): changed API: T2, T7
- [ ] Line 122, Manual Menu Construction: changed API: T2, T7
- [ ] Line 138, Attaching Menus to Your Application: changed API: T1, T2, T7
- [ ] Line 183, Simple Status Lines: changed API: T3, T7
- [ ] Line 209, Context-Sensitive Status Lines: changed API: T3, T7
- [ ] Line 241, How Context Switching Works: needs the earlier listings: T11
- [ ] Line 269, Basic Dialog Structure: changed API: T4, T5
- [ ] Line 314, Executing Dialog Boxes: needs the earlier listings: T11
- [ ] Line 327, Reusable Dialog Functions: changed API: T8
- [ ] Line 373, Making State Serializable: needs the earlier listings: T11
- [ ] Line 399, Saving State: changed API: T6
- [ ] Line 426, Restoring State: needs the earlier listings: T11
- [ ] Line 462, Integration Points: changed API: T1
- [ ] Line 496, Alternative: Using RON Format: needs the earlier listings: T11

### Chapter 5: [Creating Data Entry Forms](user-guide/Chapter-05-Creating-Data-Entry-Forms.md)

18 blocks: 1 compile, 8 use API that has changed, 9 need the earlier listings or are not code.

- [x] Line 36, Creating a Function to Build the Dialog: compiles
- [ ] Line 49, Creating a Function to Build the Dialog: changed API: T5
- [ ] Line 71, Creating a Function to Build the Dialog: needs the earlier listings: T11
- [ ] Line 86, Creating a Function to Build the Dialog: needs the earlier listings: T11
- [ ] Line 108, Sending Messages: changed API: T6
- [ ] Line 160, Adding Controls to the Dialog: changed API: T4, T5, T10
- [ ] Line 438, Setting Initial Values: needs the earlier listings: T11
- [ ] Line 462, Reading Control Values: needs the earlier listings: T11
- [ ] Line 486, Responding to Button Commands: needs the earlier listings: T11
- [ ] Line 509, Responding to Button Commands: needs the earlier listings: T11
- [ ] Line 529, Modal Dialogs vs. Non-Modal Dialogs: needs the earlier listings: T11
- [ ] Line 587, Using FilterValidator: changed API: T5
- [ ] Line 613, Using RangeValidator: changed API: T5
- [ ] Line 641, Using PictureValidator: changed API: T5
- [ ] Line 692, Complete Example with Validators: changed API: T5
- [ ] Line 802, Validation When Focus Changes: needs the earlier listings: T11
- [ ] Line 818, Validation on Demand: not compilable as written: T10
- [ ] Line 861, Displaying Validation Errors: changed API: T8

### Chapter 6: [Managing Data Collections](user-guide/Chapter-06-Managing-Data-Collections.md)

21 blocks: 4 compile, 8 use API that has changed, 9 need the earlier listings or are not code.

- [x] Line 43, Understanding Collections in Turbo Vision Rust: compiles
- [x] Line 61, Understanding Collections in Turbo Vision Rust: compiles
- [x] Line 82, Understanding Collections in Turbo Vision Rust: compiles
- [x] Line 103, Creating Data Structures: compiles
- [ ] Line 175, Loading and Initializing the Collection: needs the earlier listings: T11
- [ ] Line 202, Loading and Initializing the Collection: needs the earlier listings: T11
- [ ] Line 261, Loading and Initializing the Collection: needs the earlier listings: T11
- [ ] Line 295, Displaying a Record: changed API: T6
- [ ] Line 319, Displaying a Record: changed API: T6
- [ ] Line 360, Saving the Record: changed API: T6
- [ ] Line 401, Moving from Record to Record: needs the earlier listings: T11
- [ ] Line 478, Adding New Records: needs the earlier listings: T11
- [ ] Line 543, Canceling Edits: needs the earlier listings: T11
- [ ] Line 600, Understanding the View Trait: needs the earlier listings: T11
- [ ] Line 620, Creating the Counting Engine: changed API: T9
- [ ] Line 685, Drawing the View: changed API: T9
- [ ] Line 750, Real-World Example: Broadcast Demo Counter: changed API: T9
- [ ] Line 821, Adding the Counter to Your Dialog: changed API: T6
- [ ] Line 848, Manipulating the Counter: needs the earlier listings: T11
- [ ] Line 916, Additional Features to Consider: changed API: T10
- [ ] Line 956, Additional Features to Consider: needs the earlier listings: T11
