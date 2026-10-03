// (C) 2025 - Enzo Lombardi

//! Form layout - build a dialog from a list of fields, with no coordinates.
//!
//! A [`Form`] lays out a [`Dialog`] for you: labels in one column, fields in
//! the next, buttons centred along the bottom, and the dialog sized to fit
//! and centred on the screen. You say *what* goes in the form, in order; the
//! form decides *where* it goes.
//!
//! ```
//! use turbo_vision::core::geometry::Rect;
//! use turbo_vision::views::GroupLike;
//! use turbo_vision::views::checkbox::CheckBox;
//! use turbo_vision::views::form::{Form, size};
//! use turbo_vision::views::input_line::InputLine;
//!
//! let mut form = Form::new("Customer");
//! // No size: the field stretches to the width of the field column.
//! let name = form.field("~N~ame", InputLine::new(Rect::default(), 40));
//! // A size: the field keeps it (8 columns, 1 row).
//! let zip = form.field("~Z~IP code", InputLine::new(size(8, 1), 8));
//! // A row with no label, aligned with the fields.
//! let vip = form.field("", CheckBox::new(Rect::default(), "VIP customer"));
//! form.ok_cancel();
//! let mut dialog = form.build();
//!
//! // Run it with `dialog.execute(&mut app)`; then read the values back
//! // through the handles the form returned:
//! let name_text = dialog.get(name).map(|f| f.text().to_string());
//! # assert_eq!(name_text.as_deref(), Some(""));
//! # let _ = (zip, vip);
//! ```
//!
//! # Layout rules
//!
//! - **Rows** go top to bottom in the order they are added, with
//!   [`spacing`](Form::spacing) blank rows between them (1 by default).
//! - **Labels** ([`field`](Form::field)) are left-aligned in a column as wide
//!   as the longest label. A label is linked to its field: clicking it, or
//!   pressing Alt and its `~`-marked letter, focuses the field. An empty label
//!   leaves the label column blank, so the view still lines up with the fields.
//! - **Sizes** come from the view: build it with `size(width, height)` to fix
//!   its size, or with `Rect::default()` to let it stretch to the width of its
//!   column, one row high. Stretched views also follow the dialog's width when
//!   it is resized.
//! - **Full-width rows** ([`row`](Form::row)) start at the label column and
//!   span the whole form; [`section`](Form::section) adds a heading with a
//!   blank row above it.
//! - **Buttons** sit on one row at the bottom, centred (or right-aligned with
//!   [`button_align`](Form::button_align)), each at least 10 columns wide.
//! - **The dialog** is sized to fit its contents and its title, and centred
//!   on the desktop when it is executed or added to the desktop.
//!
//! Every view keeps its own behaviour: validators, history lists, colours.
//! The form only sets each view's position and size, through
//! [`View::set_bounds`], once [`build`](Form::build) is called.

use super::button::Button;
use super::dialog::Dialog;
use super::group::GroupLike;
use super::handle::Handle;
use super::label::Label;
use super::static_text::StaticText;
use super::view::{View, ViewId};
use crate::core::command::{CM_CANCEL, CM_OK, CommandId};
use crate::core::geometry::Rect;
use crate::core::state::{Grow, Options};

/// Columns between the dialog's frame and the form's contents, each side.
const MARGIN_X: i16 = 1;
/// Blank rows between the top of the frame and the first row.
const MARGIN_TOP: i16 = 1;
/// Columns between a label and its field.
const LABEL_GAP: i16 = 1;
/// Columns between two buttons.
const BUTTON_GAP: i16 = 2;
/// Narrowest button, as in Borland's standard dialogs.
const MIN_BUTTON_WIDTH: i16 = 10;
/// Rows a button takes: its face and its shadow.
const BUTTON_HEIGHT: i16 = 2;
/// Columns the frame needs besides the title: corners, close box, padding.
const TITLE_CHROME: i16 = 12;

/// A size, for a view that should keep it in a [`Form`]:
/// `InputLine::new(size(8, 1), 8)` is a field 8 columns wide and 1 row high.
///
/// The form only reads the width and height; the position is its own.
pub fn size(width: i16, height: i16) -> Rect {
    Rect::new(0, 0, width, height)
}

/// Where a [`Form`] puts its buttons along the bottom row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonAlign {
    /// Centred under the form (the default).
    #[default]
    Center,
    /// Against the right edge, as many modern dialogs do. Right-aligned
    /// buttons also stay against that edge when the dialog is resized.
    Right,
}

