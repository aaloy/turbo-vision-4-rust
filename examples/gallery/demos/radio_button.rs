//! One radio button per view, for choices laid out freely rather than in a
//! column. Buttons with the same group id are one choice: selecting one
//! clears the others in its group. Space or a click selects; Tab moves
//! between them.
//!
//! Parameters:
//! - `RadioButton::new(bounds, label, group_id)`: one row; `group_id` ties
//!   the buttons of one choice together, so each choice needs its own id.
//! - `set_selected(true)`: the button chosen to start with.
//! - `is_selected()`: read each button; `select()` and `deselect()` set it
//!   from your code.
//!
//! See also: RadioButtons, CheckBoxes, ComboBox

use crate::panel::Panel;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::radiobutton::RadioButton;

/// Each group is one choice.
const PAPER: u16 = 1;
const COLOUR: u16 = 2;

pub fn build(panel: &mut Panel) {
    let mut a4 = RadioButton::new(Rect::new(0, 0, 10, 1), "A4", PAPER);
    a4.set_selected(true);
    panel.add(a4);
    panel.add(RadioButton::new(Rect::new(12, 0, 24, 1), "Letter", PAPER));
    panel.add(RadioButton::new(Rect::new(26, 0, 38, 1), "Legal", PAPER));

    let mut mono = RadioButton::new(Rect::new(0, 2, 10, 3), "Mono", COLOUR);
    mono.set_selected(true);
    panel.add(mono);
    panel.add(RadioButton::new(Rect::new(12, 2, 24, 3), "Colour", COLOUR));
}
