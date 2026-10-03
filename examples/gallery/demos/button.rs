//! Sends a command when pressed: click it, or press Enter or Space while it
//! has the focus, or Alt and its ~letter~ from anywhere in the dialog.
//! The default button (last argument `true`) is the one Enter presses when
//! the focus is not on a button. A button greys out when its command is
//! disabled: `app.disable_command(c)`.

use crate::panel::Panel;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_USER, CommandId};
use turbo_vision::core::command_set;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::button::Button;
use turbo_vision::views::msgbox::message_box_ok;

const SAVE: CommandId = CM_USER + 100;
const PRINT: CommandId = CM_USER + 101;
const ARCHIVE: CommandId = CM_USER + 102;

pub fn build(panel: &mut Panel) {
    panel.add(Button::new(Rect::new(0, 0, 12, 2), "~S~ave", SAVE, true));
    panel.add(Button::new(
        Rect::new(14, 0, 26, 2),
        "~P~rint",
        PRINT,
        false,
    ));
    // Disabled through its command, so every Archive button and menu item
    // greys out together.
    command_set::disable_command(ARCHIVE);
    panel.add(Button::new(
        Rect::new(28, 0, 41, 2),
        "~A~rchive",
        ARCHIVE,
        false,
    ));
}

pub fn handle(app: &mut Application, command: CommandId) -> bool {
    let text = match command {
        SAVE => "Save sent its command (the default button).",
        PRINT => "Print sent its command.",
        _ => return false,
    };
    message_box_ok(app, text);
    true
}
