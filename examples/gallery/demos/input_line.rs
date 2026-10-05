//! One line of text. Type to insert; Home, End and the arrows move; Shift
//! selects; Ctrl+C, Ctrl+X and Ctrl+V copy, cut and paste. A validator
//! limits what can be typed: here only digits, and a date mask that fills
//! in the slashes.
//!
//! Parameters:
//! - `InputLine::new(bounds, max_length)`: one row; `max_length` is the
//!   most characters it takes, which may be more than the field shows.
//! - `with_validator(bounds, max_length, validator)`, or `set_validator`:
//!   `FilterValidator::new("0123456789")` allows only those characters;
//!   `RangeValidator::new(min, max)`, a number in range;
//!   `PictureValidator::new("##/##/####")`, a mask (# a digit, ? a letter,
//!   & a letter in capitals, @ any character);
//!   `LookupValidator::new(values)`, one of a list.
//! - `set_text(text)`, `text()`; `is_valid()` asks the validator about the
//!   text.
//! - `Label::set_link(id)`: a label whose ~letter~ focuses the input.
//! - In a record form, `form.input` reads and writes the field for you.
//!
//! See also: History, Memo, ComboBox

use crate::panel::Panel;
use std::cell::RefCell;
use std::rc::Rc;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::input_line::InputLine;
use turbo_vision::views::label::Label;
use turbo_vision::views::picture_validator::PictureValidator;
use turbo_vision::views::validator::FilterValidator;

pub fn build(panel: &mut Panel) {
    let mut name = InputLine::new(Rect::new(10, 0, 40, 1), 40);
    name.set_text("Ada Lovelace");
    labelled(panel, 0, "~N~ame", name);

    let digits = Rc::new(RefCell::new(FilterValidator::new("0123456789")));
    labelled(
        panel,
        2,
        "~P~hone",
        InputLine::with_validator(Rect::new(10, 2, 24, 3), 12, digits),
    );

    let date = Rc::new(RefCell::new(PictureValidator::new("##/##/####")));
    labelled(
        panel,
        4,
        "~B~orn",
        InputLine::with_validator(Rect::new(10, 4, 22, 5), 10, date),
    );
}

/// Add `field` with a label on row `y` that focuses it.
fn labelled(panel: &mut Panel, y: i16, text: &str, field: InputLine) {
    let field = panel.add(field);
    let mut label = Label::new(Rect::new(0, y, 9, y + 1), text);
    label.set_link(field);
    panel.add(label);
}
