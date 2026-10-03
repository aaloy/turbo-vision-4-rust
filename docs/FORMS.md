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

`cargo run --example form_layout` builds this form: a few fields, an address
group with two fields on one line, a check box, a notes box and OK/Cancel.

```text
╔═[■]════════ New Customer ═══════════╗
║                                     ║
║  Name ____________________________  ║   <- label column | field column
║                                     ║   <- spacing: 1 blank row
║ Email ____________________________  ║   <- labels right-aligned (LabelAlign::Right)
║                                     ║
║ ┌─ Address ───────────────────────┐ ║   <- form.group("Address")
║ │ Street ________________________ │ ║   <- a group lines up its own labels
║ │                                 │ ║
║ │   City ____________  ZIP ______ │ ║   <- form.line(): two fields on one row;
║ └─────────────────────────────────┘ ║      ZIP built with size(8, 1) keeps 8 columns,
║                                     ║      City stretches into the rest
║       [ ] VIP customer              ║   <- empty label: lines up with the fields
║                                     ║
║                                     ║   <- a section adds one more blank row
║ Notes                               ║   <- form.section("Notes")
║                                     ║
║ ___________________________________ ║   <- form.row(...): spans the whole form,
║ ___________________________________ ║      3 rows because built with size(0, 3)
║ ___________________________________ ║
║                                     ║
║          OK    ▀   Cancel  ▀        ║   <- buttons: centred, on the bottom row
║        ▄▄▄▄▄▄▄▄▄   ▄▄▄▄▄▄▄▄▄        ║
╚═════════════════════════════════════╝
```

(`____` marks where an input field is; on screen it is a coloured bar.)

With `form.label_position(LabelPosition::Above)`
(`cargo run --example form_layout -- --above`) each label moves to the row
above its field, and the form gets narrower and taller:

```text
║ ┌─ Address ────────────┐ ║
║ │ Street               │ ║
║ │ ____________________ │ ║
║ │                      │ ║
║ │ City        ZIP      │ ║
║ │ __________  ________ │ ║
║ └──────────────────────┘ ║
```

### The rules

- **Rows** go top to bottom in the order you add them, one blank row apart
  (change it with `form.spacing(n)`).
