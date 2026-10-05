// (C) 2025 - Enzo Lombardi
// Component Gallery - every component, live, with how it works and its code.
//
// The list on the left names the components; the panel on the right shows
// the one under the list's focus: the live component, "How it works", and
// the code that built it. That code is the demo's source file, shown as
// written, so what you read is what runs (docs/DESIGN-SYSTEM-PLAN.md, D3).
//
// Run with: cargo run --example gallery
// Arrows move through the list; F6 switches to the panel to try a
// component (Tab moves between its controls); F6 again goes back.
//
// The gallery follows the terminal: resize it and the list keeps its width
// and takes the new height, while the panel is laid out again for the new
// size (the "How it works" text is re-wrapped to it).

mod demos;
mod panel;
mod registry;

use registry::{DEMO_COMMANDS, DEMOS, Demo};
use std::cell::RefCell;
use std::rc::Rc;
use turbo_vision::app::{AppHandler, Application};
use turbo_vision::core::command::{CM_QUIT, CommandId};
use turbo_vision::core::event::Event;
use turbo_vision::core::geometry::{Point, Rect};
use turbo_vision::core::state::{Grow, State};
use turbo_vision::core::status_data::StatusItemBuilder;
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::editor::EditorWindow;
use turbo_vision::views::group_box::GroupBox;
use turbo_vision::views::listbox::ListBox;
use turbo_vision::views::scrollbar::ScrollBar;
use turbo_vision::views::shared::Shared;
use turbo_vision::views::static_text::StaticText;
use turbo_vision::views::status_line::StatusLine;
use turbo_vision::views::syntax::RustHighlighter;
use turbo_vision::views::window::Window;
use turbo_vision::views::{GroupLike, Handle, View, ViewId};

/// How wide the component list is.
const LIST_WIDTH: i16 = 24;

/// The gallery's state: where the list is, and which demo the panel shows.
struct Gallery {
    list_window: Handle<Window>,
    list: Handle<ListBox>,
    panel: Option<ViewId>,
    shown: Option<usize>,
    /// The desktop size the panel was built for.
    laid_out: (i16, i16),
}

impl Gallery {
    fn open(app: &mut Application) -> Self {
        let desk = desktop_size(app);
        let mut window = Window::new(Rect::new(0, 0, LIST_WIDTH, desk.1), "Components");
        no_shadow(&mut window);
        // A resize changes the list's height, never its width.
        window.set_grow_mode(Grow::HI_Y);
        let mut list = ListBox::new(Rect::new(0, 0, LIST_WIDTH - 2, desk.1 - 2), 0);
        list.set_grow_mode(Grow::HI_Y);
        list.set_items(DEMOS.iter().map(|d| d.name.to_string()).collect());
        let list = window.add_typed(list);
        let list_window = app.desktop.add_typed(window);
        Self {
            list_window,
            list,
            panel: None,
            shown: None,
            laid_out: desk,
        }
    }

    /// The demo under the list's focus.
    fn selected(&self, app: &Application) -> Option<usize> {
        app.desktop
            .get(self.list_window)?
            .get(self.list)?
            .get_selection()
    }

    /// Show demo `index` in a fresh panel, keeping the focus in the list.
    fn show(&mut self, app: &mut Application, index: usize) {
        if let Some(old) = self.panel.take() {
            app.desktop.remove_child_by_id(old);
        }
        for command in DEMO_COMMANDS {
            app.enable_command(command);
        }
        let (width, height) = desktop_size(app);
        let bounds = Rect::new(LIST_WIDTH, 0, width, height);
        self.panel = Some(app.desktop.add(panel(&DEMOS[index], bounds)));
        self.shown = Some(index);
        self.laid_out = (width, height);
        app.desktop.bring_to_front(self.list_window.id());
        app.needs_redraw();
    }
}