/// A view's size as the form sees it: `None` for a width that stretches.
#[derive(Debug, Clone, Copy)]
struct Extent {
    width: Option<i16>,
    height: i16,
}

impl Extent {
    fn of(view: &dyn View) -> Self {
        let b = view.bounds();
        Self {
            width: (b.width() > 0).then_some(b.width()),
            height: b.height().max(1),
        }
    }
}

enum Row {
    /// A label (if any) in the label column and a view in the field column.
    Field {
        label: Option<(ViewId, i16)>,
        view: ViewId,
        extent: Extent,
    },
    /// A view spanning the whole form.
    Full { view: ViewId, extent: Extent },
    /// A heading over the rows that follow.
    Section { view: ViewId, width: i16 },
    /// Extra blank rows.
    Gap(i16),
}

struct FormButton {
    view: ViewId,
    width: i16,
}

/// Lays out a [`Dialog`] from a list of labelled fields, with no
/// coordinates. See the [module documentation](self) for the rules and an
/// example.
///
/// Add rows in order with [`field`](Self::field), [`row`](Self::row),
/// [`section`](Self::section) and [`gap`](Self::gap), buttons with
/// [`button`](Self::button), [`default_button`](Self::default_button) or
/// [`ok_cancel`](Self::ok_cancel), then call [`build`](Self::build) for the
/// finished dialog. The methods that add a view return its typed
/// [`Handle`]: keep it to read the view back from the dialog with
/// [`GroupLike::get`] after the dialog has run.
pub struct Form {
    dialog: Dialog,
    title_width: i16,
    rows: Vec<Row>,
    buttons: Vec<FormButton>,
    spacing: i16,
    min_field_width: i16,
    button_align: ButtonAlign,
    resizable: bool,
}

impl Form {
    /// Start a form for a dialog titled `title`.
    pub fn new(title: &str) -> Self {
        Self {
            // A placeholder size; `build` gives the dialog its real one.
            dialog: Dialog::new(size(4, 4), title),
            title_width: display_width(title),
            rows: Vec::new(),
            buttons: Vec::new(),
            spacing: 1,
            min_field_width: 20,
            button_align: ButtonAlign::Center,
            resizable: false,
        }
    }

    /// Blank rows between two rows of the form (default 1; 0 packs them).
    pub fn spacing(&mut self, rows: i16) -> &mut Self {
        self.spacing = rows.max(0);
        self
    }

    /// The narrowest the field column may be (default 20). Stretched fields
    /// are at least this wide; a wider sized field widens the column.
    pub fn field_width(&mut self, width: i16) -> &mut Self {
        self.min_field_width = width.max(1);
        self
    }

    /// Where the buttons go along the bottom row (default centred).
    pub fn button_align(&mut self, align: ButtonAlign) -> &mut Self {
        self.button_align = align;
        self
    }

    /// Let the user resize the dialog (default off). Stretched fields and
    /// full-width rows follow its width; the buttons stay on the bottom row.
    pub fn resizable(&mut self, resizable: bool) -> &mut Self {
        self.resizable = resizable;
        self
    }

