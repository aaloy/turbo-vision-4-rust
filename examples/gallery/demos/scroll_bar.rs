//! A bar that shows where a view is scrolled, or holds a value of its own.
//! Linked to a view, it moves as the view scrolls (here, the editor's
//! cursor). On its own, the mouse sets it: click the arrows, the track on
//! either side of the thumb, or drag the thumb. A bar never takes the
//! focus, and it sends nothing when the user moves it, so a view in a
//! dialog does not follow its bar; an EditWindow does, because it reads its
//! own bars back.
//!
//! Parameters:
//! - `ScrollBar::new_vertical(bounds)`, `new_horizontal(bounds)`: one
//!   column tall, or one row wide, with an arrow at each end.
//! - `set_params(value, min, max, page, arrow)`: the value, its range, how
//!   far a click on the track moves it (`page`) and how far an arrow does.
//! - `set_value(v)`, `set_range(min, max)`, `get_value()`: the same, one at
//!   a time; the value is kept within the range. Read it when you need it.
//! - `set_total(n)`: the size of what is scrolled, in lines or columns,
//!   which sizes the thumb to the part shown. A bar with no total draws no
//!   thumb, so a bar on its own needs one too; any total larger than the
//!   track gives a one-cell thumb.
//! - Linking: wrap the bar in `Rc<RefCell<..>>`, give a clone to the view
//!   (`EditorWindow::with_scrollbars(bounds, h_bar, v_bar, indicator)`),
//!   and add `Shared::new(bar)` to the dialog, so the dialog draws the bar
//!   the view moves. `TextViewer` and `Memo` make their own with
//!   `.with_scrollbars(true)`.
//!
//! See also: TextViewer, Editor, Slider

use crate::panel::Panel;
use std::cell::RefCell;
use std::rc::Rc;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::editor::EditorWindow;
use turbo_vision::views::scrollbar::ScrollBar;
use turbo_vision::views::shared::Shared;
use turbo_vision::views::static_text::StaticText;

pub fn build(panel: &mut Panel) {
    // Two bars linked to an editor: it moves them as its cursor moves.
    let v_bar = Rc::new(RefCell::new(ScrollBar::new_vertical(Rect::new(
        29, 0, 30, 5,
    ))));
    let h_bar = Rc::new(RefCell::new(ScrollBar::new_horizontal(Rect::new(
        0, 5, 29, 6,
    ))));
    let mut editor = EditorWindow::with_scrollbars(
        Rect::new(0, 0, 29, 5),
        Some(Rc::clone(&h_bar)),
        Some(Rc::clone(&v_bar)),
        None,
    );
    let lines: Vec<String> = (1..=30)
        .map(|n| format!("Line {n}: scroll me, or drag the bars on my edges"))
        .collect();
    editor.set_text(&lines.join("\n"));
    panel.add(editor);
    panel.add(Shared::new(v_bar));
    panel.add(Shared::new(h_bar));

    // A bar on its own: a value from 0 to 100, ten a click on the track.
    panel.add(StaticText::new(Rect::new(32, 1, 46, 2), "On its own:"));
    let mut level = ScrollBar::new_horizontal(Rect::new(32, 3, 46, 4));
    level.set_params(40, 0, 100, 10, 1);
    level.set_total(100); // a one-cell thumb
    panel.add(level);
}
