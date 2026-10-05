//! A text editor: undo and redo (Ctrl+Z, Ctrl+Y), selection with Shift, the
//! clipboard, search and replace, and syntax colours. Tab is typed into the
//! text, so Alt+O reaches the button: it opens an EditWindow, an editor in
//! a window with scroll bars and line:column.
//!
//! Parameters:
//! - `EditorWindow::new(bounds)`: the bare editor. `with_scrollbars(bounds,
//!   h_bar, v_bar, indicator)` links it to scroll bars and a line:column
//!   indicator that you place yourself.
//! - `set_text(text)`, `get_text()`; `load_file(path)`, `save_file()`,
//!   `save_as(path)`; `is_modified()` says whether the text changed.
//! - `set_highlighter(Box::new(RustHighlighter::new()))`: syntax colours,
//!   from any `SyntaxHighlighter`.
//! - `set_read_only(true)`: the text can be read and selected, not changed.
//!   This gallery's code view is one.
//! - `set_tab_size(n)`, `set_auto_indent(on)`: how Tab and Enter indent.
//! - `undo()`, `redo()`, `find(text, options)`, `replace_all(...)`: the
//!   same, from your own commands.
//! - `EditWindow::new(bounds, title)`: the editor in a window;
//!   `editor_mut()` reaches the editor inside.
//!
//! See also: Memo, TextViewer, File dialogs

use crate::panel::Panel;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_USER, CommandId};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::button::Button;
use turbo_vision::views::edit_window::EditWindow;
use turbo_vision::views::editor::EditorWindow;
use turbo_vision::views::syntax::RustHighlighter;

const OPEN: CommandId = CM_USER + 100;

pub fn build(panel: &mut Panel) {
    let mut editor = EditorWindow::new(Rect::new(0, 0, 46, 4));
    editor.set_highlighter(Box::new(RustHighlighter::new()));
    editor.set_text(SAMPLE);
    panel.add(editor);
    panel.add(Button::new(
        Rect::new(0, 5, 24, 7),
        "~O~pen an edit window",
        OPEN,
        false,
    ));
}

pub fn handle(app: &mut Application, command: CommandId) -> bool {
    if command != OPEN {
        return false;
    }
    let window = EditWindow::new(Rect::new(26, 2, 78, 18), "Untitled.rs");
    window
        .editor_mut()
        .set_highlighter(Box::new(RustHighlighter::new()));
    window.editor_mut().set_text(SAMPLE);
    app.desktop.add(window);
    true
}

const SAMPLE: &str =
    "// Edit me: Ctrl+Z undoes.\nfn main() {\n    let answer = 42;\n    println!(\"{answer}\");\n}";
