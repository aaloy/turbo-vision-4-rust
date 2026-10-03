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

mod demos;
mod panel;
mod registry;

use registry::{DEMOS, Demo};
use turbo_vision::app::{AppHandler, Application};
use turbo_vision::core::command::{CM_QUIT, CommandId};
use turbo_vision::core::event::Event;
use turbo_vision::core::geometry::{Point, Rect};
use turbo_vision::core::state::State;
use turbo_vision::core::status_data::StatusItemBuilder;
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::group_box::GroupBox;
use turbo_vision::views::listbox::ListBox;
use turbo_vision::views::static_text::StaticText;
use turbo_vision::views::status_line::StatusLine;
use turbo_vision::views::text_viewer::TextViewer;
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
}

impl Gallery {
    fn open(app: &mut Application) -> Self {
        let desk = desktop_size(app);
        let mut window = Window::new(Rect::new(0, 0, LIST_WIDTH, desk.1), "Components");
        no_shadow(&mut window);
        let mut list = ListBox::new(Rect::new(0, 0, LIST_WIDTH - 2, desk.1 - 2), 0);
        list.set_items(DEMOS.iter().map(|d| d.name.to_string()).collect());
        let list = window.add_typed(list);
        let list_window = app.desktop.add_typed(window);
        Self {
            list_window,
            list,
            panel: None,
            shown: None,
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
        let (width, height) = desktop_size(app);
        let bounds = Rect::new(LIST_WIDTH, 0, width, height);
        self.panel = Some(app.desktop.add(panel(&DEMOS[index], bounds)));
        self.shown = Some(index);
        app.desktop.bring_to_front(self.list_window.id());
        app.needs_redraw();
    }
}

impl AppHandler for Gallery {
    fn idle(&mut self, app: &mut Application) {
        // The panel follows the list (D7).
        let selected = self.selected(app);
        if selected.is_some() && selected != self.shown {
            self.show(app, selected.unwrap_or_default());
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
    dialog.add(GroupBox::new(
        Rect::new(1, top, inner - 1, top + box_height),
        "Try it (F6)",
    ));
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
    if code_bottom > code_top {
        let mut viewer =
            TextViewer::new(Rect::new(1, code_top, inner - 1, code_bottom)).with_scrollbars(true);
        viewer.set_text(&code);
        dialog.add(viewer);
    }
    dialog
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
            for part in build.split('~').skip(1).step_by(2) {
                let key = part.chars().next().unwrap_or(' ').to_ascii_lowercase();
                assert!(seen.insert(key), "{}: two ~{key}~ hot keys", demo.name);
            }
        }
    }

    #[test]
    fn demo_commands_stay_in_their_range() {
        // D8: CM_USER + 100 and up belongs to the demos.
        for demo in DEMOS {
            let (_, code) = registry::split_source(demo.source);
            for line in code.lines().filter(|l| l.contains(": CommandId = CM_USER")) {
                let offset: u16 = line
                    .split("CM_USER +")
                    .nth(1)
                    .and_then(|n| n.trim().trim_end_matches(';').parse().ok())
                    .unwrap_or_else(|| panic!("{}: `{line}`", demo.name));
                assert!(
                    offset >= 100,
                    "{}: `{line}` is below CM_USER + 100",
                    demo.name
                );
            }
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

    #[test]
    fn the_panel_builds_for_every_demo() {
        for demo in DEMOS {
            let dialog = panel(demo, Rect::new(LIST_WIDTH, 0, 80, 23));
            assert_eq!(dialog.bounds().width(), 80 - LIST_WIDTH);
        }
    }
}