- **Labels** go to the left of their fields, in a column as wide as the
  longest label (or above them, see [Where labels go](#where-labels-go)). Each
  label is linked to its field: clicking the label, or pressing Alt and the
  letter marked with `~` (`"~N~ame"` → Alt+N), focuses the field.
- **Fields** start right after the label column, all at the same column.
- **Sizes come from the view.** How you build the view decides its size:

  | Build the view with      | Width                                    | Height |
  |--------------------------|------------------------------------------|--------|
  | `Rect::default()`        | stretches to fill the width it is given  | 1 row  |
  | `size(w, h)`             | exactly `w` columns                      | `h` rows |
  | `size(0, h)`             | stretches to fill the width it is given  | `h` rows |

  A field alone on its row is at least 20 columns wide (`form.field_width(n)`
  changes that), and the field column widens to fit the widest sized field.
  The position you give a view is ignored: the form sets it.
- **Lines** (`form.line()`) put several fields on one row, left to right. The
  first field's label is in the label column; each other field has its label
  just before it. Stretched fields on a line share the width the sized ones
  leave, each at least 10 columns.
- **Groups** (`form.group("Title")` … `form.end_group()`) draw a titled box,
  as wide as the form, around the rows added in between. A group lines up its
  own labels, independently of the rows outside it. Groups can be nested.
- **Full-width rows** (`form.row(view)`) start at the label column and span
  the whole form (or group). Use them for a long check box caption, a note, a
  list.
- **Sections** (`form.section("Address")`) put a plain heading over the rows
  that follow, with an extra blank row above it. Use a group instead when the
  rows should be boxed.
- **Buttons** go on one row at the bottom, centred, in the order added, each
  at least 10 columns wide. `form.button_align(ButtonAlign::Right)` puts them
  against the right edge.
- **The dialog** is sized to fit everything, including its title, and is
  centred on the desktop when you `execute` it or add it to the desktop. The
  first field has the focus; Tab moves through the fields in the order they
  were added: top to bottom, and left to right along a line.

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
let mut input = InputLine::new(Rect::default(), 40);
input.set_text("Ada Lovelace");
let name = form.field("~N~ame", input);

let mut dialog = form.build();
if let Some(f) = dialog.get_mut(name) {
    f.set_text("Grace Hopper");
}
```

### Several fields on one line

```rust
let mut line = form.line();
let city = line.field("~C~ity", InputLine::new(Rect::default(), 40));
let zip = line.field("~Z~IP", InputLine::new(size(8, 1), 8));
```

The line ends when you add the next row. Each `line.field(...)` returns a
handle, like `form.field(...)`. Give the narrow fields a size and leave the
main one stretched: it takes the rest of the row.

### Group related fields in a box

```rust
form.group("Address");
let street = form.field("~S~treet", InputLine::new(Rect::default(), 60));
let mut line = form.line();
let city = line.field("~C~ity", InputLine::new(Rect::default(), 40));
let zip = line.field("~Z~IP", InputLine::new(size(8, 1), 8));
form.end_group();
```

Everything between `group` and `end_group` goes inside the box: fields,
lines, rows, sections, even another group. A group you forget to end is
closed by `build()`. The box itself never takes the focus; it is a
`GroupBox` view, which you can also use on its own in hand-built dialogs.

### Where labels go

Two settings choose where labels go. Both apply to the whole form, groups
included, and can be called at any point before `build()`:

```rust
use turbo_vision::views::form::{LabelAlign, LabelPosition};

form.label_position(LabelPosition::Above); // label on the row above its field
form.label_align(LabelAlign::Right);       // labels on the left, ending at their fields
```

`cargo run --example form_labels` opens one contact form in each style, so you
can compare them; only these calls differ. Here is that form (fields shown as
`____`): a line with two fields, two fields of their own, and an address group
with another line.

**Labels on the left** (the default; Borland's own style). One label column,
labels against its left edge:

```text
╔═[■]═══════════════ Contact ═════════════════╗
║                                             ║
║ First name __________  Last name __________ ║
║                                             ║
║ Email      ________________________________ ║
║                                             ║
║ Phone      _______________                  ║
║                                             ║
║ ┌─ Address ───────────────────────────────┐ ║
║ │ Street ________________________________ │ ║
║ │                                         │ ║
║ │ City   __________________  ZIP ________ │ ║
║ └─────────────────────────────────────────┘ ║
║                                             ║
║              OK    ▀   Cancel  ▀            ║
║            ▄▄▄▄▄▄▄▄▄   ▄▄▄▄▄▄▄▄▄            ║
╚═════════════════════════════════════════════╝
```

**Labels on the left, right-aligned**: `form.label_align(LabelAlign::Right)`.
The same column, but each label ends against its field, so short labels stay
next to what they name:

```text
╔═[■]═══════════════ Contact ═════════════════╗
║                                             ║
║ First name __________  Last name __________ ║
║                                             ║
║      Email ________________________________ ║
║                                             ║
║      Phone _______________                  ║
║                                             ║
║ ┌─ Address ───────────────────────────────┐ ║
║ │ Street ________________________________ │ ║
║ │                                         │ ║
║ │   City __________________  ZIP ________ │ ║
║ └─────────────────────────────────────────┘ ║
║                                             ║
║              OK    ▀   Cancel  ▀            ║
║            ▄▄▄▄▄▄▄▄▄   ▄▄▄▄▄▄▄▄▄            ║
╚═════════════════════════════════════════════╝
```

**Labels above**: `form.label_position(LabelPosition::Above)`. Each label on
the row above its field; there is no label column, so the form is narrower and
taller:

```text
╔═[■]═════ Contact ════════╗
║                          ║
║ First name   Last name   ║
║ ___________  ___________ ║
║                          ║
║ Email                    ║
║ ________________________ ║
║                          ║
║ Phone                    ║
║ _______________          ║
║                          ║
║ ┌─ Address ────────────┐ ║
║ │ Street               │ ║
║ │ ____________________ │ ║
║ │                      │ ║
║ │ City        ZIP      │ ║
║ │ __________  ________ │ ║
║ └──────────────────────┘ ║
║                          ║
║     OK    ▀   Cancel  ▀  ║
║   ▄▄▄▄▄▄▄▄▄   ▄▄▄▄▄▄▄▄▄  ║
╚══════════════════════════╝
```

Things to notice in all three:

- In a line, only the first field's label is in the label column; the others
  (`Last name`, `ZIP`) sit just before their own fields.
- A group lines up its own labels: `Street` and `City` form their own column,
  narrower than `First name`'s.
- A field built with a size (`Phone`, `ZIP`) keeps it in every style; the
  stretched ones adapt.

Which one to pick:

| Style | Good for |
|-------|----------|
| left (default) | most forms; labels of similar length |
| left, right-aligned | labels of very different lengths, so short ones stay next to their fields |
| above | narrow dialogs, long labels, several fields on a line |

`label_align` only matters when labels are on the left.

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

Alt plus a button's `~` letter presses it from anywhere in the form (Alt+O for
**~O~K**). The letter alone works too, but only when the focused control does
not take typed text: in an input line it is typed instead.

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
| `form.line()` | Start a line of fields side by side. | `Line` |
| `line.field(label, view)` | Add a field to the line, right of the previous one. | `Handle<T>` |
| `form.row(view)` | A row spanning the whole form (or group), no label. | `Handle<T>` |
| `form.group(title)` | Start a titled box around the rows that follow. | `&mut Form` |
| `form.end_group()` | End the innermost open group. | `&mut Form` |
| `form.section(title)` | A heading, with an extra blank row above. | `&mut Form` |
| `form.gap(rows)` | Extra blank rows. | `&mut Form` |
| `form.button(title, command)` | A button on the bottom row. | `Handle<Button>` |
| `form.default_button(title, command)` | The button Enter presses. | `Handle<Button>` |
| `form.ok_cancel()` | **OK** (`CM_OK`, default) and **Cancel** (`CM_CANCEL`). | `&mut Form` |
| `form.spacing(rows)` | Blank rows between rows (default 1). | `&mut Form` |
| `form.field_width(cols)` | Narrowest field column (default 20). | `&mut Form` |
| `form.label_position(pos)` | `LabelPosition::Left` (default) or `LabelPosition::Above`. | `&mut Form` |
| `form.label_align(align)` | `LabelAlign::Left` (default) or `LabelAlign::Right`. | `&mut Form` |
| `form.button_align(align)` | `ButtonAlign::Center` (default) or `ButtonAlign::Right`. | `&mut Form` |
| `form.resizable(yes)` | Let the user resize the dialog (default off). | `&mut Form` |
| `form.build()` | Lay out and return the dialog. | `Dialog` |
| `size(w, h)` | A size for a view that should keep it: `Rect::new(0, 0, w, h)`. | `Rect` |

On the built dialog (with `use turbo_vision::views::GroupLike;`):

| Call | What it does |
|------|--------------|
| `dialog.execute(&mut app)` | Run it modally; returns the closing button's command. |
| `dialog.get(handle)` / `dialog.get_mut(handle)` | The view, with its own type. |

The settings (`spacing`, `field_width`, `label_position`, `label_align`,
`button_align`, `resizable`) can be called at any point before `build()`;
they apply to the whole form.

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
6. For fields on one row, call `form.line()` once and add each field with
   `line.field(...)`; add the next row with `form.*` as usual. Give every
   field but the main one a `size(w, 1)`.
7. Wrap related rows in `form.group("Title")` … `form.end_group()`; pair
   every `group` with an `end_group`.
8. Exactly one `default_button` (or `ok_cancel()`) per form.
9. Compare `execute`'s result with the button commands you added; `CM_CANCEL`
   also comes back when the user presses Esc or closes the window.
10. Bring `turbo_vision::views::GroupLike` into scope to use `get` / `get_mut`.

## Limitations

- Label position and alignment apply to the whole form, not per group.
- Groups are as wide as the form: two groups cannot sit side by side.
- A line's fields after the first do not line up with the fields of other
  lines; only the first field of each row is in the field column.
- The layout is computed once, at `build()`. Views added to the dialog
  afterwards (with `dialog.add`) are placed by hand, in the dialog's interior
  coordinates.
- A `CheckBox` does not react to a `~` hot key in its own caption yet. If it
  needs one, give it a form label instead:
  `form.field("~V~IP", CheckBox::new(Rect::default(), "Yes"))` makes Alt+V
  focus it, and Space then ticks it.

## See also

- `examples/form_layout.rs`: the complete form shown at the top.
- `examples/form_labels.rs`: one form in each label style.
- [Chapter 5 – Creating Data-Entry Forms](user-guide/Chapter-05-Creating-Data-Entry-Forms.md):
  the same controls placed by hand, and how dialogs work underneath.
- [Chapter 13 – Data Validation](user-guide/Chapter-13-Data-Validation.md):
  the validators you can attach to input lines.
