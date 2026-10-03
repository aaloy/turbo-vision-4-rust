# Building Forms with `Form`

`Form` (in `turbo_vision::views::form`) builds a data-entry dialog from a list
of labelled fields. You never write a coordinate: you say **what** goes in the
form, in order, and the form decides **where** it goes, how big the dialog is,
and centres it on the screen.

Without it, every label, field and button needs a hand-computed `Rect`, and the
dialog's size has to be worked out to match. With it, a form is a list.

## Quick start

A complete program (`cargo run --example form_layout` runs a fuller version):

```rust
use turbo_vision::app::Application;
use turbo_vision::core::command::CM_OK;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::GroupLike; // brings `dialog.get(handle)` into scope
use turbo_vision::views::form::{Form, size};
use turbo_vision::views::input_line::InputLine;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let mut form = Form::new("New Customer");
    let name = form.field("~N~ame", InputLine::new(Rect::default(), 40));
    let zip = form.field("~Z~IP code", InputLine::new(size(8, 1), 8));
    form.ok_cancel();
    let mut dialog = form.build();

    if dialog.execute(&mut app) == CM_OK {
        let name = dialog.get(name).map(|f| f.text().to_string()).unwrap_or_default();
        let zip = dialog.get(zip).map(|f| f.text().to_string()).unwrap_or_default();
        println!("{name}, {zip}");
    }
    Ok(())
}
```

Three steps, always the same:

1. **Describe** the form: `Form::new(title)`, then add rows and buttons in
   order. Each call that adds a view returns a typed `Handle`.
2. **Build** it: `form.build()` returns an ordinary `Dialog`.
3. **Run and read**: `dialog.execute(&mut app)` returns the command of the
   button that closed it; `dialog.get(handle)` gives back each view, with its
   own type, to read its value.

## What the layout looks like

The form above, plus a check box, a section and a notes box, comes out like
this (`cargo run --example form_layout`):

```text
╔═[■]═════ New Customer ════════╗
║                               ║
║ Name     ____________________ ║   <- label column | field column
║                               ║   <- spacing: 1 blank row
║ Email    ____________________ ║
║                               ║
║ ZIP code ________             ║   <- built with size(8, 1): keeps 8 columns
║                               ║
║          [ ] VIP customer     ║   <- empty label: lines up with the fields
║                               ║
║                               ║   <- a section adds one more blank row
║ Notes                         ║   <- form.section("Notes")
║                               ║
║ _____________________________ ║   <- form.row(...): spans the whole form,
║ _____________________________ ║      3 rows because built with size(0, 3)
║ _____________________________ ║
║                               ║
║       OK    ▀   Cancel  ▀     ║   <- buttons: centred, on the bottom row
║     ▄▄▄▄▄▄▄▄▄   ▄▄▄▄▄▄▄▄▄     ║
╚═══════════════════════════════╝
```

(`____` marks where an input field is; on screen it is a coloured bar.)

### The rules

- **Rows** go top to bottom in the order you add them, one blank row apart
  (change it with `form.spacing(n)`).
- **Labels** are left-aligned in a column as wide as the longest label. Each
  label is linked to its field: clicking the label, or pressing Alt and the
  letter marked with `~` (`"~N~ame"` → Alt+N), focuses the field.
- **Fields** start right after the label column, all at the same column.
- **Sizes come from the view.** How you build the view decides its size:

  | Build the view with      | Width                                    | Height |
  |--------------------------|------------------------------------------|--------|
  | `Rect::default()`        | stretches to fill its column             | 1 row  |
  | `size(w, h)`             | exactly `w` columns                      | `h` rows |
  | `size(0, h)`             | stretches to fill its column             | `h` rows |

  The field column is at least 20 columns wide (`form.field_width(n)` changes
  that) and widens to fit the widest sized field. The position you give a view
  is ignored: the form sets it.
