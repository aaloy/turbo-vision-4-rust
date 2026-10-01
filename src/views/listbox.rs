// (C) 2025 - Enzo Lombardi

//! ListBox view - scrollable list with single or multiple selection.
//!
//! By default one item is focused and that is the selection. Turning on
//! [`ListBox::set_multi_select`] adds a second, independent notion: marked
//! items. Space toggles the mark on the focused item, Shift+click marks a run,
//! and [`ListBox::marked_items`] reports them in order. The focus keeps moving
//! as it always did, so the two never fight.

use super::list_viewer::{ListViewer, ListViewerState};
use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::command::CommandId;
use crate::core::draw::DrawBuffer;
use crate::core::event::{Event, EventType, KB_ENTER, MB_LEFT_BUTTON};
use crate::core::geometry::Rect;
use crate::core::palette::{LISTBOX_FOCUSED, LISTBOX_NORMAL, LISTBOX_SELECTED};
use crate::core::state::State;
use crate::terminal::Terminal;
use std::collections::BTreeSet;

/// Key code for the space bar, which toggles a mark.
const KB_SPACE: u16 = b' ' as u16;

/// Drawn before a marked item when multi-select is on.
const MARK_ON: &str = "\u{221A} ";
/// Drawn before an unmarked item when multi-select is on, to keep the columns
/// lined up.
const MARK_OFF: &str = "  ";

/// A source of items a [`ListBox`] reads only as it draws them.
///
/// `item` is only asked for indices below `len()`. A provider whose length
/// changes must be followed by [`ListBox::refresh_items`].
pub trait ListProvider {
    /// How many items there are.
    fn len(&self) -> usize;
    /// The text of one item.
    fn item(&self, index: usize) -> String;
    /// Whether there are no items.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Where a list box's items come from.
enum Items {
    Owned(Vec<String>),
    Provided(Box<dyn ListProvider>),
}

impl Items {
    fn len(&self) -> usize {
        match self {
            Items::Owned(items) => items.len(),
            Items::Provided(p) => p.len(),
        }
    }

    fn get(&self, index: usize) -> Option<String> {
        match self {
            Items::Owned(items) => items.get(index).cloned(),
            Items::Provided(p) => (index < p.len()).then(|| p.item(index)),
        }
    }
}

/// ListBox - A scrollable list of selectable items
///
/// Now implements ListViewer trait for standard navigation behavior.
/// Matches Borland: TListBox (extends TListViewer)
pub struct ListBox {
    core: ViewCore,
    items: Items,
    list_state: ListViewerState, // Embedded state from ListViewer
    on_select_command: CommandId,
    /// Whether items can be marked independently of the focus.
    multi_select: bool,
    /// Marked item indices, kept ordered so callers read them in list order.
    marked: BTreeSet<usize>,
    /// Where the last Shift+click run started.
    anchor: usize,
}

impl ListBox {
    /// Create a new list box
    pub fn new(bounds: Rect, on_select_command: CommandId) -> Self {
        Self {
            core: ViewCore {
                bounds,
                state: State::empty(),
                palette_chain: None,
                ..ViewCore::default()
            },
            items: Items::Owned(Vec::new()),
            list_state: ListViewerState::new(),
            on_select_command,
            multi_select: false,
            marked: BTreeSet::new(),
            anchor: 0,
        }
    }

    /// Set the items in the list.
    ///
    /// Marks are dropped, because their indices refer to the old list.
    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = Items::Owned(items);
        self.list_state.set_range(self.items.len());
        self.marked.clear();
        self.anchor = 0;
    }

    /// Append one item. On a provider-backed list this starts a new
    /// in-memory list holding just this item.
    pub fn add_item(&mut self, item: String) {
        match &mut self.items {
            Items::Owned(items) => items.push(item),
            Items::Provided(_) => {
                self.items = Items::Owned(vec![item]);
                self.marked.clear();
            }
        }
        self.list_state.set_range(self.items.len());
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.items = Items::Owned(Vec::new());
        self.list_state.set_range(0);
        self.marked.clear();
        self.anchor = 0;
    }

