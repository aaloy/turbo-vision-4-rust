//! Fixed text, one or more lines: captions, notes, help. Neither StaticText
//! nor ParamText takes the focus; a Label is the caption that focuses a
//! field.
//!
//! Parameters:
//! - `StaticText::new(bounds, text)`: breaks at each `\n`; `bounds` must be
//!   as tall as the lines. `new_centered(bounds, text)` centres each line.
//! - `ParamText::new(bounds, template)`: text with placeholders, %s for
//!   text and %d for numbers; `set_params(texts, numbers)` fills them in,
//!   for text that changes, such as a count or a file name.
//! - `Label::new(bounds, text)`: a caption with a ~letter~; `set_link(id)`
//!   names the view that Alt and the letter, or a click, focuses.
//!   `set_error(true)` shows it in the error colour.
//!
//! See also: InputLine, GroupBox, Form

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
