//! A titled frame around related controls. It only draws: the controls
//! inside are added after it, as its siblings, so they keep the dialog's
//! focus order and Tab moves through them as usual. `Form::group` puts one
//! around the rows that follow it, with no coordinates.

use crate::panel::Panel;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::checkbox::CheckBox;
use turbo_vision::views::group_box::GroupBox;
use turbo_vision::views::input_line::InputLine;
use turbo_vision::views::label::Label;

pub fn build(panel: &mut Panel) {
    // The box first, so the controls draw on top of it.
    panel.add(GroupBox::new(Rect::new(0, 0, 46, 5), "Delivery"));

    let mut street = InputLine::new(Rect::new(11, 1, 44, 2), 40);
    street.set_text("12 St James's Square");
    let street = panel.add(street);
    let mut label = Label::new(Rect::new(2, 1, 10, 2), "~S~treet");
    label.set_link(street);
    panel.add(label);

    let city = panel.add(InputLine::new(Rect::new(11, 2, 44, 3), 40));
    let mut label = Label::new(Rect::new(2, 2, 10, 3), "~C~ity");
    label.set_link(city);
    panel.add(label);

    panel.add(CheckBox::new(
        Rect::new(2, 3, 30, 4),
        "~L~eave with a neighbour",
    ));
}