    /// Read items from `provider` instead of an in-memory list. Marks are
    /// dropped, because their indices refer to the old items; the focus is
    /// clamped to the new length.
    pub fn set_provider(&mut self, provider: Box<dyn ListProvider>) {
        self.items = Items::Provided(provider);
        self.marked.clear();
        self.refresh_items();
    }

    /// Re-read the item count after a provider's source changed length.
    /// Marks past the new end are dropped.
    pub fn refresh_items(&mut self) {
        let len = self.items.len();
        self.marked.retain(|&i| i < len);
        self.list_state.set_range(len);
    }

    /// Allow items to be marked independently of the focus.
    ///
    /// Turning it off drops the marks, since nothing can act on them any more.
    pub fn set_multi_select(&mut self, multi: bool) {
        self.multi_select = multi;
        if !multi {
            self.marked.clear();
        }
    }

    /// Whether items can be marked.
    pub fn is_multi_select(&self) -> bool {
        self.multi_select
    }

    /// Whether one item is marked.
    pub fn is_marked(&self, index: usize) -> bool {
        self.marked.contains(&index)
    }

    /// The marked items, in list order.
    pub fn marked_items(&self) -> Vec<usize> {
        self.marked.iter().copied().collect()
    }

    /// The marked items' text, in list order.
    pub fn marked_text(&self) -> Vec<String> {
        self.marked
            .iter()
            .filter_map(|&i| self.items.get(i))
            .collect()
    }

    /// How many items are marked.
    pub fn marked_count(&self) -> usize {
        self.marked.len()
    }

    /// Mark or unmark one item. Out-of-range indices are ignored.
    ///
    /// Works whether or not multi-select is on, so a caller can pre-mark a list
    /// before showing it.
    pub fn set_marked(&mut self, index: usize, marked: bool) {
        if index >= self.items.len() {
            return;
        }
        if marked {
            self.marked.insert(index);
        } else {
            self.marked.remove(&index);
        }
    }

    /// Flip one item's mark.
    pub fn toggle_mark(&mut self, index: usize) {
        let marked = self.is_marked(index);
        self.set_marked(index, !marked);
    }

    /// Mark every item.
    pub fn mark_all(&mut self) {
        self.marked = (0..self.items.len()).collect();
    }

    /// Unmark everything.
    pub fn clear_marks(&mut self) {
        self.marked.clear();
    }

    /// Mark every item between the anchor and `index`, inclusive, leaving marks
    /// outside that run alone.
    fn mark_run_to(&mut self, index: usize) {
        let (lo, hi) = if self.anchor <= index {
            (self.anchor, index)
        } else {
            (index, self.anchor)
        };
        for i in lo..=hi.min(self.items.len().saturating_sub(1)) {
            self.marked.insert(i);
        }
    }

    /// Get the currently selected item index
    pub fn get_selection(&self) -> Option<usize> {
        self.list_state.focused
    }

    /// Get the currently selected item text
    pub fn get_selected_item(&self) -> Option<String> {
        self.items.get(self.list_state.focused?)
    }

    /// Set the selected item by index
    pub fn set_selection(&mut self, index: usize) {
        if index < self.items.len() {
            let visible_rows = self.core.bounds.height_clamped() as usize;
            self.list_state.focus_item(index, visible_rows);
        }
    }

    /// Get the number of items
    ///
    /// Cached from the item source at the last `set_items`, `set_provider`
    /// or `refresh_items`.
    pub fn item_count(&self) -> usize {
        self.list_state.range
    }

    // Convenience methods for compatibility with existing code
    // These delegate to ListViewerState methods

    /// Move selection up (convenience method)
    pub fn select_prev(&mut self) {
        let visible_rows = self.core.bounds.height_clamped() as usize;
        self.list_state.focus_prev(visible_rows);
    }

    /// Move selection down (convenience method)
    pub fn select_next(&mut self) {
        let visible_rows = self.core.bounds.height_clamped() as usize;
        self.list_state.focus_next(visible_rows);
    }

    /// Select first item (convenience method)
    pub fn select_first(&mut self) {
        let visible_rows = self.core.bounds.height_clamped() as usize;
        self.list_state.focus_first(visible_rows);
    }

    /// Select last item (convenience method)
    pub fn select_last(&mut self) {
        let visible_rows = self.core.bounds.height_clamped() as usize;
        self.list_state.focus_last(visible_rows);
    }

