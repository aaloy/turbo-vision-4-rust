//! The row of key hints at the bottom of the screen. Each item shows a key
//! and sends a command when that key is pressed or the item is clicked; an
//! item with no command only explains a key. Give it to the application
//! with `set_status_line`. A hint, `set_hint`, adds a note on the right.
//! Below, a status line of its own: click its items. The button puts a
//! hint on this gallery's status line.

use crate::panel::Panel;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_USER, CommandId};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::status_data::StatusItemBuilder;
use turbo_vision::views::button::Button;
use turbo_vision::views::msgbox::message_box_ok;
use turbo_vision::views::status_line::StatusLine;

const SAVE: CommandId = CM_USER + 100;
const PRINT: CommandId = CM_USER + 101;
const HINT: CommandId = CM_USER + 102;

pub fn build(panel: &mut Panel) {
    panel.add(StatusLine::new(
        Rect::new(0, 0, 46, 1),
        vec![
            StatusItemBuilder::new()
                .text("~F2~ Save")
                .key("F2")
                .command(SAVE)
                .build(),
            StatusItemBuilder::new()
                .text("~F9~ Print")
                .key("F9")
                .command(PRINT)
                .build(),
            StatusItemBuilder::new().text("~Esc~ Cancel").build(),
        ],
    ));
    panel.add(Button::new(
        Rect::new(0, 2, 20, 4),
        "Show a ~h~int",
        HINT,
        true,
    ));
}

pub fn handle(app: &mut Application, command: CommandId) -> bool {
    match command {
        SAVE => {
            message_box_ok(app, "Save was clicked.");
        }
        PRINT => {
            message_box_ok(app, "Print was clicked.");
        }
        HINT => {
            if let Some(line) = app.status_line.as_mut() {
                line.set_hint(Some("A hint from the gallery".into()));
            }
            app.needs_redraw();
        }
        _ => return false,
    }
    true
}