    /// Add a labelled field: `label` in the label column, `view` beside it.
    ///
    /// Mark the label's hot key with `~`, as in `"~N~ame"`: Alt+N then
    /// focuses the field. An empty label leaves the label column blank.
    /// `view` keeps the size it was built with, or stretches to the field
    /// column if it was built with no width (`Rect::default()`).
    pub fn field<T: View + 'static>(&mut self, label: &str, view: T) -> Handle<T> {
        let extent = Extent::of(&view);
        let handle = self.dialog.add_typed(view);
        let label = (!label.is_empty()).then(|| {
            let mut l = Label::new(Rect::default(), label);
            l.set_link(handle.id());
            (self.dialog.add(l), display_width(label))
        });
        self.rows.push(Row::Field {
            label,
            view: handle.id(),
            extent,
        });
        handle
    }

    /// Add a view spanning the whole form, with no label: a check box with
    /// a long caption, a note, a list. It keeps its size, or stretches to
    /// the form's width if it was built with no width.
    pub fn row<T: View + 'static>(&mut self, view: T) -> Handle<T> {
        let extent = Extent::of(&view);
        let handle = self.dialog.add_typed(view);
        self.rows.push(Row::Full {
            view: handle.id(),
            extent,
        });
        handle
    }

    /// Add a heading over the rows that follow, with a blank row above it
    /// (unless it is the first row).
    pub fn section(&mut self, title: &str) -> &mut Self {
        let view = self.dialog.add(StaticText::new(Rect::default(), title));
        self.rows.push(Row::Section {
            view,
            width: display_width(title),
        });
        self
    }

    /// Add `rows` blank rows (on top of the usual spacing).
    pub fn gap(&mut self, rows: i16) -> &mut Self {
        self.rows.push(Row::Gap(rows.max(0)));
        self
    }

    /// Add a button on the bottom row that sends `command` when pressed.
    /// Buttons appear left to right in the order they are added.
    pub fn button(&mut self, title: &str, command: CommandId) -> Handle<Button> {
        self.add_button(title, command, false)
    }

    /// Add the default button: the one Enter presses when the focused
    /// control is not a button.
    pub fn default_button(&mut self, title: &str, command: CommandId) -> Handle<Button> {
        self.add_button(title, command, true)
    }

    /// Add the usual pair: **OK** (`CM_OK`, the default) and **Cancel**
    /// (`CM_CANCEL`). Either one closes the dialog, and `execute` returns
    /// its command.
    pub fn ok_cancel(&mut self) -> &mut Self {
        self.default_button("~O~K", CM_OK);
        self.button("Cancel", CM_CANCEL);
        self
    }

    fn add_button(&mut self, title: &str, command: CommandId, is_default: bool) -> Handle<Button> {
        let handle =
            self.dialog
                .add_typed(Button::new(Rect::default(), title, command, is_default));
        self.buttons.push(FormButton {
            view: handle.id(),
            width: (display_width(title) + 4).max(MIN_BUTTON_WIDTH),
        });
        handle
    }

    /// Lay the form out and return the finished dialog: sized to fit,
    /// centred when it is executed or added to the desktop, with the first
    /// field focused.
    ///
    /// The handles returned while building stay valid: use them with
    /// [`GroupLike::get`] / [`GroupLike::get_mut`] on the returned dialog.
    pub fn build(mut self) -> Dialog {
        let layout = self.measure();

        // Size the dialog first. Its children carry no grow bits yet, so the
        // resize leaves them alone; they are placed just below.
        self.dialog
            .set_bounds(size(layout.width + 2, layout.height + 2));
        for (id, bounds, grow) in &layout.places {
            if let Some(view) = self.dialog.child_by_id_mut(*id) {
                view.set_bounds(*bounds);
                if self.resizable {
                    view.set_grow_mode(*grow);
                }
            }
        }

        self.dialog
            .set_options(self.dialog.options() | Options::CENTERED);
        self.dialog.set_resizable(self.resizable);
        self.dialog.set_initial_focus();
        self.dialog
    }

    /// Work out every view's place, and the size of the dialog's interior.
    fn measure(&self) -> Layout {
        let columns = self.columns();
        let mut places = Vec::new();
        let rows_end = self.place_rows(&columns, &mut places);
        // A blank row under the rows, and the buttons below it if any.
        let height = if self.buttons.is_empty() {
            rows_end + 1
        } else {
            let top = if self.rows.is_empty() {
                MARGIN_TOP
            } else {
                rows_end + 1
            };
            self.place_buttons(&columns, top, &mut places);
            top + BUTTON_HEIGHT
        };
        Layout {
            width: columns.content + 2 * MARGIN_X,
            height,
            places,
        }
    }

    /// The column widths: the narrowest content that fits every row, the
    /// buttons and the title.
    fn columns(&self) -> Columns {
        let label_width = self
            .rows
            .iter()
            .filter_map(|row| match row {
                Row::Field {
                    label: Some((_, w)),
                    ..
                } => Some(*w),
                _ => None,
            })
            .max();
        // The field column starts after the labels, if there are any.
        let field_x = MARGIN_X + label_width.map_or(0, |w| w + LABEL_GAP);
        let label_column = field_x - MARGIN_X;

        let mut content = 0;
        let mut widest_field = None;
        for row in &self.rows {
            match row {
                Row::Field { extent, .. } => {
                    let w = extent.width.unwrap_or(0).max(self.min_field_width);
                    widest_field = widest_field.max(Some(w));
                }
                Row::Full { extent, .. } => content = content.max(extent.width.unwrap_or(0)),
                Row::Section { width, .. } => content = content.max(*width),
                Row::Gap(_) => {}
            }
        }
        if let Some(w) = widest_field {
            content = content.max(label_column + w);
        }

        let gaps = i16::try_from(self.buttons.len().saturating_sub(1)).unwrap_or(0);
        let buttons = self.buttons.iter().map(|b| b.width).sum::<i16>() + BUTTON_GAP * gaps;
        let content = content
            .max(buttons)
            .max(self.title_width + TITLE_CHROME - 2 - 2 * MARGIN_X)
            .max(1);
        Columns {
            field_x,
            field_width: content - label_column,
            content,
            buttons,
        }
    }

    /// Place the rows top to bottom; returns the first row below them.
    fn place_rows(&self, c: &Columns, places: &mut Vec<(ViewId, Rect, Grow)>) -> i16 {
        let mut bottom = MARGIN_TOP; // the first free row after the last row
        for (i, row) in self.rows.iter().enumerate() {
            let mut y = if i == 0 {
                MARGIN_TOP
            } else {
                bottom + self.spacing
            };
            match row {
                Row::Field {
                    label,
                    view,
                    extent,
                } => {
                    if let Some((label, w)) = label {
                        let r = Rect::new(MARGIN_X, y, MARGIN_X + w, y + 1);
                        places.push((*label, r, Grow::empty()));
                    }
                    let (w, grow) = stretch(extent.width, c.field_width);
                    let r = Rect::new(c.field_x, y, c.field_x + w, y + extent.height);
                    places.push((*view, r, grow));
                    bottom = y + extent.height;
                }
                Row::Full { view, extent } => {
                    let (w, grow) = stretch(extent.width, c.content);
                    let r = Rect::new(MARGIN_X, y, MARGIN_X + w, y + extent.height);
                    places.push((*view, r, grow));
                    bottom = y + extent.height;
                }
                Row::Section { view, width } => {
                    if i > 0 {
                        y += 1; // the blank row above a heading
                    }
                    let r = Rect::new(MARGIN_X, y, MARGIN_X + width, y + 1);
                    places.push((*view, r, Grow::empty()));
                    bottom = y + 1;
                }
                Row::Gap(rows) => {
                    // Blank rows on top of the spacing on either side of it:
                    // the next row adds its own spacing after them.
                    bottom += rows;
                }
            }
        }
        bottom
    }

    /// Place the buttons on one row starting at `top`.
    fn place_buttons(&self, c: &Columns, top: i16, places: &mut Vec<(ViewId, Rect, Grow)>) {
        let (offset, grow) = match self.button_align {
            ButtonAlign::Center => ((c.content - c.buttons) / 2, Grow::LO_Y | Grow::HI_Y),
            ButtonAlign::Right => (c.content - c.buttons, Grow::ALL),
        };
        let mut x = MARGIN_X + offset;
        for b in &self.buttons {
            let r = Rect::new(x, top, x + b.width, top + BUTTON_HEIGHT);
            places.push((b.view, r, grow));
            x += b.width + BUTTON_GAP;
        }
    }
}

