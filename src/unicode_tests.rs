// (C) 2025 - Enzo Lombardi
//! Every view that shows text survives multi-byte characters.
//!
//! Each view gets text with `é`, `ñ`, `−`, `€` that is wider than the view,
//! is drawn, edited and scrolled to the right one column at a time, and is
//! drawn again after every key. Cutting a string at a byte offset that falls
//! inside a character panics; `TextViewer` used to do it.

use crate::core::event::{
    Event, KB_BACKSPACE, KB_CTRL_Y, KB_DEL, KB_DOWN, KB_END, KB_ENTER, KB_HOME, KB_LEFT, KB_PGDN,
    KB_PGUP, KB_RIGHT, KB_TAB, KB_UP, KeyCode,
};
use crate::core::geometry::Rect;
use crate::core::menu_data::{Menu, MenuItemBuilder};
use crate::core::status_data::StatusItemBuilder;
use crate::views::View;
use crate::views::view::{dispatch_to_child, draw_child};
use std::cell::RefCell;
use std::rc::Rc;

const LONG: &str = "año − vacaciones − días € ñandú ü — «cita» … tést ŝ ĉ";
const KEYS: &[KeyCode] = &[
    KB_END,
    KB_LEFT,
    KB_LEFT,
    KB_DEL,
    KB_BACKSPACE,
    KB_HOME,
    KB_RIGHT,
    KB_RIGHT,
    KB_RIGHT,
    KB_DOWN,
    KB_PGDN,
    KB_END,
    KB_RIGHT,
    KB_UP,
    KB_PGUP,
    KB_CTRL_Y,
    KB_ENTER,
    KB_TAB,
];

/// Draw `view`, then feed it keys (typing `ñ` now and then) and scroll it
/// right, drawing after every event.
fn exercise(view: &mut dyn View) {
    let mut terminal = crate::test_util::test_terminal(80, 25);
    view.set_focus(true);
    draw_child(&mut terminal, view);
    let typed = std::iter::repeat_n(KB_RIGHT, 80);
    for (i, key) in KEYS.iter().copied().chain(typed).enumerate() {
        dispatch_to_child(view, &mut Event::keyboard(key));
        draw_child(&mut terminal, view);
        if i % 3 == 0 && i < KEYS.len() {
            let mut event = Event::keyboard(0);
            event.ch = Some('ñ');
            dispatch_to_child(view, &mut event);
            draw_child(&mut terminal, view);
        }
    }
}

fn small() -> Rect {
    Rect::new(0, 0, 9, 4)
}

fn line() -> Rect {
    Rect::new(0, 0, 9, 1)
}

#[test]
fn text_views() {
    let mut viewer = crate::views::text_viewer::TextViewer::new(small());
    viewer.set_text(&format!("{LONG}\n{LONG}{LONG}\nñ"));
    exercise(&mut viewer);

    let mut input = crate::views::input_line::InputLine::new(line(), 200);
    input.set_text(LONG);
    exercise(&mut input);

    let mut memo = crate::views::memo::Memo::new(small());
    memo.set_text(&format!("{LONG}\n{LONG}"));
    exercise(&mut memo);

    let mut editor = crate::views::editor::EditorWindow::new(small());
    editor.set_text(&format!("{LONG}\n{LONG}"));
    exercise(&mut editor);

    exercise(&mut crate::views::static_text::StaticText::new(
        small(),
        &format!("{LONG}\n{LONG}"),
    ));

    let mut param = crate::views::paramtext::ParamText::new(small(), "%s − años €");
    param.set_param_str(LONG);
    exercise(&mut param);
}

#[test]
fn lists() {
    let mut list = crate::views::listbox::ListBox::new(small(), 300);
    list.set_items(vec![LONG.into(), "ñ".into(), LONG.into()]);
    exercise(&mut list);

    let mut sorted = crate::views::sorted_listbox::SortedListBox::new(small(), 300);
    sorted.set_items(vec![LONG.into(), "ñ".into(), "ña".into()]);
    exercise(&mut sorted);

    let mut table = crate::views::table::Table::new(small(), 300);
    table.set_columns(vec![
        crate::views::table::Column::new("Ñame − €", 5),
        crate::views::table::Column::new(LONG, 4),
    ]);
    table.set_rows(vec![
        vec![LONG.into(), LONG.into()],
        vec!["ñ".into(), "€".into()],
    ]);
    exercise(&mut table);

    let mut outline = crate::views::outline::OutlineViewer::new(small(), String::clone);
    outline.set_roots(vec![Rc::new(RefCell::new(
        crate::views::outline::Node::with_children(
            LONG.to_string(),
            vec![Rc::new(RefCell::new(crate::views::outline::Node::new(
                LONG.to_string(),
            )))],
        ),
    ))]);
    exercise(&mut outline);

    exercise(&mut crate::views::combo_box::ComboBox::with_items(
        line(),
        4242,
        vec![LONG.into(), "ñ".into()],
    ));
}

#[test]
fn controls_and_frames() {
    let title = "~Ñ~año − días €";
    exercise(&mut crate::views::label::Label::new(line(), title));
    exercise(&mut crate::views::button::Button::new(
        Rect::new(0, 0, 9, 2),
        title,
        300,
        false,
    ));
    exercise(&mut crate::views::checkbox::CheckBox::new(line(), title));
    let labels = vec![LONG.to_string(), "~ñ~ − €".to_string()];
    exercise(&mut crate::views::cluster_group::CheckBoxes::new(
        small(),
        labels.clone(),
    ));
    exercise(&mut crate::views::cluster_group::RadioButtons::new(
        small(),
        labels,
    ));
    exercise(&mut crate::views::group_box::GroupBox::new(small(), LONG));
    exercise(&mut crate::views::frame::Frame::new(
        Rect::new(0, 0, 12, 4),
        LONG,
        true,
    ));
    exercise(&mut crate::views::window::Window::new(
        Rect::new(0, 0, 12, 4),
        LONG,
    ));

    let mut tabs = crate::views::tabbed_pane::TabbedPane::new(Rect::new(0, 0, 12, 5));
    tabs.add_page(
        title,
        crate::views::group::Group::new(Rect::new(0, 0, 12, 4)),
    );
    tabs.add_page(
        LONG,
        crate::views::group::Group::new(Rect::new(0, 0, 12, 4)),
    );
    exercise(&mut tabs);
}

#[test]
fn menus_and_status_line() {
    let mut bar = crate::views::menu_bar::MenuBar::new(Rect::new(0, 0, 12, 1));
    bar.add_submenu(crate::views::menu_bar::SubMenu::new(
        "~Ñ~año − €",
        Menu::from_items(vec![MenuItemBuilder::new().text(LONG).command(300).build()]),
    ));
    bar.add_submenu(crate::views::menu_bar::SubMenu::new(
        LONG,
        Menu::from_items(vec![]),
    ));
    exercise(&mut bar);

    exercise(&mut crate::views::status_line::StatusLine::new(
        Rect::new(0, 0, 12, 1),
        vec![
            StatusItemBuilder::new()
                .text("~Ñ~ año − €")
                .key("F2")
                .command(300)
                .build(),
            StatusItemBuilder::new()
                .text(LONG)
                .key("F3")
                .command(301)
                .build(),
        ],
    ));
}