impl AppHandler for Gallery {
    fn idle(&mut self, app: &mut Application) {
        // The panel follows the list (D7), and the terminal: grow modes
        // stretch it at once, but its text was wrapped to the old width, so
        // it is built again for the new size.
        let selected = self.selected(app);
        let resized = desktop_size(app) != self.laid_out;
        if let Some(index) = selected.filter(|&i| resized || Some(i) != self.shown) {
            self.show(app, index);
        }
    }

    fn handle_command(&mut self, app: &mut Application, command: CommandId, _: &Event) -> bool {
        // Only the shown demo is asked about its commands (D8).
        self.shown
            .and_then(|i| DEMOS[i].handle)
            .is_some_and(|handle| handle(app, command))
    }

    fn window_closed(&mut self, _app: &mut Application, id: ViewId) {
        // A closed panel is rebuilt on the next idle tick.
        if Some(id) == self.panel {
            self.panel = None;
            self.shown = None;
        }
    }
}

/// Side by side, the list and the panel fill the desktop: without a shadow
/// a window may reach its edges.
fn no_shadow(window: &mut impl View) {
    window.set_state(window.state() & !State::SHADOW);
}

/// Re-flow `text` to lines at most `width` wide: lines that follow each
/// other form one paragraph, and a blank line starts a new one.
fn wrap(text: &str, width: usize) -> String {
    let mut out: Vec<String> = Vec::new();
    for paragraph in text.split("\n\n") {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let shown = |s: &str| s.chars().filter(|&c| c != '~').count();
            if !line.is_empty() && shown(&line) + 1 + shown(word) > width {
                out.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        out.push(line);
        out.push(String::new());
    }
    out.pop();
    out.join("\n")
}

/// The desktop's width and height.
fn desktop_size(app: &Application) -> (i16, i16) {
    let b = app.desktop.get_bounds();
    (b.width(), b.height())
}

/// The panel for `demo`: the live component in a "Try it" box, then how it
/// works, then its code.
fn panel(demo: &Demo, bounds: Rect) -> Dialog {
    let mut dialog = Dialog::new(bounds, demo.name);
    no_shadow(&mut dialog);
    // Until it is rebuilt after a resize, the panel stretches with the
    // desktop: its right and bottom edges follow the terminal's.
    dialog.set_grow_mode(Grow::HI_X | Grow::HI_Y);
    let inner = bounds.width() - 2;
    let (how, code) = registry::split_source(demo.source);
    let how = wrap(&how, usize::try_from(inner - 2).unwrap_or(1));

    // Module path and summary on the first row.
    dialog.add(StaticText::new(
        Rect::new(1, 0, inner - 1, 1),
        &format!("turbo_vision::{}", demo.module),
    ));

    // The live component, inside a box.
    let top = 1;
    let box_height = demo.height + 2;
    let mut try_it = GroupBox::new(
        Rect::new(1, top, inner - 1, top + box_height),
        "Try it (F6)",
    );
    try_it.set_grow_mode(Grow::HI_X);
    dialog.add(try_it);
    (demo.build)(&mut panel::Panel::new(&mut dialog, Point::new(3, top + 1)));

    // How it works: the demo file's `//!` header.
    let how_top = top + box_height;
    let how_lines = i16::try_from(how.lines().count()).unwrap_or(1).max(1);
    dialog.add(StaticText::new(
        Rect::new(1, how_top, inner - 1, how_top + how_lines),
        &how,
    ));

    // The code: the rest of the file, as written.
    let code_top = how_top + how_lines + 1;
    let code_bottom = bounds.height() - 2;
    if code_bottom - code_top >= 3 {
        add_code(
            &mut dialog,
            Rect::new(1, code_top, inner - 1, code_bottom),
            &code,
        );
    }
    dialog
}

/// Show `code` in `area` of `dialog`, coloured as Rust: a read-only editor
/// with a scroll bar on its right and one along its bottom.
fn add_code(dialog: &mut Dialog, area: Rect, code: &str) {
    let (right, bottom) = (area.b.x - 1, area.b.y - 1);
    let mut v_bar = ScrollBar::new_vertical(Rect::new(right, area.a.y, area.b.x, bottom));
    v_bar.set_grow_mode(Grow::LO_X | Grow::HI_X | Grow::HI_Y);
    let mut h_bar = ScrollBar::new_horizontal(Rect::new(area.a.x, bottom, right, area.b.y));
    h_bar.set_grow_mode(Grow::LO_Y | Grow::HI_Y | Grow::HI_X);
    let (v_bar, h_bar) = (Rc::new(RefCell::new(v_bar)), Rc::new(RefCell::new(h_bar)));

    let mut editor = EditorWindow::with_scrollbars(
        Rect::new(area.a.x, area.a.y, right, bottom),
        Some(Rc::clone(&h_bar)),
        Some(Rc::clone(&v_bar)),
        None,
    );
    editor.set_highlighter(Box::new(RustHighlighter::new()));
    editor.set_text(code);
    editor.set_read_only(true);
    dialog.add(editor);
    dialog.add(Shared::new(v_bar));
    dialog.add(Shared::new(h_bar));
}

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();
    app.set_status_line(StatusLine::new(
        Rect::new(0, height - 1, width, height),
        vec![
            StatusItemBuilder::new()
                .text("~Alt-X~ Exit")
                .key("Alt+X")
                .command(CM_QUIT)
                .build(),
            StatusItemBuilder::new().text("~F6~ List/Panel").build(),
            StatusItemBuilder::new().text("~Tab~ Next control").build(),
        ],
    ));
    let mut gallery = Gallery::open(&mut app);
    gallery.show(&mut app, 0);
    app.run_with(&mut gallery);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use registry::PANEL_WIDTH;

    /// A dialog holding only what `demo` builds, with the panel's corner
    /// at (0, 0).
    fn built(demo: &Demo) -> Dialog {
        // As `Gallery::show` does: a command an earlier demo disabled (D12)
        // would grey out this demo's button.
        for command in DEMO_COMMANDS {
            turbo_vision::core::command_set::enable_command(command);
        }
        let mut dialog = Dialog::new(Rect::new(0, 0, PANEL_WIDTH + 2, demo.height + 2), "T");
        (demo.build)(&mut panel::Panel::new(&mut dialog, Point::new(0, 0)));
        dialog
    }

    #[test]
    fn every_demo_builds_inside_its_panel() {
        for demo in DEMOS {
            let live = built(demo);
            assert!(live.child_count() > 0, "{} shows nothing", demo.name);
            for i in 0..live.child_count() {
                let b = live.child_at(i).bounds();
                assert!(
                    b.a.x >= 0 && b.a.y >= 0 && b.b.x <= PANEL_WIDTH && b.b.y <= demo.height,
                    "{}: a view at {b:?} is outside {}x{}",
                    demo.name,
                    PANEL_WIDTH,
                    demo.height
                );
            }
        }
    }

    #[test]
    fn every_demo_explains_how_it_works() {
        for demo in DEMOS {
            let (how, code) = registry::split_source(demo.source);
            assert!(
                how.lines().count() >= 2,
                "{}: write a //! header",
                demo.name
            );
            assert!(
                code.contains("pub fn build"),
                "{}: no build function",
                demo.name
            );
            assert!(
                !code.starts_with('\n'),
                "{}: one blank line after the header",
                demo.name
            );
        }
    }

    #[test]
    fn a_demo_s_hot_keys_are_unique() {
        for demo in DEMOS {
            let (_, code) = registry::split_source(demo.source);
            let mut seen = std::collections::HashSet::new();
            // Only the views the demo builds into its panel, not the
            // dialogs its handler opens.
            let build = code.split("pub fn handle").next().unwrap_or_default();
            // A hot key is one letter; a longer marker, ~F2~ in a status
            // line item, highlights a key name.
            let hot_keys = build.split('~').skip(1).step_by(2);
            for part in hot_keys.filter(|p| p.chars().count() == 1) {
                let key = part.to_ascii_lowercase();
                assert!(
                    seen.insert(key.clone()),
                    "{}: two ~{key}~ hot keys",
                    demo.name
                );
            }
        }
    }

    #[test]
    fn demo_commands_stay_in_their_range() {
        // D8: CM_USER + 100 to CM_USER + 199 belong to the demos.
        for demo in DEMOS {
            let (_, code) = registry::split_source(demo.source);
            for line in code.lines().filter(|l| l.contains(": CommandId = CM_USER")) {
                let offset: u16 = line
                    .split("CM_USER +")
                    .nth(1)
                    .and_then(|n| n.trim().trim_end_matches(';').parse().ok())
                    .unwrap_or_else(|| panic!("{}: `{line}`", demo.name));
                assert!(
                    DEMO_COMMANDS.contains(&(CM_USER + offset)),
                    "{}: `{line}` is outside {DEMO_COMMANDS:?}",
                    demo.name
                );
            }
        }
    }

    /// View modules with no demo of their own, and why.
    const NOT_DEMOED: &[(&str, &str)] = &[
        ("background", "the desktop's pattern, behind every demo"),
        ("cluster", "the trait CheckBoxes and RadioButtons share"),
        ("color_selector", "part of ColorDialog"),
        ("desktop", "every demo runs on it"),
        ("dir_listbox", "part of the folder dialog"),
        ("editor_traits", "the traits the editors share"),
        ("file_editor", "the Editor demo's editor, tied to a file"),
        ("file_list", "part of the file dialog"),
        ("frame", "every window's and dialog's border"),
        ("handle", "typed handles to views, not a view"),
        ("help_context", "part of the help the Help demo opens"),
        ("help_index", "part of the help the Help demo opens"),
        ("help_toc", "part of the help the Help demo opens"),
        ("help_viewer", "part of the help the Help demo opens"),
        ("help_window", "the window the Help demo opens"),
        ("history_viewer", "part of the History demo's popup"),
        ("history_window", "the History demo's popup"),
        ("indicator", "the line and column in an EditWindow"),
        ("list_viewer", "the trait the lists share"),
        ("lookup_validator", "a validator, not a view; see InputLine"),
        ("menu_viewer", "the trait the menus share"),
        ("scrollbar", "part of the lists, TextViewer and Editor"),
        ("scroller", "the base of TextViewer"),
        ("shared", "helpers, not a view"),
        ("view", "the trait every view implements"),
    ];

    #[test]
    fn every_view_module_has_a_demo_or_a_reason() {
        // A demo covers the modules its code imports.
        let views = concat!(env!("CARGO_MANIFEST_DIR"), "/src/views");
        let mut modules: Vec<String> = std::fs::read_dir(views)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .map(|name| name.trim_end_matches(".rs").to_string())
            .filter(|name| name != "mod")
            .collect();
        modules.sort();
        let imported = |module: &str| {
            let path = format!("turbo_vision::views::{module}");
            DEMOS.iter().any(|d| {
                d.source.lines().any(|l| {
                    l.split_once(&path).is_some_and(|(_, rest)| {
                        !rest.starts_with(char::is_alphanumeric) && !rest.starts_with('_')
                    })
                })
            })
        };
        for module in &modules {
            let reason = NOT_DEMOED.iter().any(|(m, _)| m == module);
            assert!(
                imported(module) || reason,
                "views::{module} has no demo: write one, or add it to NOT_DEMOED with a reason"
            );
            assert!(
                !(imported(module) && reason),
                "views::{module} has a demo now: take it out of NOT_DEMOED"
            );
        }
        for (module, _) in NOT_DEMOED {
            assert!(
                modules.contains(&module.to_string()),
                "views::{module} is gone"
            );
        }
    }

    #[test]
    fn wrap_reflows_paragraphs_to_the_width() {
        let text = "one two three\nfour five\n\nsix";
        assert_eq!(wrap(text, 9), "one two\nthree\nfour five\n\nsix");
        assert_eq!(wrap("press ~O~K now", 6), "press\n~O~K now");
    }

    #[test]
    fn a_demo_s_controls_can_take_the_focus() {
        for demo in DEMOS {
            let live = built(demo);
            assert!(
                (0..live.child_count()).any(|i| live.child_at(i).can_focus()),
                "{}: nothing to try",
                demo.name
            );
        }
    }

    /// A terminal whose size the test changes, as a user resizing it does.
    struct Resizable(Arc<(AtomicU16, AtomicU16)>);

    impl turbo_vision::terminal::Backend for Resizable {
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
        fn init(&mut self) -> std::io::Result<()> {
            Ok(())
        }
        fn cleanup(&mut self) -> std::io::Result<()> {
            Ok(())
        }
        fn size(&self) -> std::io::Result<(u16, u16)> {
            Ok((
                self.0.0.load(Ordering::SeqCst),
                self.0.1.load(Ordering::SeqCst),
            ))
        }
        fn poll_event(&mut self, _: std::time::Duration) -> std::io::Result<Option<Event>> {
            Ok(None)
        }
        fn write_raw(&mut self, _: &[u8]) -> std::io::Result<()> {
            Ok(())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
        fn show_cursor(&mut self, _: u16, _: u16) -> std::io::Result<()> {
            Ok(())
        }
        fn hide_cursor(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    use std::sync::Arc;
    use std::sync::atomic::{AtomicU16, Ordering};
    use turbo_vision::core::command::CM_USER;

    /// An application on a [`Resizable`] terminal, and the handle that
    /// changes its size.
    fn app(width: u16, height: u16) -> (Application, Arc<(AtomicU16, AtomicU16)>) {
        let size = Arc::new((AtomicU16::new(width), AtomicU16::new(height)));
        let backend = Box::new(Resizable(Arc::clone(&size)));
        let terminal = turbo_vision::terminal::Terminal::with_backend(backend).unwrap();
        (Application::with_terminal(terminal), size)
    }

    #[test]
    fn the_gallery_follows_a_terminal_resize() {
        let (mut app, size) = app(80, 25);
        let mut gallery = Gallery::open(&mut app);
        gallery.show(&mut app, 0);

        size.0.store(120, Ordering::SeqCst);
        size.1.store(40, Ordering::SeqCst);
        app.step(&mut gallery, None); // idle: the application and the gallery see the new size

        let (width, height) = desktop_size(&app);
        let list = app.desktop.get(gallery.list_window).unwrap().bounds();
        assert_eq!((list.width(), list.height()), (LIST_WIDTH, height));
        let panel = app
            .desktop
            .child_by_id(gallery.panel.unwrap())
            .unwrap()
            .bounds();
        assert_eq!(panel, Rect::new(LIST_WIDTH, 0, width, height));
        assert_eq!(gallery.shown, Some(0), "the same demo is shown again");
    }

    #[test]
    fn a_command_one_demo_disables_is_enabled_for_the_next() {
        // The Button demo disables a command; the MenuBox demo uses the same
        // number for an item.
        let (mut app, _) = app(80, 25);
        let mut gallery = Gallery::open(&mut app);
        let button = DEMOS.iter().position(|d| d.name == "Button").unwrap();
        gallery.show(&mut app, button);
        let disabled = DEMO_COMMANDS
            .filter(|&c| !app.command_enabled(c))
            .collect::<Vec<_>>();
        assert!(!disabled.is_empty(), "the Button demo disables a command");
        gallery.show(&mut app, button + 1);
        assert!(disabled.iter().all(|&c| app.command_enabled(c)));
    }

    #[test]
    fn the_panel_builds_for_every_demo() {
        for demo in DEMOS {
            let dialog = panel(demo, Rect::new(LIST_WIDTH, 0, 80, 23));
            assert_eq!(dialog.bounds().width(), 80 - LIST_WIDTH);
        }
    }
}
