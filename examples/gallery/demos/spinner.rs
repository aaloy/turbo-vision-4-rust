//! A whole number in a range, with up and down steppers. Up and Down step,
//! PgUp and PgDn step ten times, Home and End jump to the ends; digits are
//! typed in and kept within the range. A click on an arrow steps too.
//! Read it with `value()`; `set_on_change(cmd)` broadcasts each change to
//! the views of the same window.

use crate::panel::Panel;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::label::Label;
use turbo_vision::views::spinner::Spinner;

pub fn build(panel: &mut Panel) {
    let mut copies = Spinner::new(Rect::new(12, 0, 22, 1), 1, 99);
    copies.set_value(2);
    labelled(panel, 0, "~C~opies", copies);

    let mut zoom = Spinner::new(Rect::new(12, 2, 22, 3), 25, 400);
    zoom.set_value(100);
    zoom.set_step(25);
    zoom.set_suffix("%");
    labelled(panel, 2, "~Z~oom", zoom);

    // With wrap on, stepping past one end comes back at the other.
    let mut hour = Spinner::new(Rect::new(12, 4, 22, 5), 0, 23);
    hour.set_value(9);
    hour.set_wrap(true);
    labelled(panel, 4, "~H~our", hour);
}

/// Add `field` with a label on row `y` that focuses it.
fn labelled(panel: &mut Panel, y: i16, text: &str, field: Spinner) {
    let field = panel.add(field);
    let mut label = Label::new(Rect::new(0, y, 11, y + 1), text);
    label.set_link(field);
    panel.add(label);
}
