//! A framed window on the desktop: drag it by its title, resize it from its
//! corner, zoom it with the arrow at the top right, close it with the box
//! at the top left. F6 moves between windows. Keep the handle that
//! `add_typed` returns to reach it, and its views, again.

use crate::panel::Panel;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_USER, CommandId};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::state::Grow;
use turbo_vision::views::button::Button;
use turbo_vision::views::text_viewer::TextViewer;
use turbo_vision::views::window::Window;
use turbo_vision::views::{GroupLike, View};

const OPEN: CommandId = CM_USER + 100;

pub fn build(panel: &mut Panel) {
    panel.add(Button::new(
        Rect::new(0, 0, 20, 2),
        "~O~pen a window",
        OPEN,
        true,
    ));
}

pub fn handle(app: &mut Application, command: CommandId) -> bool {
    if command != OPEN {
        return false;
    }
    let mut window = Window::new(Rect::new(30, 3, 70, 15), "Notes");
    // (0, 0) is the corner inside the frame.
    let mut notes = TextViewer::new(Rect::new(0, 0, 38, 10)).with_scrollbars(true);
    notes.set_text("Drag me by the title.\nResize me from the corner.\nClose me with the box.");
    notes.set_grow_mode(Grow::ALL); // follow the window when it is resized
    window.add(notes);
    app.desktop.add(window);
    true
}
