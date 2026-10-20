//! The bar of pull-down menus on the top row. F10 or a click opens it, Alt
//! and a menu's ~letter~ opens that menu. Picking an item sends its command
//! to your handler, as a button does. Try it: the button puts a bar on this
//! gallery's top row and takes it away again.
//!
//! Parameters:
//! - `MenuBar::new(bounds)`: one row across the top;
//!   `app.set_menu_bar(bar)` installs it.
//! - `add_submenu(SubMenu::new(title, menu))`: a menu on the bar; mark its
//!   hot key with tildes.
//! - `Menu::from_items(items)`: a menu's items; `MenuItem::separator()`
//!   draws a line between them.
//! - `MenuItemBuilder::new().text(t).command(c)`: an item; `.key("Ctrl+O")`
//!   binds a key and shows it, `.enabled(false)` greys it out,
//!   `.checked(f)` puts a check mark by it while `f()` is true,
//!   `.help_ctx(n)` sets its help context.
//! - An item's command greys out with `app.disable_command`, as a button's
//!   does.
//!
//! See also: MenuBox, StatusLine, Button

use crate::panel::Panel;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_USER, CommandId};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::menu_data::{Menu, MenuItem, MenuItemBuilder};
use turbo_vision::views::button::Button;
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::msgbox::message_box_ok;

const TOGGLE: CommandId = CM_USER + 100;
const NEW: CommandId = CM_USER + 101;
const OPEN: CommandId = CM_USER + 102;
const ABOUT: CommandId = CM_USER + 103;

pub fn build(panel: &mut Panel) {
    panel.add(Button::new(
        Rect::new(0, 0, 24, 2),
        "~T~oggle the menu bar",
        TOGGLE,
        true,
    ));
}

pub fn handle(app: &mut Application, command: CommandId) -> bool {
    match command {
        TOGGLE if app.menu_bar.is_some() => app.menu_bar = None,
        TOGGLE => {
            let (width, _) = app.terminal.size();
            let mut bar = MenuBar::new(Rect::new(0, 0, width, 1));
            bar.add_submenu(SubMenu::new(
                "~F~ile",
                Menu::from_items(vec![
                    MenuItemBuilder::new().text("~N~ew").command(NEW).build(),
                    MenuItemBuilder::new()
                        .text("~O~pen...")
                        .command(OPEN)
                        .build(),
                    MenuItem::separator(),
                    MenuItemBuilder::new()
                        .text("~A~bout")
                        .command(ABOUT)
                        .build(),
                ]),
            ));
            app.set_menu_bar(bar);
        }
        NEW => {
            message_box_ok(app, "You picked New.");
        }
        OPEN => {
            message_box_ok(app, "You picked Open.");
        }
        ABOUT => {
            message_box_ok(app, "A menu bar, from the gallery.");
        }
        _ => return false,
    }
    app.needs_redraw();
    true
}
