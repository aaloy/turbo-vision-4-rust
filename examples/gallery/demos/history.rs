//! Earlier entries for an input line. The `History` button (the arrow on
//! the right) is linked to its input by a handle; click it to pick an
//! entry, which is copied into the input. Lists are kept per history id,
//! so inputs with the same id share one. A dialog closed with OK records
//! what was typed; `HistoryManager::add` adds an entry yourself.

use crate::panel::Panel;
use turbo_vision::core::geometry::{Point, Rect};
use turbo_vision::core::history::HistoryManager;
use turbo_vision::views::history::History;
use turbo_vision::views::input_line::InputLine;
use turbo_vision::views::label::Label;

/// The list of searches, shared by every input that uses this id.
const SEARCHES: u16 = 100;

pub fn build(panel: &mut Panel) {
    for earlier in ["invoices 2025", "unpaid", "Ada Lovelace"] {
        HistoryManager::add(SEARCHES, earlier.to_string());
    }

    let search = panel.add_typed(InputLine::new(Rect::new(9, 0, 37, 1), 60));
    panel.add(History::new(Point::new(37, 0), SEARCHES, search));
    let mut label = Label::new(Rect::new(0, 0, 8, 1), "~S~earch");
    label.set_link(search.id());
    panel.add(label);
}