- **Full-width rows** (`form.row(view)`) start at the label column and span
  the whole form. Use them for a long check box caption, a note, a list.
- **Sections** (`form.section("Address")`) put a heading over the rows that
  follow, with an extra blank row above it.
- **Buttons** go on one row at the bottom, centred, in the order added, each
  at least 10 columns wide. `form.button_align(ButtonAlign::Right)` puts them
  against the right edge.
- **The dialog** is sized to fit everything, including its title, and is
  centred on the desktop when you `execute` it or add it to the desktop. The
  first field has the focus; Tab moves through the fields in row order.

## Recipes

### Read values back

Keep the handle each call returns, then ask the built dialog for the view.
`get` returns `Option<&T>` of the view's own type:

```rust
let name = form.field("~N~ame", InputLine::new(Rect::default(), 40));
let vip = form.field("", CheckBox::new(Rect::default(), "VIP customer"));
let notes = form.row(Memo::new(size(0, 4)));
let mut dialog = form.build();
// ... dialog.execute(&mut app) ...
let name: String = dialog.get(name).map(|f| f.text().to_string()).unwrap_or_default();
let vip: bool = dialog.get(vip).is_some_and(CheckBox::is_checked);
let notes: String = dialog.get(notes).map(Memo::get_text).unwrap_or_default();
```

`get` needs `use turbo_vision::views::GroupLike;`. It returns `None` only if
the handle came from a different dialog.

### Pre-fill values (editing an existing record)

Set the value on the view before adding it, or through `get_mut` afterwards:

```rust
let mut line = InputLine::new(Rect::default(), 40);
line.set_text("Ada Lovelace");
let name = form.field("~N~ame", line);

let mut dialog = form.build();
if let Some(f) = dialog.get_mut(name) {
    f.set_text("Grace Hopper");
}
```

### Validate input

Views keep everything they had before joining the form. Give an input line a
validator as usual:

```rust
use std::cell::RefCell;
use std::rc::Rc;
use turbo_vision::views::validator::RangeValidator;

let age = form.field(
    "~A~ge",
    InputLine::with_validator(size(4, 1), 3, Rc::new(RefCell::new(RangeValidator::new(0, 150)))),
);
```

### Check boxes and other unlabelled views

An empty label leaves the label column blank, so the view lines up under the
fields:

```rust
form.field("", CheckBox::new(Rect::default(), "Send newsletter"));
```

To put it at the left edge instead, use `form.row(...)`.

### Long text, lists, anything taller than one row

Give the view a height; leave the width at 0 to stretch it:

```rust
form.section("Notes");
let notes = form.row(Memo::new(size(0, 5))); // full width, 5 rows
```

### Compact forms, extra space

```rust
form.spacing(0); // no blank rows between rows
form.gap(1);     // one extra blank row here
```

### Custom buttons

`execute` returns the command of the button that closed the dialog. By default
any button you add closes it (`CloseOn::StandardAndButtons`):

```rust
use turbo_vision::core::command::{CM_CANCEL, CommandId};
use turbo_vision::views::form::ButtonAlign;

const CM_SAVE_AND_NEW: CommandId = 1001;

form.default_button("~S~ave", CM_OK);
form.button("Save and ~n~ew", CM_SAVE_AND_NEW);
form.button("Cancel", CM_CANCEL);
form.button_align(ButtonAlign::Right);
// ...
match dialog.execute(&mut app) {
    CM_OK => { /* save */ }
    CM_SAVE_AND_NEW => { /* save, then open a fresh form */ }
    _ => { /* cancelled */ }
}
```

The default button is the one Enter presses when the focused control is not a
button. `form.ok_cancel()` adds **OK** (`CM_OK`, default) and **Cancel**
(`CM_CANCEL`).

### Resizable forms

```rust
form.resizable(true);
```

The user can then resize the dialog from its corner. Stretched fields and
full-width rows follow its width; the buttons stay on the bottom row. Fields
built with a fixed size keep it.

