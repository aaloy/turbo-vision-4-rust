//! A whole number on a track: drag the thumb or click where it should go.
//! Left and Right move it one step, Home and End to the ends. Read it with
//! `value()`; `set_on_change(cmd)` broadcasts each change to the views of
//! the same window. It is the dragging counterpart of the Spinner.

use crate::panel::Panel;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::label::Label;
use turbo_vision::views::slider::Slider;

pub fn build(panel: &mut Panel) {
    let mut volume = Slider::new(Rect::new(12, 0, 44, 1), 0, 100);
    volume.set_value(60);
    volume.set_step(5);
    labelled(panel, 0, "~V~olume", volume);

    let mut balance = Slider::new(Rect::new(12, 2, 44, 3), -10, 10);
    balance.set_value(0);
    labelled(panel, 2, "~B~alance", balance);
}

/// Add `field` with a label on row `y` that focuses it.
fn labelled(panel: &mut Panel, y: i16, text: &str, field: Slider) {
    let field = panel.add(field);
    let mut label = Label::new(Rect::new(0, y, 11, y + 1), text);
    label.set_link(field);
    panel.add(label);
}
