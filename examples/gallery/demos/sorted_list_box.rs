//! A list kept in order, whatever order the items come in. Typing searches:
//! each letter narrows to the first item that starts with what was typed so
//! far.
//!
//! Parameters:
//! - `SortedListBox::new(bounds, command)`: as a ListBox; `command` is sent
//!   on Enter or a double click (0 for none).
//! - `set_items(items)`, `add_item(text)`: items go to their place in the
//!   order.
//! - `set_case_sensitive(true)`: sort and search with case; off by default.
//! - `find_prefix(text)`, `find_exact(text)`: search by binary search;
//!   `focus_prefix(text)` also moves the focus there.
//! - `get_selection()`, `get_selected_item()`: the focused item.
//!
//! See also: ListBox, ComboBox, Table

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
