//! Several independent options in one control. Up and Down move between
//! them, Space ticks the focused one, a click ticks the one clicked.
//! Read them with `is_checked(i)` or `checked_items()`. A single option on
//! its own is a `CheckBox`, read with `is_checked()`.

use crate::panel::Panel;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::checkbox::CheckBox;
use turbo_vision::views::cluster_group::CheckBoxes;

pub fn build(panel: &mut Panel) {
    let mut toppings = CheckBoxes::new(
        Rect::new(0, 0, 22, 3),
        vec!["Cheese".into(), "Olives".into(), "Basil".into()],
    );
    toppings.set_checked(0, true);
    panel.add(toppings);

    let mut extra = CheckBox::new(Rect::new(26, 0, 46, 1), "~E~xtra crispy");
    extra.set_checked(true);
    panel.add(extra);
}
