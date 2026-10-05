//! Ready-made dialogs for a message, a question or one line of input. Each
//! is one call that runs modally and returns what the user chose.
//!
//! Parameters:
//! - `message_box_ok(app, text)`: a message with OK. `message_box_error`
//!   and `message_box_warning`: the same, titled Error and Warning.
//! - `confirmation_box_yes_no(app, text)`: returns `CM_YES` or `CM_NO`;
//!   `confirmation_box_ok_cancel`, `CM_OK` or `CM_CANCEL`.
//! - `input_box(app, title, label, initial, max_length)`: one input line;
//!   `Some(text)` on OK, `None` when cancelled.
//! - `message_box(app, text, MsgBox::WARNING | MsgBox::YES_NO_CANCEL)`: any
//!   mix of icon and buttons; `MsgBox::AUTO_DISMISS` closes it on its own.
//!
//! See also: Form, File dialogs, Button

use crate::panel::Panel;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_USER, CM_YES, CommandId};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::button::Button;
use turbo_vision::views::msgbox::{confirmation_box_yes_no, input_box, message_box_ok};

const INFO: CommandId = CM_USER + 100;
const ASK: CommandId = CM_USER + 101;
const INPUT: CommandId = CM_USER + 102;

pub fn build(panel: &mut Panel) {
    panel.add(Button::new(Rect::new(0, 0, 12, 2), "~I~nfo", INFO, true));
    panel.add(Button::new(Rect::new(14, 0, 26, 2), "~A~sk", ASK, false));
    panel.add(Button::new(
        Rect::new(28, 0, 40, 2),
        "In~p~ut",
        INPUT,
        false,
    ));
}

pub fn handle(app: &mut Application, command: CommandId) -> bool {
    match command {
        INFO => {
            message_box_ok(app, "The backup has finished.");
        }
        ASK => {
            let answer = confirmation_box_yes_no(app, "Delete the old backup?");
            let text = if answer == CM_YES { "Yes" } else { "No" };
            message_box_ok(app, &format!("You answered: {text}"));
        }
        INPUT => {
            if let Some(name) = input_box(app, "Rename", "~N~ew name", "backup", 30) {
                message_box_ok(app, &format!("New name: {name}"));
            }
        }
        _ => return false,
    }
    true
}
