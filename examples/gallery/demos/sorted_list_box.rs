//! A list kept in order, whatever order the items come in. Typing searches:
//! each letter narrows to the first item that starts with what was typed
//! so far. `find_prefix` and `find_exact` search by binary search; add
//! items with `add_item` and they go to their place.

use crate::panel::Panel;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::sorted_listbox::SortedListBox;

pub fn build(panel: &mut Panel) {
    let mut cities = SortedListBox::new(Rect::new(0, 0, 24, 6), 0);
    cities.set_items(
        [
            "Paris", "Berlin", "Madrid", "Rome", "Lisbon", "Vienna", "Prague", "Dublin", "Athens",
            "Oslo", "Bern", "Warsaw",
        ]
        .map(String::from)
        .to_vec(),
    );
    cities.add_item("Brussels".to_string());
    panel.add(cities);
}
