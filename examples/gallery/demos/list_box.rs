//! A scrolling list of strings. Up, Down, PgUp and PgDn move; typing a
//! letter jumps to an item starting with it. Enter or a double click sends
//! the list's command.
//!
//! Parameters:
//! - `ListBox::new(bounds, command)`: one item per row; `command` is sent
//!   on Enter or a double click (0 for none).
//! - `set_items(items)`, `add_item(text)`, `clear()`: the items, from a
//!   Vec.
//! - `set_provider(provider)`: read the items from a `ListProvider` as they
//!   are drawn, for many thousands of them; `refresh_items()` after its
//!   length changes.
//! - `get_selection()`, `get_selected_item()`: the focused item, by index
//!   or as text; `set_selection(i)` moves the focus.
//! - `set_multi_select(true)`: Space marks items and Shift+click marks a
//!   run, apart from the focus; `marked_items()` returns them in order.
//!
//! See also: SortedListBox, Table, Outline

use crate::panel::Panel;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_USER, CommandId};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::listbox::ListBox;
use turbo_vision::views::msgbox::message_box_ok;

const PICKED: CommandId = CM_USER + 100;

pub fn build(panel: &mut Panel) {
    let mut planets = ListBox::new(Rect::new(0, 0, 24, 5), PICKED);
    planets.set_items(
        [
            "Mercury", "Venus", "Earth", "Mars", "Jupiter", "Saturn", "Uranus", "Neptune",
        ]
        .map(String::from)
        .to_vec(),
    );
    panel.add(planets);
}

pub fn handle(app: &mut Application, command: CommandId) -> bool {
    if command != PICKED {
        return false;
    }
    message_box_ok(app, "The list sent its command: read the item there.");
    true
}
