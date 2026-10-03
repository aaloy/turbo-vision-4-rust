# Design System: Action Plan and Decision Log

The plan for a design system for Turbo Vision for Rust: what every component
looks like, how it behaves, how to use it, and how the components fit
together in a real application. This file tracks the work and the decisions
behind it; update it as each step lands.

## Goal

Anyone, a developer or an AI assistant, can see every component, in every
state, with the exact code that produced it, and can follow written rules for
colours, spacing, keyboard and wording. Everything shown is generated from
code, so it cannot drift from the library.

## Deliverables

| # | Deliverable | Where | Status |
|---|-------------|-------|--------|
| 1 | **Component gallery**: an interactive program listing every component, each live, with how it works and the code that built it. | `examples/gallery/` | 10 components (step 1 done); the rest in step 3 |
| 2 | **Generated reference pages**: one page and PNG per component, rendered from the gallery's demos. | `website/docs/components/` | planned (step 2) |
| 3 | **Foundations**: colours by role, spacing, states, keyboard conventions, wording. | `docs/DESIGN-SYSTEM.md` | planned (step 4) |
| 4 | **Sample application**: a small contacts-and-invoices program that uses the components the way a real application would. | `examples/crm/` | planned (step 5) |

## Principles

1. **One source.** A component's demo is one file. The gallery runs it, shows
   its text as the code, and the page generator renders it; nothing is copied.
2. **Nothing hand-made.** Screenshots and component pages are generated.
   The hand-captured images on the site are replaced by generated ones.
3. **Complete or failing.** A test fails when a view module has no gallery
   demo, as `tests/docs_index.rs` already does for `AGENTS.md` and the API
   catalog.
4. **Idiomatic code only.** Demos use the API the way `AGENTS.md` teaches:
   commands from `CM_USER`, handles, `Form` for dialogs, no hand-written loops.

## Steps

### Step 1: gallery engine and the first ten components

- [x] Plan and decision log (this file).
- [x] Gallery engine: a component list on the left; on the right a panel
      with the live demo, "How it works" and the code. Moving through the
      list switches the demo; the demo's buttons reach its own handler.
- [x] Demo file convention: `examples/gallery/demos/<name>.rs` with a `//!`
      header (shown as "How it works"), `build(panel: &mut Panel)` and an
      optional `handle(app, command)`. A new demo also needs a `pub mod` line
      in `demos/mod.rs` and an entry in `registry.rs`.
- [x] First ten demos: Button, InputLine (with validators), CheckBoxes and
      CheckBox, RadioButtons, ComboBox, ListBox, Table, Form, message boxes,
      Window.
- [x] Tests (`cargo test` runs them): every demo builds inside its panel,
      has a header and something to try, keeps its hot keys unique and its
      commands in range; the panel builds for every demo.

**Done when** `cargo run --example gallery` shows all ten, and the tests pass.

### Step 2: generated pages and pictures

- [ ] A renderer that builds each demo on the headless terminal and writes a
      PNG (`core::screenshot::render_to_png`) and a Markdown page per component.
- [ ] Site navigation: a "Components" section built from the registry.
- [ ] Replace the hand-made captures in `website/docs/assets/captures/` that
      the pages use (one shows `pascal_ide`, which no longer exists).
- [ ] Decide when the renderer runs: by hand with the site sync, or in CI.

### Step 3: every component

- [ ] Demos for the remaining views: Memo, Label, StaticText, ParamText,
      Spinner, Slider, RadioButton, Outline, SortedListBox, TextViewer,
      ProgressBar, Tooltip, TabbedPane, SplitPane, GroupBox, history, file and
      directory dialogs, colour dialog, editor windows, help, menus, status
      line.
- [ ] Coverage test: every view module has a demo or is listed as internal
      (frame, scroller, base traits) with a reason.

### Step 4: foundations document

- [ ] `docs/DESIGN-SYSTEM.md`: palettes and colour roles, spacing (the
      values `Form` uses), frames and shadows, states (focused, default,
      disabled, pressed, invalid, modal), keyboard conventions, wording.
- [ ] Expose `Form`'s spacing as public constants so the document and custom
      layouts use the same numbers (decision O2).
- [ ] Pictures from the gallery for each rule.

### Step 5: sample application

