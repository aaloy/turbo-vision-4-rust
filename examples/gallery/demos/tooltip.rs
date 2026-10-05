//! Hints that appear when the pointer rests on a control. One `Tooltip`
//! serves a whole dialog: register a rect and a line of text per control,
//! with `add_hint`. Add it last, so it draws over the controls; clicks
//! still go through it to the control underneath. Rest the pointer on a
//! button below; the hint goes away when you move off, click or type.

use crate::panel::Panel;
use std::time::Duration;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_USER, CommandId};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::button::Button;
use turbo_vision::views::msgbox::message_box_ok;
use turbo_vision::views::tooltip::Tooltip;

const SEND: CommandId = CM_USER + 100;
const DRAFT: CommandId = CM_USER + 101;

pub fn build(panel: &mut Panel) {
    let send = Rect::new(0, 0, 12, 2);
    let draft = Rect::new(14, 0, 32, 2);
    panel.add(Button::new(send, "~S~end", SEND, true));
    panel.add(Button::new(draft, "Save ~d~raft", DRAFT, false));

    // Hint rects are in the tooltip's coordinates; it covers the controls
    // from the same corner, so they are the controls' own rects.
    let mut tips = Tooltip::new(Rect::new(0, 0, 46, 3));
    tips.set_delay(Duration::from_millis(400));
    tips.add_hint(send, "Send the message now");
    tips.add_hint(draft, "Keep it to finish later");
    panel.add(tips);
}

pub fn handle(app: &mut Application, command: CommandId) -> bool {
    let text = match command {
        SEND => "The message was sent.",
        DRAFT => "The draft was saved.",
        _ => return false,
    };
    message_box_ok(app, text);
    true
}
