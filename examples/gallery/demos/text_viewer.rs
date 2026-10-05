//! Read-only text that scrolls: a log, a report, a file. The arrows, PgUp,
//! PgDn, Home and End scroll it, and so do its scroll bars.
//!
//! Parameters:
//! - `TextViewer::new(bounds)`: `.with_scrollbars(true)` puts scroll bars
//!   inside those bounds; `.with_indicator(true)` a line:column row on top.
//! - `set_text(text)`, `load_file(path)`: what it shows.
//! - `set_show_line_numbers(true)`: numbers the lines.
//! - `set_grow_mode(Grow::ALL)`: in a window, it follows the window's size.
//!
//! See also: Editor, Memo, StaticText

use crate::panel::Panel;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::text_viewer::TextViewer;

pub fn build(panel: &mut Panel) {
    let mut log = TextViewer::new(Rect::new(0, 0, 46, 6)).with_scrollbars(true);
    log.set_show_line_numbers(true);
    let lines: Vec<String> = (1..=40)
        .map(|n| {
            format!(
                "12:{:02}:{:02} backup: copied file {n} of 40",
                n / 6,
                (n * 7) % 60
            )
        })
        .collect();
    log.set_text(&lines.join("\n"));
    panel.add(log);
}