/// The form's column widths, in the dialog's interior.
struct Columns {
    /// Where the field column starts.
    field_x: i16,
    /// How wide the field column is: what a stretched field gets.
    field_width: i16,
    /// How wide the whole form is, labels included.
    content: i16,
    /// How wide the row of buttons is.
    buttons: i16,
}

/// The finished layout: the interior's size and each view's place in it,
/// with the grow bits it takes when the dialog is resizable.
struct Layout {
    width: i16,
    height: i16,
    places: Vec<(ViewId, Rect, Grow)>,
}

/// A view's width in a column `column` wide: its own, or the column's (and
/// then it follows the dialog's width).
fn stretch(width: Option<i16>, column: i16) -> (i16, Grow) {
    match width {
        Some(w) => (w, Grow::empty()),
        None => (column, Grow::HI_X),
    }
}

/// Columns `text` takes on screen: its characters, without the `~` that
/// mark a hot key.
fn display_width(text: &str) -> i16 {
    i16::try_from(text.chars().filter(|&c| c != '~').count()).unwrap_or(i16::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::{Event, EventType, KB_ALT_E};
    use crate::views::checkbox::CheckBox;
    use crate::views::input_line::InputLine;
    use crate::views::memo::Memo;

    fn input() -> InputLine {
        InputLine::new(Rect::default(), 40)
    }

    fn bounds<T: View + 'static>(d: &Dialog, h: Handle<T>) -> Rect {
        d.get(h).expect("the handle finds its view").bounds()
    }

    /// Every label in the dialog, with the view it is linked to.
    fn labels(d: &Dialog) -> Vec<(String, Rect, Option<ViewId>)> {
        (0..d.child_count())
            .filter_map(|i| {
                let v = d.child_at(i);
                let l = v.as_any().downcast_ref::<Label>()?;
                Some((l.text().to_string(), l.bounds(), v.label_link()))
            })
            .collect()
    }

    #[test]
    fn labels_share_a_column_and_fields_line_up_beside_them() {
        let mut form = Form::new("T");
        let name = form.field("~N~ame", input());
        let city = form.field("~P~ostal town", input());
        let d = form.build();

        let (n, c) = (bounds(&d, name), bounds(&d, city));
        assert_eq!(n.a.x, c.a.x, "fields share a column");
        // "Postal town" is the longest label: 11 columns, then one gap.
        assert_eq!(n.a.x, MARGIN_X + 11 + LABEL_GAP);
        assert_eq!(
            (n.a.y, c.a.y),
            (MARGIN_TOP, MARGIN_TOP + 2),
            "one blank row between"
        );

        let labels = labels(&d);
        assert_eq!(labels.len(), 2);
        for ((text, rect, link), field) in labels.iter().zip([name.id(), city.id()]) {
            assert_eq!(rect.a.x, MARGIN_X, "{text} starts the label column");
            assert_eq!(*link, Some(field), "{text} is linked to its field");
        }
        assert_eq!(labels[1].1.a.y, c.a.y, "a label sits on its field's row");
    }

    #[test]
    fn a_sized_view_keeps_its_size_and_an_unsized_one_stretches() {
        let mut form = Form::new("T");
        let zip = form.field("~Z~IP", InputLine::new(size(8, 1), 8));
        let name = form.field("~N~ame", input());
        let notes = form.field("N~o~tes", Memo::new(size(0, 3)));
        let d = form.build();

        assert_eq!(bounds(&d, zip).width(), 8);
        assert_eq!(bounds(&d, name).width(), 20, "the default field column");
        let notes = bounds(&d, notes);
        assert_eq!(
            (notes.width(), notes.height()),
            (20, 3),
            "height kept, width stretched"
        );
    }

    #[test]
    fn a_wide_sized_field_widens_the_column_for_the_stretched_ones() {
        let mut form = Form::new("T");
        form.field("A", InputLine::new(size(30, 1), 30));
        let b = form.field("B", input());
        let d = form.build();
        assert_eq!(bounds(&d, b).width(), 30);
    }

    #[test]
    fn the_dialog_is_sized_to_fit_and_centred() {
        let mut form = Form::new("T");
        let name = form.field("~N~ame", input()); // label 4 + gap 1 + field 20
        form.ok_cancel();
        let d = form.build();

        let content = 4 + LABEL_GAP + 20;
        let b = d.bounds();
        assert_eq!(b.width(), content + 2 * MARGIN_X + 2);
        // Row 1 the field, row 2 blank, rows 3-4 the buttons, plus the frame.
        assert_eq!(b.height(), MARGIN_TOP + 1 + 1 + BUTTON_HEIGHT + 2);
        assert!(d.options().contains(Options::CENTERED));
        assert!(
            d.get(name).is_some_and(View::is_focused),
            "first field focused"
        );
    }

    #[test]
    fn a_long_title_widens_the_dialog() {
        let title = "A rather long dialog title";
        let mut form = Form::new(title);
        form.field("X", input());
        let d = form.build();
        assert!(d.bounds().width() >= display_width(title) + TITLE_CHROME);
    }

    #[test]
    fn buttons_are_centred_on_the_bottom_row_and_ok_is_the_default() {
        let mut form = Form::new("T");
        form.field("~N~ame", input());
        form.ok_cancel();
        let d = form.build();

        let buttons: Vec<&Button> = (0..d.child_count())
            .filter_map(|i| d.child_at(i).as_any().downcast_ref::<Button>())
            .collect();
        let [ok, cancel] = buttons[..] else {
            panic!("two buttons")
        };
        assert_eq!((ok.command(), ok.is_default()), (CM_OK, true));
        assert_eq!((cancel.command(), cancel.is_default()), (CM_CANCEL, false));

        let (o, c) = (ok.bounds(), cancel.bounds());
        assert_eq!(o.a.y, c.a.y, "one row");
        assert_eq!(c.a.x, o.b.x + BUTTON_GAP);
        let interior = d.bounds().width() - 2;
        let (left, right) = (o.a.x, interior - c.b.x);
        assert!(
            (left - right).abs() <= 1,
            "centred: {left} left, {right} right"
        );
        assert_eq!(
            c.b.y,
            d.bounds().height() - 2,
            "the last row of the interior"
        );
    }

    #[test]
    fn right_aligned_buttons_end_at_the_margin() {
        let mut form = Form::new("T");
        form.field("~N~ame", input());
        form.button_align(ButtonAlign::Right);
        let close = form.default_button("~C~lose", CM_OK);
        let d = form.build();
        assert_eq!(bounds(&d, close).b.x, d.bounds().width() - 2 - MARGIN_X);
    }

    #[test]
    fn an_empty_label_leaves_the_column_blank_but_keeps_the_alignment() {
        let mut form = Form::new("T");
        let name = form.field("~N~ame", input());
        let vip = form.field("", CheckBox::new(Rect::default(), "~V~IP"));
        let d = form.build();
        assert_eq!(labels(&d).len(), 1, "no label for the check box");
        assert_eq!(bounds(&d, vip).a.x, bounds(&d, name).a.x);
    }

    #[test]
    fn rows_and_sections_span_the_form() {
        let mut form = Form::new("T");
        let name = form.field("~N~ame", input());
        form.section("Notes");
        let notes = form.row(Memo::new(size(0, 3)));
        let d = form.build();

        let (n, m) = (bounds(&d, name), bounds(&d, notes));
        assert_eq!(m.a.x, MARGIN_X);
        assert_eq!(m.b.x, n.b.x, "as wide as the form");
        // Name on row 1; blank, extra blank, heading on row 4; blank; notes.
        assert_eq!(m.a.y, n.a.y + 5);
    }

    #[test]
    fn spacing_and_gaps_set_the_blank_rows() {
        let mut form = Form::new("T");
        form.spacing(0);
        let a = form.field("A", input());
        let b = form.field("B", input());
        form.gap(2);
        let c = form.field("C", input());
        let d = form.build();
        assert_eq!(bounds(&d, b).a.y, bounds(&d, a).a.y + 1, "packed");
        assert_eq!(bounds(&d, c).a.y, bounds(&d, b).a.y + 3, "two blank rows");
    }

    #[test]
    fn a_resizable_form_stretches_its_fields_and_keeps_the_buttons_at_the_bottom() {
        let mut form = Form::new("T");
        form.resizable(true);
        let zip = form.field("~Z~IP", InputLine::new(size(8, 1), 8));
        let name = form.field("~N~ame", input());
        let ok = form.default_button("~O~K", CM_OK);
        let mut d = form.build();

        let (zip0, name0, ok0) = (bounds(&d, zip), bounds(&d, name), bounds(&d, ok));
        let b = d.bounds();
        d.set_bounds(Rect::new(b.a.x, b.a.y, b.b.x + 10, b.b.y + 4));

        assert_eq!(bounds(&d, zip), zip0, "a sized field stays put");
        assert_eq!(bounds(&d, name).width(), name0.width() + 10);
        assert_eq!(
            bounds(&d, ok).a.y,
            ok0.a.y + 4,
            "buttons stay on the bottom row"
        );
    }

    #[test]
    fn a_label_hot_key_focuses_its_field() {
        let mut form = Form::new("T");
        let name = form.field("~N~ame", input());
        let email = form.field("~E~mail", input());
        let mut d = form.build();
        assert!(d.get(name).is_some_and(View::is_focused));

        let mut alt_e = Event::keyboard(KB_ALT_E);
        d.handle_event(&mut alt_e);
        assert!(d.get(email).is_some_and(View::is_focused));
        assert_eq!(alt_e.what, EventType::Nothing);
    }

    #[test]
    fn executing_the_dialog_centres_it_on_the_desktop() {
        let mut app =
            crate::app::Application::with_terminal(crate::test_util::test_terminal(80, 25));
        let mut form = Form::new("T");
        form.field("~N~ame", input());
        let mut d = form.build();
        d.prepare_modal(&mut app);

        let (desk, b) = (app.desktop.get_bounds(), d.bounds());
        assert!(
            (b.a.x - desk.a.x - (desk.b.x - b.b.x)).abs() <= 1,
            "centred across"
        );
        assert!(
            (b.a.y - desk.a.y - (desk.b.y - b.b.y)).abs() <= 1,
            "centred down"
        );
    }
}
