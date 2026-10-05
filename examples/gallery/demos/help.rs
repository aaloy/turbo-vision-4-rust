//! F1 help from Markdown: each "# Title {#id}" starts a topic, and
//! "[text](#id)" links to another. Give the application the help with
//! `set_help` (or `set_help_file` for a file); F1 then opens the first
//! topic, and `show_help_topic` opens any. In the help window Tab moves
//! between links, Enter follows one, Alt+F1 goes back, Esc closes it.

use crate::panel::Panel;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_USER, CommandId};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::button::Button;
use turbo_vision::views::help_file::HelpFile;

const OPEN: CommandId = CM_USER + 100;

const HELP: &str = "\
# Invoices {#invoices}

An invoice bills a customer for the lines on it.
See [Customers](#customers) for who can be billed.

# Customers {#customers}

A customer has a name, an address and a credit limit.
Back to [Invoices](#invoices).
";

pub fn build(panel: &mut Panel) {
    panel.add(Button::new(
        Rect::new(0, 0, 16, 2),
        "~O~pen help",
        OPEN,
        true,
    ));
}

pub fn handle(app: &mut Application, command: CommandId) -> bool {
    if command != OPEN {
        return false;
    }
    app.set_help(HelpFile::from_content(HELP));
    app.show_help_topic("invoices");
    true
}
