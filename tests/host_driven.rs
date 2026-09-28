//! An embedder drives an Application by pushing events and reading cells.

use turbo_vision::app::{AppHandler, Application};
use turbo_vision::core::command::{CM_CANCEL, CommandId};
use turbo_vision::core::event::Event;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::keys::{KeyCode, KeyEvent, KeyModifiers};
use turbo_vision::terminal::{HostBackend, Terminal};
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::input_line::InputLine;
use turbo_vision::views::window::WindowBuilder;
use turbo_vision::views::{GroupLike, View};

fn app(w: u16, h: u16) -> (Application, turbo_vision::terminal::HostInput) {
    let (backend, input) = HostBackend::new(w, h);
    let terminal = Terminal::with_backend(Box::new(backend)).expect("terminal");
    let mut app = Application::with_terminal(terminal);
    // Without a menu bar and status line the desktop keeps a spare row at
    // the top and bottom (Borland: TProgram::initDeskTop's r.a.y++ and
    // r.b.y--); give it the whole screen so window rows are screen rows.
    let (sw, sh) = app.terminal.size();
    app.desktop.set_bounds(Rect::new(0, 0, sw, sh));
    (app, input)
}

fn key(c: char) -> Event {
    Event::from_crossterm_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty()))
}

fn row_text(app: &Application, y: usize) -> String {
    app.terminal.buffer()[y].iter().map(|c| c.ch).collect()
}

#[test]
fn pump_draws_without_blocking_and_reports_running() {
    let (mut app, _input) = app(40, 10);
    let mut window = WindowBuilder::new()
        .bounds(Rect::new(0, 0, 40, 10))
        .title("Hello")
        .build();
    window.add(InputLine::new(Rect::new(1, 1, 30, 2), 50));
    app.desktop.add(window);
    assert!(app.pump(&mut ()));
    assert!(
        row_text(&app, 0).contains("Hello"),
        "{:?}",
        row_text(&app, 0)
    );
}

#[test]
fn pushed_keys_reach_the_focused_view() {
    let (mut app, input) = app(40, 10);
    let mut window = WindowBuilder::new()
        .bounds(Rect::new(0, 0, 40, 10))
        .title("T")
        .build();
    // Children sit in the window's interior, inside the frame: interior row 0
    // is screen row 1.
    window.add(InputLine::new(Rect::new(1, 0, 30, 1), 50));
    app.desktop.add(window);
    for c in "Hi".chars() {
        input.push(key(c));
    }
    assert_eq!(input.len(), 2);
    app.pump(&mut ());
    assert_eq!(input.len(), 0, "pump drains the queue");
    assert!(row_text(&app, 1).contains("Hi"), "{:?}", row_text(&app, 1));
}

#[test]
fn a_modal_call_returns_cancel_instead_of_blocking() {
    let (mut app, _input) = app(40, 10);
    assert!(app.is_host_driven());
    let dialog = *Dialog::new_modal(Rect::new(5, 2, 35, 8), "Modal");
    let started = std::time::Instant::now();
    let result: CommandId = app.exec_view(dialog);
    assert_eq!(result, CM_CANCEL);
    assert!(started.elapsed().as_millis() < 500);
}

#[test]
fn a_handler_sees_commands_during_pump() {
    struct Seen(Vec<CommandId>);
    impl AppHandler for Seen {
        fn handle_command(&mut self, _: &mut Application, c: CommandId, _: &Event) -> bool {
            self.0.push(c);
            true
        }
    }
    let (mut app, input) = app(40, 10);
    input.push(Event::command(1234));
    let mut seen = Seen(Vec::new());
    app.pump(&mut seen);
    assert_eq!(seen.0, vec![1234]);
}

#[test]
fn set_size_resizes_on_the_next_pump() {
    let (mut app, input) = app(40, 10);
    input.set_size(60, 20);
    app.pump(&mut ());
    assert_eq!(app.terminal.size(), (60, 20));
}

