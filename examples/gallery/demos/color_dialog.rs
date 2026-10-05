//! Pick a colour pair: a foreground and a background, with a sample of the
//! result. The two grids inside are ColorSelector views.
//!
//! Parameters:
//! - `ColorDialog::new(bounds, title, initial)`: `initial` is the `Attr` to
//!   start from, as in `Attr::new(TvColor::Yellow, TvColor::Blue)`.
//! - `execute(app)`: runs it; `Some(attr)` on OK, `None` when cancelled.
//!
//! See also: File dialogs, Message boxes, Help

use crate::panel::Panel;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_USER, CommandId};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::{Attr, TvColor};
use turbo_vision::views::button::Button;
use turbo_vision::views::color_dialog::ColorDialog;
use turbo_vision::views::msgbox::message_box_ok;

const PICK: CommandId = CM_USER + 100;

pub fn build(panel: &mut Panel) {
    panel.add(Button::new(
        Rect::new(0, 0, 18, 2),
        "~P~ick a colour",
        PICK,
        true,
    ));
}

pub fn handle(app: &mut Application, command: CommandId) -> bool {
    if command != PICK {
        return false;
    }
    let initial = Attr::new(TvColor::Yellow, TvColor::Blue);
    let mut dialog = ColorDialog::new(Rect::new(14, 3, 66, 20), "Colours", initial);
    let text = match dialog.execute(app) {
        Some(attr) => format!("You picked {:?} on {:?}.", attr.fg, attr.bg),
        None => "Cancelled.".to_string(),
    };
    message_box_ok(app, &text);
    true
}
