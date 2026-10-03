//! One choice from a drop-down list. F4 or a click on the arrow opens the
//! list; Up and Down change the choice without opening it. Read it with
//! `selected()`. Each live combo box needs its own id. In a record form,
//! `form.choice` maps the choices to values of any type.

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
