//! Tabs over pages, each page a `Group` of ordinary controls. F6 or
//! Ctrl+PgDn shows the next tab, Shift+F6 or Ctrl+PgUp the previous one,
//! Alt and a tab's ~letter~ picks it, and a click on a tab works too. The
//! pane keeps F6 while it has the focus: here, click the list to go back.
//! Build each page at `pane.page_area()`; `active()` says which is shown.

use crate::panel::Panel;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::GroupLike;
use turbo_vision::views::checkbox::CheckBox;
use turbo_vision::views::group::Group;
use turbo_vision::views::input_line::InputLine;
use turbo_vision::views::static_text::StaticText;
use turbo_vision::views::tabbed_pane::TabbedPane;

pub fn build(panel: &mut Panel) {
    let mut pane = TabbedPane::new(Rect::new(0, 0, 46, 7));

    // A page's children are placed from the page's own corner.
    let mut general = Group::new(pane.page_area());
    general.add(StaticText::new(Rect::new(1, 0, 8, 1), "Title"));
    let mut title = InputLine::new(Rect::new(8, 0, 40, 1), 40);
    title.set_text("Quarterly report");
    general.add(title);
    general.add(CheckBox::new(Rect::new(1, 2, 24, 3), "Read only"));
    general.set_initial_focus();
    pane.add_page("~G~eneral", general);

    let mut about = Group::new(pane.page_area());
    about.add(StaticText::new(
        Rect::new(1, 0, 42, 3),
        "Each tab shows its own page.\nThe controls on it keep their values\nwhile another tab is in front.",
    ));
    pane.add_page("~A~bout", about);

    pane.set_initial_focus();
    panel.add(pane);
}
