//! One choice from a drop-down list, in one row. F4 or a click on the arrow
//! opens the list; Up and Down change the choice without opening it. It
//! takes less room than RadioButtons when the choices are many.
//!
//! Parameters:
//! - `ComboBox::with_items(bounds, id, items)`, or `new(bounds, id)` then
//!   `set_items` or `add_item`: `bounds` is one row, the field and its
//!   arrow; `id` names the combo box to its drop-down list and must be
//!   unique among the combo boxes alive at the same time.
//! - `set_selected(Some(i))`, `selected()`, `selected_text()`: the choice,
//!   by index or as text; `None` is no choice.
//! - `set_on_change(cmd)`: broadcast `cmd` whenever the choice changes; 0,
//!   the default, sends none.
//! - In a record form, `form.choice` maps the choices to values of any
//!   type.
//!
//! See also: RadioButtons, ListBox, InputLine

use crate::panel::Panel;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::combo_box::ComboBox;
use turbo_vision::views::label::Label;

const COUNTRY_COMBO: u16 = 900;

pub fn build(panel: &mut Panel) {
    let mut country = ComboBox::with_items(
        Rect::new(10, 0, 32, 1),
        COUNTRY_COMBO,
        vec![
            "France".into(),
            "Italy".into(),
            "Spain".into(),
            "Portugal".into(),
        ],
    );
    country.set_selected(Some(2));
    let country = panel.add(country);
    let mut label = Label::new(Rect::new(0, 0, 9, 1), "~C~ountry");
    label.set_link(country);
    panel.add(label);
}