    /// Page up (convenience method)
    pub fn page_up(&mut self) {
        let visible_rows = self.core.bounds.height_clamped() as usize;
        self.list_state.focus_page_up(visible_rows);
    }

    /// Page down (convenience method)
    pub fn page_down(&mut self) {
        let visible_rows = self.core.bounds.height_clamped() as usize;
        self.list_state.focus_page_down(visible_rows);
    }
}

impl View for ListBox {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.core.bounds.width_clamped() as usize;
        let height = self.core.bounds.height_clamped() as usize;

        // ListBox palette indices:
        // 1: Normal, 2: Focused, 3: Selected, 4: Divider
        let color_normal = if self.is_focused() {
            self.map_color(LISTBOX_FOCUSED) // Focused
        } else {
            self.map_color(LISTBOX_NORMAL) // Normal
        };
        let color_selected = self.map_color(LISTBOX_SELECTED); // Selected

        // Draw visible items
        for i in 0..height {
            let mut buf = DrawBuffer::new(width);
            let item_idx = self.list_state.top_item + i;

            if item_idx < self.items.len() {
                let is_focused_row = Some(item_idx) == self.list_state.focused;
                let color = if is_focused_row {
                    color_selected
                } else {
                    color_normal
                };

                buf.move_char(0, ' ', color, width);

                // With multi-select on, every row carries a two-cell mark
                // column so the text stays aligned whether marked or not.
                let text_at = if self.multi_select {
                    let mark = if self.is_marked(item_idx) {
                        MARK_ON
                    } else {
                        MARK_OFF
                    };
                    buf.move_str(0, mark, color);
                    MARK_OFF.len()
                } else {
                    0
                };

                if text_at < width {
                    let room = width - text_at;
                    let text: String = self
                        .items
                        .get(item_idx)
                        .unwrap_or_default()
                        .chars()
                        .take(room)
                        .collect();
                    buf.move_str(text_at, &text, color);
                }
            } else {
                // Empty line
                buf.move_char(0, ' ', color_normal, width);
            }

            write_line_to_terminal(terminal, 0, i as i16, &buf);
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Handle double-click BEFORE handle_list_event consumes it
        // This ensures double-click triggers the command even though single-click is handled
        if event.what == EventType::MouseDown {
            let mouse_pos = event.mouse.pos;

            // Check if click is within the listbox bounds
            if self.extent().contains(mouse_pos) && event.mouse.buttons & MB_LEFT_BUTTON != 0 {
                // Double-click triggers selection command (matching Borland's TListViewer)
                if event.mouse.double_click {
                    // CRITICAL: Update selection to the double-clicked item BEFORE converting to command
                    // Without this, the selection would still point to the previously selected item,
                    // causing FileDialog to act on the wrong file/directory (e.g., double-clicking a
                    // folder would close the dialog instead of navigating into it)
                    let relative_y = (mouse_pos.y) as usize;
                    let clicked_item = self.list_state.top_item + relative_y;

                    // Update the selection to the double-clicked item
                    if clicked_item < self.items.len() {
                        let visible_rows = self.core.bounds.height_clamped() as usize;
                        self.list_state.focus_item(clicked_item, visible_rows);
                    }

                    // Now convert to command with the correct item selected
                    *event = Event::command(self.on_select_command);
                    return;
                }
            }
        }

        // Multi-select clicks: a plain click sets the anchor, a Shift+click
        // marks the run from it. Handled before the shared list navigation so
        // the modifier is not lost.
        if self.multi_select
            && event.what == EventType::MouseDown
            && event.mouse.buttons & MB_LEFT_BUTTON != 0
            && self.extent().contains(event.mouse.pos)
        {
            let relative_y = (event.mouse.pos.y) as usize;
            let clicked = self.list_state.top_item + relative_y;
            if clicked < self.items.len() {
                if event
                    .key_modifiers
                    .contains(crate::core::keys::KeyModifiers::SHIFT)
                {
                    self.mark_run_to(clicked);
                } else {
                    self.anchor = clicked;
                }
            }
        }

        // First try standard list navigation (from ListViewer trait)
        // This handles single-click, arrow keys, etc.
        if self.handle_list_event(event) {
            return;
        }

        // Handle ListBox-specific events
        match event.what {
            EventType::Keyboard => {
                if event.key_code == KB_ENTER {
                    // Enter on selected item generates command
                    *event = Event::command(self.on_select_command);
                } else if self.multi_select && event.key_code == KB_SPACE {
                    // Space marks the focused item and re-anchors, so a
                    // following Shift+click extends from here.
                    if let Some(focused) = self.list_state.focused {
                        self.toggle_mark(focused);
                        self.anchor = focused;
                    }
                    event.clear();
                }
            }
            EventType::MouseDown => {
                // Single click is already handled by handle_list_event above
                // This code path is for any MouseDown events that weren't handled
            }
            EventType::MouseWheelUp => {
                let mouse_pos = event.mouse.pos;
                if self.extent().contains(mouse_pos) {
                    self.select_prev();
                    event.clear();
                }
            }
            EventType::MouseWheelDown => {
                let mouse_pos = event.mouse.pos;
                if self.extent().contains(mouse_pos) {
                    self.select_next();
                    event.clear();
                }
            }
            _ => {}
        }
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_LISTBOX))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// Implement ListViewer trait
impl ListViewer for ListBox {
    fn list_state(&self) -> &ListViewerState {
        &self.list_state
    }

