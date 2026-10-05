//! One choice among several, in a column. Up and Down move the selection; a
//! click, or Alt and an item's ~letter~, selects. The whole control is one
//! Tab stop.
//!
//! Parameters:
//! - `RadioButtons::new(bounds, labels)`: one row per label (32 items at
//!   most); mark a label's hot key with tildes.
//! - `set_selected(i)`: the choice to start with.
//! - `selected()`: the index chosen; `selected_label()`, its text.
//!
//! See also: RadioButton, ComboBox, CheckBoxes

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
