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
