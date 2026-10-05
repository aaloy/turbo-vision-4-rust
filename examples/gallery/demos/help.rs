//! F1 help from Markdown: each "# Title {#id}" starts a topic, and
//! "[text](#id)" links to another. In the help window Tab moves between
//! links, Enter follows one, Alt+F1 goes back, Esc closes it.
//!
//! Parameters:
//! - `app.set_help(HelpFile::from_content(text))`, or
//!   `app.set_help_file(path)`: give the application its help; F1 then
//!   opens the first topic.
//! - `app.show_help_topic(id)`: open a topic from your own command.
//! - `app.register_help_context(context, id)` and
//!   `app.set_help_context(context)`: F1 opens the topic for what the user
//!   is doing.
//!
//! See also: StatusLine, Message boxes, Tooltip

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