### Non-modal forms

`build()` returns a normal `Dialog`, so it can also live on the desktop like a
window instead of running modally: `app.desktop.add(dialog)`. It is centred
there too.

## API reference

All in `turbo_vision::views::form`.

| Call | What it does | Returns |
|------|--------------|---------|
| `Form::new(title)` | Start a form for a dialog titled `title`. | `Form` |
| `form.field(label, view)` | A labelled row. `label` may mark a hot key with `~`; `""` for no label. | `Handle<T>` |
| `form.row(view)` | A row spanning the whole form, no label. | `Handle<T>` |
| `form.section(title)` | A heading, with an extra blank row above. | `&mut Form` |
| `form.gap(rows)` | Extra blank rows. | `&mut Form` |
| `form.button(title, command)` | A button on the bottom row. | `Handle<Button>` |
| `form.default_button(title, command)` | The button Enter presses. | `Handle<Button>` |
| `form.ok_cancel()` | **OK** (`CM_OK`, default) and **Cancel** (`CM_CANCEL`). | `&mut Form` |
| `form.spacing(rows)` | Blank rows between rows (default 1). | `&mut Form` |
| `form.field_width(cols)` | Narrowest field column (default 20). | `&mut Form` |
| `form.button_align(align)` | `ButtonAlign::Center` (default) or `ButtonAlign::Right`. | `&mut Form` |
| `form.resizable(yes)` | Let the user resize the dialog (default off). | `&mut Form` |
| `form.build()` | Lay out and return the dialog. | `Dialog` |
| `size(w, h)` | A size for a view that should keep it: `Rect::new(0, 0, w, h)`. | `Rect` |

On the built dialog (with `use turbo_vision::views::GroupLike;`):

| Call | What it does |
|------|--------------|
| `dialog.execute(&mut app)` | Run it modally; returns the closing button's command. |
| `dialog.get(handle)` / `dialog.get_mut(handle)` | The view, with its own type. |

The settings (`spacing`, `field_width`, `button_align`, `resizable`) can be
called at any point before `build()`; they apply to the whole form.

## Rules at a glance (for code generators and agents)

Follow these and the result needs no coordinates and no adjustment:

1. Create views with `Rect::default()` (stretch, 1 row) or `size(w, h)` (fixed
   size). Never compute a position: the form overwrites it.
2. Add rows in visual order, top to bottom. Add buttons in left-to-right order.
3. Keep every `Handle` you will read later; read values only after
   `dialog.execute(...)` returns, with `dialog.get(handle)`.
4. Mark each label's hot key with `~` around one letter, and use a different
   letter for each label in the form and each button.
5. Use `form.field("", view)` for a view aligned with the fields,
   `form.row(view)` for one spanning the form.
6. Exactly one `default_button` (or `ok_cancel()`) per form.
7. Compare `execute`'s result with the button commands you added; `CM_CANCEL`
   also comes back when the user presses Esc or closes the window.
8. Bring `turbo_vision::views::GroupLike` into scope to use `get` / `get_mut`.

## Limitations

- One label column and one field column: no two fields side by side on a row.
- Labels go to the left of their field, never above it.
- The layout is computed once, at `build()`. Views added to the dialog
  afterwards (with `dialog.add`) are placed by hand, in the dialog's interior
  coordinates.
- A `CheckBox` does not react to a `~` hot key in its own caption yet. If it
  needs one, give it a form label instead:
  `form.field("~V~IP", CheckBox::new(Rect::default(), "Yes"))` makes Alt+V
  focus it, and Space then ticks it.

## See also

- `examples/form_layout.rs`: the complete form shown above.
- [Chapter 5 – Creating Data-Entry Forms](../guide/chapter-05.md):
  the same controls placed by hand, and how dialogs work underneath.
- [Chapter 13 – Data Validation](../guide/chapter-13.md):
  the validators you can attach to input lines.
