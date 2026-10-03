//! A pop-up menu at a point: a context menu. Build a `Menu` the way a
//! menu bar's menus are built, then `MenuBox::new(point, menu)` and
//! `execute(&mut app.terminal)`; it returns the command of the item
//! picked, or 0 when Esc closes it. Up, Down and a ~letter~ pick an item.

use crate::panel::Panel;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_USER, CommandId};
use turbo_vision::core::geometry::{Point, Rect};
use turbo_vision::core::menu_data::{Menu, MenuItem, MenuItemBuilder};
use turbo_vision::views::button::Button;
use turbo_vision::views::menu_box::MenuBox;
use turbo_vision::views::msgbox::message_box_ok;

const SHOW: CommandId = CM_USER + 100;
const CUT: CommandId = CM_USER + 101;
const COPY: CommandId = CM_USER + 102;
const PASTE: CommandId = CM_USER + 103;

pub fn build(panel: &mut Panel) {
    panel.add(Button::new(
        Rect::new(0, 0, 18, 2),
        "~S~how a menu",
        SHOW,
        true,
    ));
}

pub fn handle(app: &mut Application, command: CommandId) -> bool {
    if command != SHOW {
        return false;
    }
    let menu = Menu::from_items(vec![
        MenuItemBuilder::new().text("Cu~t~").command(CUT).build(),
        MenuItemBuilder::new().text("~C~opy").command(COPY).build(),
        MenuItem::separator(),
        MenuItemBuilder::new()
            .text("~P~aste")
            .command(PASTE)
            .build(),
    ]);
    let picked = MenuBox::new(Point::new(30, 4), menu).execute(&mut app.terminal);
    let text = match picked {
        CUT => "You picked Cut.",
        COPY => "You picked Copy.",
        PASTE => "You picked Paste.",
        _ => "No item was picked.",
    };
    message_box_ok(app, text);
    true
}
