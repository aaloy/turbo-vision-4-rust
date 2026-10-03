//! Builds a dialog from labelled fields, with no coordinates: labels in one
//! column, fields in the next, buttons at the bottom, the dialog sized and
//! centred. Lines put fields side by side, groups box related rows. For a
//! struct of yours, `Form::<T>::for_record` binds and validates the fields.
//! Full guide: docs/FORMS.md.

use crate::panel::Panel;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_OK, CM_USER, CommandId};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::GroupLike;
use turbo_vision::views::button::Button;
use turbo_vision::views::checkbox::CheckBox;
use turbo_vision::views::form::{Form, size};
use turbo_vision::views::input_line::InputLine;
use turbo_vision::views::msgbox::message_box_ok;

const OPEN: CommandId = CM_USER + 100;

pub fn build(panel: &mut Panel) {
    panel.add(Button::new(
        Rect::new(0, 0, 18, 2),
        "~O~pen a form",
        OPEN,
        true,
    ));
}

pub fn handle(app: &mut Application, command: CommandId) -> bool {
    if command != OPEN {
        return false;
    }
    let mut form = Form::new("New Customer");
    let name = form.field("~N~ame", InputLine::new(Rect::default(), 40));
    form.group("Address");
    form.field("~S~treet", InputLine::new(Rect::default(), 40));
    let mut line = form.line();
    line.field("~C~ity", InputLine::new(Rect::default(), 30));
    line.field("~Z~IP", InputLine::new(size(8, 1), 8));
    form.end_group();
    form.field("", CheckBox::new(Rect::default(), "~V~IP customer"));
    form.ok_cancel();

    let mut dialog = form.build();
    if dialog.execute(app) == CM_OK {
        let name = dialog
            .get(name)
            .map(|f| f.text().to_string())
            .unwrap_or_default();
        message_box_ok(app, &format!("Saved: {name}"));
    }
    true
}