    fn list_state_mut(&mut self) -> &mut ListViewerState {
        &mut self.list_state
    }

    fn get_text(&self, item: usize, _max_len: usize) -> String {
        self.items.get(item).unwrap_or_default()
    }

    /// With multi-select on, "selected" means marked; otherwise it keeps the
    /// default meaning of "focused".
    fn is_selected(&self, item: usize) -> bool {
        if self.multi_select {
            self.is_marked(item)
        } else {
            Some(item) == self.list_state.focused
        }
    }
}

/// Builder for creating listboxes with a fluent API.
pub struct ListBoxBuilder {
    bounds: Option<Rect>,
    on_select_command: CommandId,
}

impl ListBoxBuilder {
    pub fn new() -> Self {
        Self {
            bounds: None,
            on_select_command: 0,
        }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn on_select_command(mut self, command: CommandId) -> Self {
        self.on_select_command = command;
        self
    }

    pub fn build(self) -> ListBox {
        let bounds = self.bounds.expect("ListBox bounds must be set");
        ListBox::new(bounds, self.on_select_command)
    }

    pub fn build_boxed(self) -> Box<ListBox> {
        Box::new(self.build())
    }
}

impl Default for ListBoxBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listbox_creation() {
        let listbox = ListBox::new(Rect::new(0, 0, 20, 10), 1000);
        assert_eq!(listbox.item_count(), 0);
        assert_eq!(listbox.get_selection(), None);
    }

    #[test]
    fn test_listbox_add_items() {
        let mut listbox = ListBox::new(Rect::new(0, 0, 20, 10), 1000);
        listbox.add_item("Item 1".to_string());
        listbox.add_item("Item 2".to_string());
        listbox.add_item("Item 3".to_string());

        assert_eq!(listbox.item_count(), 3);
        assert_eq!(listbox.get_selection(), Some(0));
        assert_eq!(listbox.get_selected_item().as_deref(), Some("Item 1"));
    }

    #[test]
    fn test_listbox_set_items() {
        let mut listbox = ListBox::new(Rect::new(0, 0, 20, 10), 1000);
        let items = vec!["Alpha".to_string(), "Beta".to_string(), "Gamma".to_string()];
        listbox.set_items(items);

        assert_eq!(listbox.item_count(), 3);
        assert_eq!(listbox.get_selection(), Some(0));
    }

    #[test]
    fn test_listbox_navigation() {
        let mut listbox = ListBox::new(Rect::new(0, 0, 20, 10), 1000);
        listbox.set_items(vec![
            "Item 1".to_string(),
            "Item 2".to_string(),
            "Item 3".to_string(),
        ]);

        assert_eq!(listbox.get_selection(), Some(0));

        listbox.select_next();
        assert_eq!(listbox.get_selection(), Some(1));

        listbox.select_next();
        assert_eq!(listbox.get_selection(), Some(2));

        listbox.select_next(); // Should stay at 2 (last item)
        assert_eq!(listbox.get_selection(), Some(2));

        listbox.select_prev();
        assert_eq!(listbox.get_selection(), Some(1));

        listbox.select_first();
        assert_eq!(listbox.get_selection(), Some(0));

        listbox.select_last();
        assert_eq!(listbox.get_selection(), Some(2));
    }

