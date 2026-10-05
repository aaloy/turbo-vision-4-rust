//! Several independent options in one control. Up and Down move between
//! them, Space ticks the focused one, a click ticks the one clicked, Alt
//! and an item's ~letter~ ticks that item. The whole control is one Tab
//! stop. A single option on its own is a `CheckBox`.
//!
//! Parameters:
//! - `CheckBoxes::new(bounds, labels)`: one row per label, so `bounds` is
//!   as tall as the list (32 items at most); mark a label's hot key with
//!   tildes.
//! - `set_checked(i, on)`, `is_checked(i)`: one item, by its index.
//! - `checked_items()`: the indices of the ticked items; `value()` gives
//!   them as bits.
//! - `CheckBox::new(bounds, label)`: a single box, one row;
//!   `set_checked(on)`, `is_checked()` and `toggle()`.
//!
//! See also: RadioButtons, GroupBox, Form

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