#[test]
fn a_refused_modal_leaves_the_desktop_child_count_unchanged() {
    let (mut app, _input) = app(40, 10);
    let before = app.desktop.child_count();
    let dialog = *Dialog::new_modal(Rect::new(5, 2, 35, 8), "Modal");
    let _ = app.exec_view(dialog);
    assert_eq!(app.desktop.child_count(), before);
}

#[test]
fn execute_modal_on_a_host_driven_app_returns_promptly() {
    use turbo_vision::app::ModalTick;

    let (mut app, _input) = app(40, 10);
    let mut dialog = *Dialog::new_modal(Rect::new(5, 2, 35, 8), "Modal");
    let started = std::time::Instant::now();
    let result: CommandId = app.execute_modal(&mut dialog, |_, _| ModalTick::Continue);
    assert_eq!(result, CM_CANCEL);
    assert!(started.elapsed().as_millis() < 500);
}

#[test]
fn a_pumped_show_history_command_returns_promptly_without_popping_up() {
    use turbo_vision::core::command::CM_SHOW_HISTORY;

    let (mut app, input) = app(40, 10);
    input.push(Event::command(CM_SHOW_HISTORY));
    let started = std::time::Instant::now();
    app.pump(&mut ());
    assert!(started.elapsed().as_millis() < 500);
}

#[test]
fn a_pumped_show_dropdown_command_returns_promptly_without_popping_up() {
    use turbo_vision::core::command::CM_SHOW_DROPDOWN;

    let (mut app, input) = app(40, 10);
    input.push(Event::command(CM_SHOW_DROPDOWN));
    let started = std::time::Instant::now();
    app.pump(&mut ());
    assert!(started.elapsed().as_millis() < 500);
}

/// A table in a blue window: one resolved attribute per state, and the
/// optional separator in each gap.
mod table_colours {
    use super::app;
    use turbo_vision::core::geometry::Rect;
    use turbo_vision::core::palette::Attr;
    use turbo_vision::core::palette::palettes::{
        CP_APP_COLOR, CP_BLUE_WINDOW, CP_GRAY_DIALOG, CP_LISTBOX, CP_TABLE_WINDOW,
    };
    use turbo_vision::views::GroupLike;
    use turbo_vision::views::dialog::Dialog;
    use turbo_vision::views::table::{Column, Table};
    use turbo_vision::views::window::WindowBuilder;

    /// Resolve a table palette slot through an owner palette to the app.
    fn resolve(table: &[u8], owner: &[u8], slot: usize) -> Attr {
        let owner_index = table[slot - 1] as usize;
        let app_index = owner[owner_index - 1] as usize;
        Attr::from_u8(CP_APP_COLOR[app_index - 1])
    }

    /// Columns 4, 3 and 5 wide at x 0, 5 and 9; gaps at x 4 and 8.
    fn table(separator: bool) -> Table {
        let mut t = Table::new(Rect::new(0, 0, 20, 4), 0);
        t.set_columns(vec![
            Column::new("A", 4),
            Column::new("B", 3),
            Column::new("C", 5),
        ]);
        t.set_rows(vec![
            vec!["a0".into(), "b0".into(), "c0".into()],
            vec!["a1".into(), "b1".into(), "c1".into()],
            vec!["a2".into(), "b2".into(), "c2".into()],
        ]);
        t.set_selected_row(1);
        t.set_selected_col(1);
        t.set_column_separator(separator);
        t
    }

    const GAPS: [usize; 2] = [4, 8];

    /// Screen row of table line `y`: the window interior starts at (1, 1).
    fn cell(app: &turbo_vision::app::Application, x: usize, y: usize) -> (char, Attr) {
        let c = app.terminal.buffer()[1 + y][1 + x];
        (c.ch, c.attr)
    }

