//! Two panes with a divider between them: drag the divider to share the
//! room differently. F8 moves the focus to the other pane, and a click
//! focuses the one clicked. Each pane is a `Group`; build it at
//! `first_area()` or `second_area()` and install both with `set_panes`.
//! `grow_first` and `shrink_first` move the divider from your own command.

use crate::panel::Panel;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::state::Grow;
use turbo_vision::views::group::Group;
use turbo_vision::views::listbox::ListBox;
use turbo_vision::views::split_pane::{Orientation, SplitPane};
use turbo_vision::views::text_viewer::TextViewer;
use turbo_vision::views::{GroupLike, View};

pub fn build(panel: &mut Panel) {
    // Vertical: the divider is a column, the panes sit left and right.
    let mut split = SplitPane::new(Rect::new(0, 0, 46, 7), Orientation::Vertical, 16);
    split.set_minimums(8, 12);

    let mut left = Group::new(split.first_area());
    let mut folders = ListBox::new(Rect::new(0, 0, 16, 7), 0);
    folders.set_items(
        ["Inbox", "Sent", "Drafts", "Archive"]
            .map(String::from)
            .to_vec(),
    );
    folders.set_grow_mode(Grow::HI_X | Grow::HI_Y); // follow the divider
    left.add(folders);

    let mut right = Group::new(split.second_area());
    let mut message = TextViewer::new(Rect::new(0, 0, 29, 7));
    message.set_text(
        "Drag the divider to the left\nor right, or press F8 to\nmove between the panes.",
    );
    message.set_grow_mode(Grow::HI_X | Grow::HI_Y);
    right.add(message);

    split.set_panes(left, right);
    split.set_initial_focus();
    panel.add(split);
}
