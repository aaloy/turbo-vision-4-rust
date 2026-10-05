//! Several lines of text among a form's other controls. Type and edit as in
//! an input line; Enter starts a new line, the arrows, PgUp and PgDn move,
//! and Shift selects. Tab moves on to the next control, unlike in the
//! Editor.
//!
//! Parameters:
//! - `Memo::new(bounds)`: as many rows as `bounds` is tall;
//!   `.with_scrollbars(true)` puts scroll bars inside those bounds.
//! - `set_text(text)`, `get_text()`: the text, with `\n` between lines.
//! - `is_modified()`: whether the user changed it; `clear_modified()`
//!   starts again.
//! - `set_max_length(Some(n))`: the longest a line may be; `None` for no
//!   limit.
//! - `set_read_only(true)`: shown, not changed. `set_tab_size(n)`: how wide
//!   a tab is drawn.
//! - In a record form: `form.memo`.
//!
//! See also: InputLine, Editor, TextViewer

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