    fn in_blue_window(separator: bool) -> turbo_vision::app::Application {
        let (mut app, _input) = app(40, 10);
        let mut window = WindowBuilder::new()
            .bounds(Rect::new(0, 0, 30, 8))
            .title("T")
            .build();
        window.add(table(separator));
        app.desktop.add(window);
        app.pump(&mut ());
        app
    }

    #[test]
    fn a_table_in_a_window_uses_the_window_list_colours() {
        let app = in_blue_window(false);
        // Focused list, normal row: slot 2 -> window 6 -> app 13 = 0x1E.
        let normal = resolve(CP_TABLE_WINDOW, CP_BLUE_WINDOW, 2);
        // Selected row: slot 3 -> window 7 -> app 14 = 0x71.
        let selected = resolve(CP_TABLE_WINDOW, CP_BLUE_WINDOW, 3);
        assert_eq!(normal, Attr::from_u8(0x1E));
        assert_eq!(selected, Attr::from_u8(0x71));

        // Line 0 is the header; line 1 is row 0, line 2 the selected row 1.
        for x in 0..12 {
            assert_eq!(cell(&app, x, 1).1, normal, "unselected row at x {x}");
        }
        let focused_cell = cell(&app, 5, 2).1;
        for x in (0..12).filter(|x| !(5..8).contains(x)) {
            assert_eq!(cell(&app, x, 2).1, selected, "selected row at x {x}");
        }
        for x in GAPS {
            assert_eq!(
                cell(&app, x, 2).1,
                selected,
                "gap at x {x} is part of the bar"
            );
        }
        assert_ne!(focused_cell, selected, "focused cell stands out of its row");
        assert_ne!(focused_cell, normal, "focused cell is not a normal cell");
        assert_eq!(cell(&app, 5, 2).0, 'b');
        assert_eq!(cell(&app, 4, 1).0, ' ', "no separator unless asked for");
    }

    #[test]
    fn the_separator_sits_in_every_gap() {
        let app = in_blue_window(true);
        let header = resolve(CP_TABLE_WINDOW, CP_BLUE_WINDOW, 4);
        let normal = resolve(CP_TABLE_WINDOW, CP_BLUE_WINDOW, 2);
        let selected = resolve(CP_TABLE_WINDOW, CP_BLUE_WINDOW, 3);
        for x in GAPS {
            assert_eq!(cell(&app, x, 0), ('│', header), "header gap at x {x}");
            assert_eq!(cell(&app, x, 1), ('│', normal), "row gap at x {x}");
            assert_eq!(cell(&app, x, 2), ('│', selected), "selected gap at x {x}");
        }
        // Columns keep their places: text still starts where it did.
        assert_eq!(cell(&app, 0, 1).0, 'a');
        assert_eq!(cell(&app, 5, 1).0, 'b');
        assert_eq!(cell(&app, 9, 1).0, 'c');
        assert_eq!(
            cell(&app, 14, 1).0,
            ' ',
            "no separator after the last column"
        );
    }

    #[test]
    fn a_table_in_a_dialog_keeps_the_dialog_list_colours() {
        let (mut app, _input) = app(40, 10);
        let mut dialog = Dialog::new(Rect::new(0, 0, 30, 8), "D");
        dialog.add(table(false));
        app.desktop.add(dialog);
        app.pump(&mut ());
        // Slot 2 -> dialog 26 -> app 57 = 0x30; slot 3 -> 27 -> 58 = 0x2F.
        let normal = resolve(CP_LISTBOX, CP_GRAY_DIALOG, 2);
        let selected = resolve(CP_LISTBOX, CP_GRAY_DIALOG, 3);
        assert_eq!(normal, Attr::from_u8(0x30));
        assert_eq!(selected, Attr::from_u8(0x2F));
        assert_eq!(cell(&app, 0, 1).1, normal);
        assert_eq!(cell(&app, 4, 2).1, selected);
        let focused_cell = cell(&app, 5, 2).1;
        assert_ne!(focused_cell, selected);
        assert_ne!(focused_cell, normal);
    }
}
