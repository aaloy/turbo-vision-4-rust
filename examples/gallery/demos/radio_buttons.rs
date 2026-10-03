//! One choice among several. Up and Down move the selection, a click
//! selects. Read it with `selected()`, the index of the chosen item. For a
//! long list of choices, use a `ComboBox` instead.

use crate::panel::Panel;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::cluster_group::RadioButtons;

pub fn build(panel: &mut Panel) {
    let mut size = RadioButtons::new(
        Rect::new(0, 0, 20, 3),
        vec!["Small".into(), "Medium".into(), "Large".into()],
    );
    size.set_selected(1);
    panel.add(size);
}