    #[test]
    fn test_listbox_set_selection() {
        let mut listbox = ListBox::new(Rect::new(0, 0, 20, 10), 1000);
        listbox.set_items(vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
        ]);

        listbox.set_selection(2);
        assert_eq!(listbox.get_selection(), Some(2));
        assert_eq!(listbox.get_selected_item().as_deref(), Some("C"));

        listbox.set_selection(10); // Out of bounds, should be ignored
        assert_eq!(listbox.get_selection(), Some(2)); // Should not change
    }

    #[test]
    fn test_listbox_clear() {
        let mut listbox = ListBox::new(Rect::new(0, 0, 20, 10), 1000);
        listbox.set_items(vec!["Item 1".to_string(), "Item 2".to_string()]);
        assert_eq!(listbox.item_count(), 2);

        listbox.clear();
        assert_eq!(listbox.item_count(), 0);
        assert_eq!(listbox.get_selection(), None);
    }

    // --- Multi-select ---------------------------------------------------

    fn multi_list() -> ListBox {
        let mut lb = ListBox::new(Rect::new(0, 0, 20, 5), 1000);
        lb.set_items((0..8).map(|i| format!("item{i}")).collect());
        lb.set_multi_select(true);
        lb.set_state(crate::core::state::State::FOCUSED);
        lb
    }

    fn space(lb: &mut ListBox) {
        let mut e = Event::keyboard(KB_SPACE);
        lb.handle_event(&mut e);
    }

    fn shift_click(lb: &mut ListBox, row: i16) {
        let mut e = Event::nothing();
        e.what = EventType::MouseDown;
        e.mouse.buttons = MB_LEFT_BUTTON;
        e.mouse.pos = crate::core::geometry::Point::new(1, row);
        e.key_modifiers = crate::core::keys::KeyModifiers::SHIFT;
        lb.handle_event(&mut e);
    }

    #[test]
    fn multi_select_is_off_by_default() {
        let lb = ListBox::new(Rect::new(0, 0, 20, 5), 1000);
        assert!(!lb.is_multi_select());
    }

    #[test]
    fn space_toggles_the_focused_mark() {
        let mut lb = multi_list();
        lb.set_selection(2);
        space(&mut lb);
        assert!(lb.is_marked(2));
        assert_eq!(lb.marked_items(), vec![2]);
        space(&mut lb);
        assert!(!lb.is_marked(2));
        assert_eq!(lb.marked_count(), 0);
    }

    #[test]
    fn space_does_nothing_in_single_select_mode() {
        let mut lb = multi_list();
        lb.set_multi_select(false);
        lb.set_selection(2);
        space(&mut lb);
        assert_eq!(lb.marked_count(), 0);
    }

    #[test]
    fn marks_come_back_in_list_order() {
        let mut lb = multi_list();
        for i in [5, 1, 3] {
            lb.set_marked(i, true);
        }
        assert_eq!(lb.marked_items(), vec![1, 3, 5]);
        assert_eq!(lb.marked_text(), vec!["item1", "item3", "item5"]);
    }

    #[test]
    fn marking_out_of_range_is_ignored() {
        let mut lb = multi_list();
        lb.set_marked(99, true);
        assert_eq!(lb.marked_count(), 0);
    }

    #[test]
    fn shift_click_marks_the_run_from_the_anchor() {
        let mut lb = multi_list();
        lb.set_selection(1);
        space(&mut lb); // marks 1 and anchors there
        shift_click(&mut lb, 4); // row 4 is item 4, the list is not scrolled
        assert_eq!(lb.marked_items(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn shift_click_backwards_marks_the_same_run() {
        let mut lb = multi_list();
        lb.set_selection(4);
        space(&mut lb);
        shift_click(&mut lb, 1);
        assert_eq!(lb.marked_items(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn shift_click_leaves_marks_outside_the_run_alone() {
        let mut lb = multi_list();
        lb.set_marked(7, true);
        lb.set_selection(1);
        space(&mut lb);
        shift_click(&mut lb, 3);
        assert_eq!(lb.marked_items(), vec![1, 2, 3, 7]);
    }

    #[test]
    fn mark_all_and_clear_marks() {
        let mut lb = multi_list();
        lb.mark_all();
        assert_eq!(lb.marked_count(), 8);
        lb.clear_marks();
        assert_eq!(lb.marked_count(), 0);
    }

    #[test]
    fn replacing_the_items_drops_stale_marks() {
        let mut lb = multi_list();
        lb.mark_all();
        lb.set_items(vec!["one".into(), "two".into()]);
        assert_eq!(
            lb.marked_count(),
            0,
            "old indices mean nothing in the new list"
        );
    }

    #[test]
    fn turning_multi_select_off_drops_the_marks() {
        let mut lb = multi_list();
        lb.mark_all();
        lb.set_multi_select(false);
        assert_eq!(lb.marked_count(), 0);
    }

    #[test]
    fn is_selected_follows_the_marks_only_in_multi_select() {
        let mut lb = multi_list();
        lb.set_selection(0);
        lb.set_marked(3, true);
        assert!(lb.is_selected(3), "marked");
        assert!(!lb.is_selected(0), "focused but not marked");

        lb.set_multi_select(false);
        assert!(lb.is_selected(0), "back to meaning focused");
    }

    #[test]
    fn the_focus_still_moves_independently_of_the_marks() {
        let mut lb = multi_list();
        lb.set_selection(2);
        space(&mut lb);
        lb.select_next();
        assert_eq!(lb.get_selection(), Some(3));
        assert_eq!(lb.marked_items(), vec![2], "the mark stayed put");
    }

    // --- Lazy item source -------------------------------------------------

    struct Numbers(usize);

    impl ListProvider for Numbers {
        fn len(&self) -> usize {
            self.0
        }
        fn item(&self, index: usize) -> String {
            format!("Item {index}")
        }
    }

    #[test]
    fn a_provider_supplies_items() {
        let mut lb = ListBox::new(Rect::new(0, 0, 20, 5), 0);
        lb.set_provider(Box::new(Numbers(1_000_000)));
        assert_eq!(lb.item_count(), 1_000_000);
        lb.set_selection(999_999);
        assert_eq!(lb.get_selected_item().as_deref(), Some("Item 999999"));

        let mut term = crate::test_util::test_terminal(20, 5);
        lb.draw(&mut term);
        let shown = (0..5).any(|y| {
            let line: String = (0..20).map(|x| term.read_cell(x, y).unwrap().ch).collect();
            line.trim_end() == "Item 999999"
        });
        assert!(shown);
    }

    #[test]
    fn a_shorter_provider_clamps_the_selection_and_drops_marks() {
        let mut lb = ListBox::new(Rect::new(0, 0, 20, 5), 0);
        lb.set_provider(Box::new(Numbers(50)));
        lb.set_selection(49);
        lb.set_marked(10, true);
        lb.set_provider(Box::new(Numbers(3)));
        assert_eq!(lb.get_selection(), Some(2));
        assert_eq!(lb.marked_count(), 0);
        let mut term = crate::test_util::test_terminal(20, 5);
        lb.draw(&mut term);
    }

    #[test]
    fn marked_text_reads_through_a_provider() {
        let mut lb = ListBox::new(Rect::new(0, 0, 20, 5), 0);
        lb.set_provider(Box::new(Numbers(10)));
        lb.set_marked(2, true);
        lb.set_marked(7, true);
        assert_eq!(lb.marked_text(), vec!["Item 2", "Item 7"]);
    }

    #[test]
    fn add_item_on_a_provider_starts_a_new_list() {
        let mut lb = ListBox::new(Rect::new(0, 0, 20, 5), 0);
        lb.set_provider(Box::new(Numbers(10)));
        lb.add_item("solo".into());
        assert_eq!(lb.item_count(), 1);
        assert_eq!(lb.get_selected_item().as_deref(), Some("solo"));
    }
}
