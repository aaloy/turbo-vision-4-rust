//! Several lines of text in a form. Type and edit as in an input line;
//! Enter starts a new line, the arrows, PgUp and PgDn move, and Shift
//! selects. Tab moves to the next control, so a memo sits among others.
//! Read it with `get_text()`, fill it with `set_text`; `is_modified()`
//! says whether the user changed it. In a record form: `form.memo`.

use crate::panel::Panel;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::label::Label;
use turbo_vision::views::memo::Memo;

pub fn build(panel: &mut Panel) {
    let mut notes = Memo::new(Rect::new(0, 1, 46, 6)).with_scrollbars(true);
    notes.set_text("Call back on Monday.\nAsk about the second invoice.\nSend the new price list.");
    let notes = panel.add(notes);
    let mut label = Label::new(Rect::new(0, 0, 10, 1), "~N~otes");
    label.set_link(notes);
    panel.add(label);
}
