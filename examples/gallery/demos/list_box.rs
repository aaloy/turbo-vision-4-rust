//! A scrolling list of strings. Up, Down, PgUp and PgDn move; typing a
//! letter jumps to an item starting with it. Enter or a double click sends
//! the list's command; read the item with `get_selected_item()`. For many
//! thousands of items, give it a provider instead of a Vec: `set_provider`.

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
