//! Fixed text, one or more lines: captions, help, notes. `StaticText`
//! breaks at `\n`; `new_centered` centres each line. `ParamText` is the
//! same with placeholders, %s for text and %d for numbers, filled in with
//! `set_params`, for text that changes: a count, a file name. Neither takes
//! the focus; a `Label` is the caption that focuses a field.

use crate::panel::Panel;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::input_line::InputLine;
use turbo_vision::views::label::Label;
use turbo_vision::views::paramtext::ParamText;
use turbo_vision::views::static_text::StaticText;

pub fn build(panel: &mut Panel) {
    panel.add(StaticText::new(
        Rect::new(0, 0, 46, 2),
        "Static text breaks at each newline\nand never takes the focus.",
    ));
    panel.add(StaticText::new_centered(
        Rect::new(0, 2, 46, 3),
        "A centred line",
    ));

    let mut status = ParamText::new(Rect::new(0, 3, 46, 4), "%d files in %s");
    status.set_params(&["/home/ada"], &[12]);
    panel.add(status);

    // A label's ~letter~ focuses the field it is linked to.
    let field = panel.add(InputLine::new(Rect::new(10, 5, 40, 6), 40));
    let mut label = Label::new(Rect::new(0, 5, 9, 6), "~F~ield");
    label.set_link(field);
    panel.add(label);
}