- [ ] Contacts and invoices: customers in a filtered `Table` with a record
      editor; invoices in a `TabbedPane` with a `SplitPane`; a `ComboBox` for
      the customer and `Spinner`s for quantities; import from a file with a
      progress bar; settings with groups; F1 help; tooltips; commands switched
      off when they do not apply; confirm before delete.
- [ ] Data behind a small repository trait, in memory, so a database backend
      is one more implementation.
- [ ] Scripted UI tests that drive it.

## Decision log

Each decision has an id, the date, what was decided and why. Change a
decision by adding a new entry that supersedes it; do not edit old ones.

| Id | Date | Decision | Why | Status |
|----|------|----------|-----|--------|
| D1 | 2026-10-03 | Generate pages and pictures from code; never capture them by hand. | Hand-made material drifts: the old API catalog listed methods that no longer existed, and a site capture shows a program that is gone. | accepted |
| D2 | 2026-10-03 | The gallery is an example program (`examples/gallery/`), not part of the library. | The library should not carry demo code, and an example can use every public API the way an application does. | accepted |
| D3 | 2026-10-03 | One demo is one file: its `//!` header is the "How it works" text, the rest is the code, and the gallery shows the file as written. | The code a reader sees is the code that runs; no copy can go stale. | accepted |
| D4 | 2026-10-03 | Each demo shows its states side by side (a normal, a default and a disabled button), not through toggles. | A static PNG then shows every state too, and the demos stay short. Toggles can come later if a component has too many states to show at once. | accepted |
| D5 | 2026-10-03 | The gallery and the sample application are separate programs. | The gallery must show everything in every state; the application must look like real software. `showcase` tries both and does neither fully. | accepted |
| D6 | 2026-10-03 | The demo panel is a dialog (gray palette); the component list is a window (blue). | Controls are designed for dialogs, so they show their intended colours. | accepted |
| D7 | 2026-10-03 | The demo follows the list as the focus moves (checked when the application is idle), without needing Enter. | Browsing with the arrow keys is the fastest way to look through components. | accepted |
| D8 | 2026-10-03 | Demo commands start at `CM_USER + 100`; only the shown demo's handler is asked about them. | Below `CM_USER + 100` is left to the gallery itself, and demos cannot clash because one runs at a time. | accepted |
| D9 | 2026-10-03 | Demos build into a `Panel` (`examples/gallery/panel.rs`) that places their views straight in the gallery's dialog, offset to the "Try it" box; not into a nested `Group`. | A plain `Group` never takes the focus, so controls inside one could not be used from the keyboard (O5). `Panel::add` reads like `Group::add`. | accepted |
| D10 | 2026-10-03 | The list and the panel have no shadow. | Side by side they fill the desktop; a window keeps room for its shadow and would be pushed over its neighbour. | accepted |

### Open questions

| Id | Question | Decide by |
|----|----------|-----------|
| O1 | Colour themes beyond the Borland palettes (`app.set_palette`): wanted? If so, the gallery should switch between them. | step 2 |
| O2 | Make `Form`'s spacing values public constants. | step 4 |
| O3 | The sample application's domain: contacts and invoices, as proposed? | step 5 |
| O4 | Run the page renderer in CI, or by hand with `website/sync_docs.py`? | step 2 |
| O5 | A `Group` inside a dialog never takes the focus (`can_focus` is false), so its controls cannot be reached by keyboard; `TabbedPane` manages its pages itself. Make a group focusable when it holds focusable views (Borland nests groups freely)? It would also let the gallery go back to plain groups. | step 3 |

## Progress log

| Date | Step | What happened |
|------|------|---------------|
| 2026-10-03 | 1 | Plan written; gallery engine and the first ten demos built on branch `feat/gallery`. |
| 2026-10-03 | 1 | Building the gallery found a library bug: `CheckBoxes` and `RadioButtons` drew at their own position instead of their corner (a leftover from before owner-relative coordinates), so away from (0, 0) they appeared doubly offset and clicks hit the wrong row. Fixed, with tests. Also found O5. |
| 2026-10-03 | 1 | Step 1 done: `cargo run --example gallery` shows the ten components; 7 gallery tests run with `cargo test`; checked by hand in an 80x25 terminal (list browsing, F6, every demo's controls and commands, mouse). |
