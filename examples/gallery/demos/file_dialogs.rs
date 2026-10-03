//! The standard Open / Save dialog and the change-folder dialog. Each is
//! built, run with `execute(app)`, and returns `Some(path)` or `None` when
//! cancelled. The file dialog takes a title, a wildcard (`*.rs`) and an
//! optional starting folder; typing a folder or `dir/*.ext` in its name
//! field moves there.

use crate::panel::Panel;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_USER, CommandId};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::button::Button;
use turbo_vision::views::chdir_dialog::ChDirDialog;
use turbo_vision::views::file_dialog::FileDialog;
use turbo_vision::views::msgbox::message_box_ok;

const OPEN: CommandId = CM_USER + 100;
const CHDIR: CommandId = CM_USER + 101;

pub fn build(panel: &mut Panel) {
    panel.add(Button::new(Rect::new(0, 0, 14, 2), "~O~pen...", OPEN, true));
    panel.add(Button::new(
        Rect::new(16, 0, 34, 2),
        "~C~hange folder",
        CHDIR,
        false,
    ));
}

pub fn handle(app: &mut Application, command: CommandId) -> bool {
    let chosen = match command {
        OPEN => FileDialog::new(Rect::new(10, 2, 70, 20), "Open", "*.rs", None)
            .build()
            .execute(app),
        CHDIR => ChDirDialog::new(None).execute(app),
        _ => return false,
    };
    let text = chosen.map_or_else(
        || "Cancelled.".to_string(),
        |path| format!("You chose\n{}", path.display()),
    );
    message_box_ok(app, &text);
    true
}
