# To do: the user guide walk-through does not compile

Chapters 1 to 6 of the [user guide](user-guide/) are a walk-through: one
application built in twelve steps, from an empty desktop to a data-entry
system with validation and a custom view. Much of its code does not compile
against the current API.

This file was written on 2026-11-13 and checked again on 2026-11-27 against
the guide of release 4.2.0. That release fixed the guide's complete programs
(#131, #134) and added `scripts/check-guide-listings.py`, run by the
`Guide listings` workflow, which compiles them. The fragments between those
programs are still unchecked, and most of the work below is in them.

## What was found

The six chapters have 82 Rust code blocks:

| Result | 2026-11-13 | 2026-11-27 | After T4 | Meaning |
|--------|-----------:|-----------:|---------:|---------|
| Compile | 11 | 25 | 26 | Correct as written. |
| Use API that has changed | 41 | 23 | 19 | They call methods, constructors or constants that no longer exist, or pass the wrong arguments. These are the blocks to fix. |
| Need the earlier listings, or are not code | 29 | 34 | 37 | They use variables, types or functions that an earlier listing defines (`app`, `Order`, `create_order_dialog`), crates they never declare (`serde_json`, `dirs`, `ron`), or are not complete code (a bare `mod commands;`, a lone `match` arm). They could not be checked on their own, so they may hide more changed API. |

Every listing of Chapters 1 and 2 that stands on its own now compiles,
Listing 1.1 included. The changed API left is in Chapters 3 to 6, and most
of it in Chapter 5 (8 of 18 blocks after T4) and Chapter 6 (6 of 21). Chapter 4 has
16 blocks now, one more than on 2026-11-13.

## How it was checked

Each block was compiled on its own, compile only, with every module of the
crate glob-imported, so that a missing `use` line could not hide a real
error:

1. Make a throwaway crate that depends on this checkout by path (and on
   `serde` and `serde_json`, which the walk-through uses).
2. Turn each ```` ```rust ```` block of the guide into one binary of it. Put
   `use turbo_vision::{prelude, app, views, core, terminal}::*;` and a glob
   import of every module under `turbo_vision::views` and
   `turbo_vision::core` at the top, and wrap a block that has no `fn main`
   in one, as rustdoc does.
3. Run `cargo check --bins --keep-going --message-format=json` and group the
   errors by binary.

Some errors are caused by the glob imports, not by the guide, and were
discounted: `StatusLine` and `KeyCode` are ambiguous between two modules
(pin them with an explicit `use`), and the crate's `Result<T>` alias hides
std's two-argument `Result<T, E>`.

## Tasks

Each task names the current replacement. The block checklist further down
says which tasks each block needs.

- [x] **T1. `main` returns the crate's `Result`.** Done in #131: every
  `main` returns `turbo_vision::core::error::Result<()>`.
- [x] **T2. Menus are built with `MenuItemBuilder`.** Done in #134: no
  listing uses `item_with_shortcut`, `MenuItem::new` or
  `MenuItem::with_shortcut` any more.
- [x] **T3. Status lines are built with `StatusItemBuilder`.** Done in #134.
- [x] **T4. Builders end in `build()`, and views are added by value.**
  Done: every `build_boxed()` is now `build()`, and every
  `add(Box::new(view))` is `add(view)`. Only `ButtonBuilder` and
  `WindowBuilder` had lost `build_boxed()`; the other builders still have
  it, and `add(Box::new(..))` still compiles, because `Box<V>` is a `View`
  too (`src/views/view.rs`). But the guide now adds views by value, as
  `AGENTS.md` does. Chapter 6's sketch of `Group` shows the generic `add`.
  The `Box::new` around validators in Chapter 5 is left for T5, and
  `Dialog::insert(Box::new(..))` for T6.
- [ ] **T5. Input lines and centring.** `InputLineBuilder::data` no longer
  exists: give the text with `.text(..)` and read it back with
  `input_line.text()`, through the `Handle` that `add_typed` returns.
  `Dialog::set_centered` no longer exists: set `Options::CENTERED` in the
  dialog's options (`set_options`). Left in Chapter 4 and in every dialog
  and validator listing of Chapter 5.
- [ ] **T6. Views are reached through handles.** Still used:
  `TextViewer::load_text` (Listing 3.3), `Desktop::windows` (Chapter 4,
  "Saving State"), `Desktop::children`, `child_at` and `select_view`
  (Chapter 5, "Sending Messages"), `Desktop::new()` with no `Rect`,
  `Dialog::find_view_by_id` and `Dialog::insert` (Chapter 6). Use `add` or
  `add_typed`, `get` and `get_mut` with a `Handle`, `child_by_id`,
  `bring_to_front`, `select_next`, and `TextViewer::set_text` or
  `load_file`.
- [ ] **T7. Command numbers.** The constants that did not exist are gone.
  But Chapter 4 now defines its own `CM_HELP_KEYBOARD`, `CM_HELP_COMMANDS`,
  `CM_HELP_CONTENTS`, `CM_HELP_ABOUT` (141–144) and `CM_DELETE_LINE` (150),
  inside the range 100–199 that belongs to the crate. Number them from
  `CM_USER`, as `AGENTS.md` says.
- [ ] **T8. Message boxes take the application.** `confirmation_box(&mut app.terminal, title, message)`
  in Chapter 4 ("Reusable Dialog Functions") is now
  `confirmation_box(app, message)`, and it returns the command that closed
  it (`CM_YES`, `CM_OK`, ...), not a `bool`. Chapter 5 ("Displaying
  Validation Errors") still imports from `views::dialogs`, which does not
  exist: the message boxes are in `views::msgbox`.
- [ ] **T9. Step 12, the custom view, uses the old `View` trait.** The modules
  `core::view` and `core::draw_buffer` do not exist: use `views::view` and
  `core::draw::DrawBuffer`. `DrawBuffer::move_char` and `move_str` take an
  `Attr`, not an integer: take it from the palette with `self.map_color(..)`.
  A `View` must implement `core`, `core_mut`, `as_any`, `as_any_mut` and
  `get_palette`. The recipe "Your own view" in `AGENTS.md` has the current
  shape.
- [ ] **T10. Code that cannot compile as written.** A bare `mod commands;`
  (Chapter 1), a lone `match` arm (Chapter 5, "Validation on Demand"),
  methods with `self` written outside an `impl` (Chapter 6, "Displaying a
  Record" and "Saving the Record"), a `SortedListBox` that calls a
  `find_insertion_point` it never defines, and `i16` and `i32` mixed in
  arithmetic in "Adding Controls to the Dialog". Make each one complete, or
  mark it `rust,ignore` when it is only an outline.
- [ ] **T11. The walk-through as one program.** Many blocks build on earlier
  listings. Write the complete order-entry application, one example per step
  if it helps, and show the parts in the chapters, with the rest of each
  listing as hidden `#` lines so that every block compiles. Add the
  declarations the blocks leave out (`use std::rc::Rc;`, `std::path::Path`,
  `serde`, ...).
- [ ] **T12. Keep it compiling.** Partly done: since #134,
  `scripts/check-guide-listings.py` compiles every complete program (a block
  with `fn main` and a `use turbo_vision` line) on each change to the guide
  or the source. Fragments are still not compiled. Once T11 gives each
  fragment its hidden lines, extend that script to every block (or include
  the chapters as doctests, as for `AGENTS.md`). Before that, mark the view
  tree diagram in Chapter 3 (line 265) as ```` ```text ````: it is still an
  unmarked fence.
- [x] **T13. Check the rest of the user guide** (Chapters 7 to 19). Done on
  2026-11-27, with the method above. The results are in the next section.

## The rest of the guide (Chapters 7 to 19)

| Chapter | Blocks | Compile | Changed API | Need context |
|---------|-------:|--------:|------------:|-------------:|
| 7. Architecture Overview | 22 | 6 | 3 | 13 |
| 8. Views and Groups | 37 | 4 | 1 | 32 |
| 9. Event-Driven Programming | 8 | 3 | 1 | 4 |
| 10. Application Objects | 16 | 3 | 1 | 12 |
| 11. Window and Dialog Box Objects | 14 | 1 | 7 | 6 |
| 12. Control Objects | 18 | 1 | 5 | 12 |
| 13. Data Validation | 22 | 11 | 6 | 5 |
| 14. Palettes and Color Selection | 28 | 12 | 1 | 15 |
| 15. Editor and Text Views | 26 | 6 | 1 | 19 |
| 16. Collections and Streams | 19 | 7 | 4 | 8 |
| 17. Streams and Persistence | 14 | 1 | 2 | 11 |
| 18. Resources | 9 | 5 | 1 | 3 |
| 19. Localization | 12 | 9 | 0 | 3 |

Most "need context" blocks in Chapters 7 to 18 are methods shown without
their `impl`, or use a `self`, `app` or `dialog` from the text around them.
Chapter 19 is new in 4.2.0 and has no changed API.

The changed API, by chapter:

- [ ] **Chapter 7:** a `View` impl with the old trait (missing `core`,
  `core_mut`, `as_any`, `as_any_mut`, `get_palette`); `build_boxed`; a
  `Button` built with `bounds` and `state` fields, which are private.
- [ ] **Chapter 8:** `Terminal::set_cursor` ("Handling the Cursor").
- [ ] **Chapter 9:** `Event::new_broadcast` ("Broadcasts for
  notifications").
- [ ] **Chapter 10:** `build_boxed` ("Executing a Modal Dialog").
- [ ] **Chapter 11:** `CommandId::CM_OK` and `CommandId::CM_CANCEL`
  (`CommandId` is a `u16`; the constants are in `core::command`);
  `build_boxed`; `MsgBox::new` ("Using Message Boxes").
- [ ] **Chapter 12:** `CheckBoxBuilder::text`, `RadioButtonBuilder::text`,
  `ListBoxBuilder::command`, `InputLineBuilder::data`.
- [ ] **Chapter 13:** `InputLineBuilder::data` in five listings;
  `PictureValidator::format`.
- [ ] **Chapter 14:** a custom view with the old `View` trait ("Example:
  Custom View with Colors").
- [ ] **Chapter 15:** a `SyntaxHighlighter` without `language`, and
  `TokenType::Text`, which does not exist.
- [ ] **Chapter 16:** `HistoryList::new(20)` (it takes no capacity now);
  `build_boxed`; `child_at` on a dialog; `Desktop`'s private `children` and
  a `remove_all` that does not exist; `Rect` used with serde, which it does
  not implement; `bincode` never declared.
- [ ] **Chapter 17:** `Rect` and `Point` used with serde ("Using Serde for
  Configuration", "Desktop State Snapshots"). The snapshot also iterates
  over the `Desktop` and calls `clear`, `insert`, `select_nth` and a
  `current` field, none of which exist. `bincode` is never declared.
- [ ] **Chapter 18:** `set_centered`, `InputLineBuilder::data` and
  `build_boxed` in "Defining Dialog Boxes". It also uses a `resources`
  module and the `notify` crate, which it does not declare.

## Block checklist

Line numbers are those of the chapter files on `main` after T4. A ticked
box compiles now.

### Chapter 1: [Stepping into Turbo Vision](user-guide/Chapter-01-Stepping-into-Turbo-Vision.md)

5 blocks: 4 compile, 0 use API that has changed, 1 need the earlier listings or are not code.

- [x] Line 68, Listing 1.1 — The Simplest Turbo Vision Program: compiles
- [x] Line 91, Listing 1.2 — An Extensible Application: compiles
- [x] Line 124, Creating a Command Module: compiles
- [x] Line 138, Listing 1.3 — Custom Command Constants: compiles
- [ ] Line 159, Listing 1.3 — Custom Command Constants: needs the earlier listings: T10, T11

### Chapter 2: [Responding to Commands](user-guide/Chapter-02-Responding-to-Commands.md)

11 blocks: 6 compile, 0 use API that has changed, 5 need the earlier listings or are not code.

- [x] Line 30, Event Types: compiles
- [x] Line 49, Command Identifiers: compiles
- [x] Line 83, Listing 2.1 — Handling Commands in Main Event Loop: compiles
- [x] Line 170, Listing 2.2 — Adding a Menu Bar and Status Line: compiles
- [x] Line 270, Breaking Down the Menu Construction: compiles
- [ ] Line 280, Breaking Down the Menu Construction: needs the earlier listings: T11
- [ ] Line 289, Breaking Down the Menu Construction: needs the earlier listings: T11
- [ ] Line 332, How Command Sets Work: needs the earlier listings: T11
- [x] Line 356, Example: Dynamic Command Management: compiles
- [ ] Line 395, Context-Sensitive Status Lines: needs the earlier listings: T11
- [ ] Line 448, Event Handling Pattern: needs the earlier listings: T11

### Chapter 3: [Adding Windows](user-guide/Chapter-03-Adding-Windows.md)

11 blocks: 5 compile, 1 use API that has changed, 5 need the earlier listings or are not code.

- [x] Line 44, Listing 3.1 — Adding a Simple Window: compiles
- [x] Line 110, 1. Assigning the Window Boundaries: compiles
- [ ] Line 120, 2. Constructing the Window Object: needs the earlier listings: T11
- [ ] Line 141, 3. Inserting the Window: needs the earlier listings: T11
- [x] Line 189, Listing 3.2 — Creating a Modal Dialog: compiles
- [x] Line 309, Window Architecture Details: compiles
- [ ] Line 339, Listing 3.3 — Window with Multiple Views: changed API: T6
- [ ] Line 414, Window Positioning: needs the earlier listings: T11
- [x] Line 433, Cascading Windows: compiles
- [ ] Line 455, Checking for Window Closure: needs the earlier listings: T11
- [ ] Line 531, Window Construction: needs the earlier listings: T11

### Chapter 4: [Persistence and Configuration](user-guide/Chapter-04-Persistence-and-Configuration.md)

16 blocks: 5 compile, 4 use API that has changed, 7 need the earlier listings or are not code.

- [x] Line 71, Basic Menu Structure: compiles
- [x] Line 97, Creating Nested Menus (Submenus): compiles
- [x] Line 124, Manual Menu Construction: compiles
- [x] Line 144, Attaching Menus to Your Application: compiles
- [ ] Line 187, Simple Status Lines: needs the earlier listings: T11
- [ ] Line 215, Context-Sensitive Status Lines: changed API: T7, T11
- [ ] Line 251, Context-Sensitive Status Lines: needs the earlier listings: T11
- [ ] Line 261, How Context Switching Works: needs the earlier listings: T11
- [ ] Line 284, Basic Dialog Structure: changed API: T5
- [ ] Line 329, Executing Dialog Boxes: needs the earlier listings: T11
- [ ] Line 342, Reusable Dialog Functions: changed API: T8, T11
- [x] Line 388, Making State Serializable: compiles
- [ ] Line 414, Saving State: changed API: T6, T11
- [ ] Line 441, Restoring State: needs the earlier listings: T11
- [ ] Line 477, Integration Points: needs the earlier listings: T11
- [ ] Line 511, Alternative: Using RON Format: needs the earlier listings: T11

### Chapter 5: [Creating Data-Entry Forms](user-guide/Chapter-05-Creating-Data-Entry-Forms.md)

18 blocks: 1 compile, 8 use API that has changed, 9 need the earlier listings or are not code.

- [x] Line 36, Creating a Function to Build the Dialog: compiles
- [ ] Line 49, Creating a Function to Build the Dialog: changed API: T5
- [ ] Line 71, Creating a Function to Build the Dialog: needs the earlier listings: T11
- [ ] Line 86, Creating a Function to Build the Dialog: needs the earlier listings: T11
- [ ] Line 108, Sending Messages: changed API: T6, T11
- [ ] Line 160, Adding Controls to the Dialog: changed API: T5, T10, T11
- [ ] Line 438, Setting Initial Values: needs the earlier listings: T11
- [ ] Line 462, Reading Control Values: needs the earlier listings: T11
- [ ] Line 486, Responding to Button Commands: needs the earlier listings: T11
- [ ] Line 509, Responding to Button Commands: needs the earlier listings: T11
- [ ] Line 529, Modal Dialogs vs. Non-Modal Dialogs: needs the earlier listings: T11
- [ ] Line 587, Using FilterValidator: changed API: T5, T11
- [ ] Line 613, Using RangeValidator: changed API: T5, T11
- [ ] Line 641, Using PictureValidator: changed API: T5, T11
- [ ] Line 692, Complete Example with Validators: changed API: T5, T11
- [ ] Line 802, Validation When Focus Changes: needs the earlier listings: T11
- [ ] Line 818, Validation on Demand: needs the earlier listings: T10, T11
- [ ] Line 861, Displaying Validation Errors: changed API: T8

### Chapter 6: [Managing Data Collections](user-guide/Chapter-06-Managing-Data-Collections.md)

21 blocks: 5 compile, 6 use API that has changed, 10 need the earlier listings or are not code.

- [x] Line 43, Understanding Collections in Turbo Vision Rust: compiles
- [x] Line 61, Understanding Collections in Turbo Vision Rust: compiles
- [x] Line 82, Understanding Collections in Turbo Vision Rust: compiles
- [x] Line 104, Creating Data Structures: compiles
- [ ] Line 176, Loading and Initializing the Collection: needs the earlier listings: T11
- [ ] Line 203, Loading and Initializing the Collection: needs the earlier listings: T11
- [ ] Line 262, Loading and Initializing the Collection: needs the earlier listings: T11
- [ ] Line 296, Displaying a Record: changed API: T6, T11
- [ ] Line 320, Displaying a Record: changed API: T6, T10, T11
- [ ] Line 361, Saving the Record: changed API: T6, T10, T11
- [ ] Line 402, Moving from Record to Record: needs the earlier listings: T11
- [ ] Line 479, Adding New Records: needs the earlier listings: T11
- [ ] Line 544, Canceling Edits: needs the earlier listings: T11
- [x] Line 601, Understanding the View Trait: compiles
- [ ] Line 621, Creating the Counting Engine: changed API: T9
- [ ] Line 686, Drawing the View: needs the earlier listings: T11
- [ ] Line 751, Real-World Example: Broadcast Demo Counter: changed API: T9, T11
- [ ] Line 822, Adding the Counter to Your Dialog: changed API: T6, T11
- [ ] Line 849, Manipulating the Counter: needs the earlier listings: T11
- [ ] Line 917, Additional Features to Consider: not compilable as written: T10
- [ ] Line 957, Additional Features to Consider: needs the earlier listings: T11
