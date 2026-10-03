// (C) 2025 - Enzo Lombardi
//! The gallery's list of demos.
//!
//! Each entry ties a demo file to what the gallery needs to show it. The
//! file's own text is the code shown (`include_str!`), so adding a demo is:
//! write `demos/<name>.rs`, add a `pub mod` line in `demos/mod.rs`, and an
//! entry here.

use super::demos;
use super::panel::Panel;
use turbo_vision::app::Application;
use turbo_vision::core::command::CommandId;

/// The columns a demo may use: the panel's width on an 80-column terminal,
/// inside its "Try it" box.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "the tests check every demo against it")
)]
pub const PANEL_WIDTH: i16 = 48;

/// One component's demo.
pub struct Demo {
    /// The name in the list and the panel's title.
    pub name: &'static str,
    /// The module, under `turbo_vision::`.
    pub module: &'static str,
    /// The demo file, as written: a `//!` header, then the code.
    pub source: &'static str,
    /// Rows the live demo takes.
    pub height: i16,
    /// Put the demo's views in the panel, from `(0, 0)` and at most
    /// [`PANEL_WIDTH`] wide.
    pub build: fn(&mut Panel),
    /// React to the demo's commands; `true` when it handled one.
    pub handle: Option<fn(&mut Application, CommandId) -> bool>,
}

/// Every demo, in the order of the list.
pub const DEMOS: &[Demo] = &[
    Demo {
        name: "Button",
        module: "views::button",
        source: include_str!("demos/button.rs"),
        height: 2,
        build: demos::button::build,
        handle: Some(demos::button::handle),
    },
    Demo {
        name: "InputLine",
        module: "views::input_line",
        source: include_str!("demos/input_line.rs"),
        height: 5,
        build: demos::input_line::build,
        handle: None,
    },
    Demo {
        name: "CheckBoxes",
        module: "views::cluster_group",
        source: include_str!("demos/check_boxes.rs"),
        height: 4,
        build: demos::check_boxes::build,
        handle: None,
    },
    Demo {
        name: "RadioButtons",
        module: "views::cluster_group",
        source: include_str!("demos/radio_buttons.rs"),
        height: 3,
        build: demos::radio_buttons::build,
        handle: None,
    },
    Demo {
        name: "ComboBox",
        module: "views::combo_box",
        source: include_str!("demos/combo_box.rs"),
        height: 3,
        build: demos::combo_box::build,
        handle: None,
    },
    Demo {
        name: "ListBox",
        module: "views::listbox",
        source: include_str!("demos/list_box.rs"),
        height: 5,
        build: demos::list_box::build,
        handle: Some(demos::list_box::handle),
    },
    Demo {
        name: "Table",
        module: "views::table",
        source: include_str!("demos/table.rs"),
        height: 6,
        build: demos::table::build,
        handle: None,
    },
    Demo {
        name: "Form",
        module: "views::form",
        source: include_str!("demos/form.rs"),
        height: 2,
        build: demos::form::build,
        handle: Some(demos::form::handle),
    },
    Demo {
        name: "Message boxes",
        module: "views::msgbox",
        source: include_str!("demos/message_boxes.rs"),
        height: 2,
        build: demos::message_boxes::build,
        handle: Some(demos::message_boxes::handle),
    },
    Demo {
        name: "Window",
        module: "views::window",
        source: include_str!("demos/window.rs"),
        height: 2,
        build: demos::window::build,
        handle: Some(demos::window::handle),
    },
];

/// Split a demo file into its `//!` header, as plain text, and the code
/// after it.
pub fn split_source(source: &str) -> (String, String) {
    let mut how = Vec::new();
    let mut lines = source.lines().peekable();
    while let Some(line) = lines.next_if(|l| l.starts_with("//!")) {
        how.push(line.trim_start_matches("//!").trim_start().to_string());
    }
    // The blank line between the header and the code.
    lines.next_if(|l| l.trim().is_empty());
    let code: Vec<&str> = lines.collect();
    (how.join("\n"), code.join("\n"))
}
