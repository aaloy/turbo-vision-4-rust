// (C) 2026 - Enzo Lombardi

//! Slider view - a horizontal track with a thumb that picks one integer.
//!
//! Not part of the Borland Turbo Vision widget set; the control comes from the
//! TV Tool Box add-on library. It is the dragging counterpart of `Spinner`.
//!
//! # Keys
//!
//! | Key | Action |
//! |-----|--------|
//! | Left, Right | One step down or up |
//! | Home, End | Minimum or maximum |
//!
//! Clicking or dragging on the track moves the thumb under the mouse.

use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::command::CommandId;
use crate::core::draw::DrawBuffer;
use crate::core::event::{Event, EventType, KB_END, KB_HOME, KB_LEFT, KB_RIGHT, MB_LEFT_BUTTON};
use crate::core::geometry::Rect;
use crate::core::palette::{INPUT_ARROWS, INPUT_NORMAL, INPUT_SELECTED};
use crate::core::state::{State, StateFlags};
use crate::terminal::Terminal;

/// The track character.
pub const TRACK: char = '─';
/// The thumb character. A CP437 glyph, so it also shows in PNG captures.
pub const THUMB: char = '■';

/// A horizontal value slider over `min..=max`.
pub struct Slider {
    core: ViewCore,
    view_state: StateFlags,
    min: i64,
    max: i64,
    value: i64,
    step: i64,
    /// Broadcast on every user change; 0 sends nothing.
    on_change: CommandId,
    /// True while the user is dragging the thumb.
    dragging: bool,
}

impl Slider {
    /// A slider over `min..=max`, starting at `min`. Reversed bounds are
    /// swapped.
    pub fn new(bounds: Rect, min: i64, max: i64) -> Self {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        Self {
            core: ViewCore {
                bounds,
                palette_chain: None,
                ..ViewCore::default()
            },
            view_state: State::empty(),
            min,
            max,
            value: min,
            step: 1,
            on_change: 0,
            dragging: false,
        }
    }

    /// The current value.
    pub fn value(&self) -> i64 {
        self.value
    }

    /// Set the value, clamped to the range. Returns whether it changed.
    pub fn set_value(&mut self, value: i64) -> bool {
        let value = value.clamp(self.min, self.max);
        let changed = value != self.value;
        self.value = value;
        changed
    }

    /// The bounds, smallest first.
    pub fn range(&self) -> (i64, i64) {
        (self.min, self.max)
    }

    /// How far Left and Right move. At least 1.
    pub fn set_step(&mut self, step: i64) {
        self.step = step.max(1);
    }

    /// Broadcast `command` whenever the user changes the value; 0 turns it off.
    pub fn set_on_change(&mut self, command: CommandId) {
        self.on_change = command;
    }

    /// The span of the range, in a type that cannot overflow.
    fn span(&self) -> i128 {
        i128::from(self.max) - i128::from(self.min)
    }

    /// The column the thumb sits in, for a track `width` cells wide.
    fn thumb_col(&self, width: usize) -> usize {
        let track = i128::try_from(width.saturating_sub(1)).unwrap_or(0);
        let span = self.span();
        if track == 0 || span == 0 {
            return 0;
        }
        let offset = i128::from(self.value) - i128::from(self.min);
        usize::try_from(track * offset / span).unwrap_or(0)
    }

    /// The value under column `col` of a track `width` cells wide.
    fn value_at(&self, col: i16, width: usize) -> i64 {
        let track = i128::try_from(width.saturating_sub(1)).unwrap_or(0);
        if track == 0 {
            return self.min;
        }
        let col = i128::from(col).clamp(0, track);
        let value = i128::from(self.min) + self.span() * col / track;
        i64::try_from(value).unwrap_or(self.max)
    }

    fn report(&self, event: &mut Event, changed: bool) {
        if changed && self.on_change != 0 {
            *event = Event::broadcast(self.on_change);
        } else {
            event.clear();
        }
    }
}

impl View for Slider {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn state(&self) -> StateFlags {
        self.view_state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.view_state = state;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = usize::try_from(self.core.bounds.width_clamped()).unwrap_or(0);
        if width == 0 {
            return;
        }
        // Like Spinner: no cursor, so focus borrows the selected-text colour.
        let track_attr = if self.is_focused() {
            self.map_color(INPUT_SELECTED)
        } else {
            self.map_color(INPUT_NORMAL)
        };
        let thumb_attr = self.map_color(INPUT_ARROWS);

        let mut buf = DrawBuffer::new(width);
        buf.move_char(0, TRACK, track_attr, width);
        buf.put_char(self.thumb_col(width), THUMB, thumb_attr);
        write_line_to_terminal(terminal, 0, 0, &buf);
    }

    fn handle_event(&mut self, event: &mut Event) {
        let width = usize::try_from(self.core.bounds.width_clamped()).unwrap_or(0);
        match event.what {
            EventType::MouseDown
                if event.mouse.buttons & MB_LEFT_BUTTON != 0
                    && self.extent().contains(event.mouse.pos) =>
            {
                self.dragging = true;
                let changed = self.set_value(self.value_at(event.mouse.pos.x, width));
                self.report(event, changed);
            }
            EventType::MouseMove => {
                // If dragging, continue to update even if the pointer leaves the track.
                if self.dragging {
                    if event.mouse.buttons & MB_LEFT_BUTTON != 0 {
                        let changed = self.set_value(self.value_at(event.mouse.pos.x, width));
                        self.report(event, changed);
                    } else {
                        // Button released; end the drag.
                        self.dragging = false;
                        event.clear();
                    }
                }
            }
            EventType::Keyboard if self.is_focused() => {
                let target = match event.key_code {
                    KB_LEFT => self.value.saturating_sub(self.step),
                    KB_RIGHT => self.value.saturating_add(self.step),
                    KB_HOME => self.min,
                    KB_END => self.max,
                    _ => return,
                };
                let changed = self.set_value(target);
                self.report(event, changed);
            }
            _ => {}
        }
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_INPUT_LINE))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geometry::Point;

    fn focused(min: i64, max: i64, width: i16) -> Slider {
        let mut s = Slider::new(Rect::new(0, 0, width, 1), min, max);
        s.set_focus(true);
        s
    }

    fn key(s: &mut Slider, code: u16) -> Event {
        let mut e = Event::keyboard(code);
        s.handle_event(&mut e);
        e
    }

    fn click(s: &mut Slider, x: i16) -> Event {
        let mut e = Event::nothing();
        e.what = EventType::MouseDown;
        e.mouse.pos = Point::new(x, 0);
        e.mouse.buttons = MB_LEFT_BUTTON;
        s.handle_event(&mut e);
        e
    }

    fn mouse_down(s: &mut Slider, x: i16, y: i16) -> Event {
        let mut e = Event::nothing();
        e.what = EventType::MouseDown;
        e.mouse.pos = Point::new(x, y);
        e.mouse.buttons = MB_LEFT_BUTTON;
        s.handle_event(&mut e);
        e
    }

    fn mouse_move(s: &mut Slider, x: i16, y: i16, buttons: u8) -> Event {
        let mut e = Event::nothing();
        e.what = EventType::MouseMove;
        e.mouse.pos = Point::new(x, y);
        e.mouse.buttons = buttons;
        s.handle_event(&mut e);
        e
    }

    #[test]
    fn keys_step_and_clamp() {
        let mut s = focused(0, 10, 11);
        s.set_step(3);
        assert_eq!(key(&mut s, KB_RIGHT).what, EventType::Nothing);
        assert_eq!(s.value(), 3);
        key(&mut s, KB_END);
        assert_eq!(s.value(), 10);
        key(&mut s, KB_RIGHT);
        assert_eq!(s.value(), 10);
        key(&mut s, KB_HOME);
        key(&mut s, KB_LEFT);
        assert_eq!(s.value(), 0);
    }

    #[test]
    fn a_click_sets_the_value_under_the_mouse() {
        // 11 cells over 0..=10: one value per cell.
        let mut s = focused(0, 10, 11);
        click(&mut s, 7);
        assert_eq!(s.value(), 7);
        click(&mut s, 50);
        assert_eq!(s.value(), 7, "clicks outside the view are ignored");
    }

    #[test]
    fn on_change_broadcasts_only_real_changes() {
        let mut s = focused(0, 10, 11);
        s.set_on_change(900);
        let e = key(&mut s, KB_RIGHT);
        assert_eq!((e.what, e.command), (EventType::Broadcast, 900));
        key(&mut s, KB_END);
        let e = key(&mut s, KB_RIGHT);
        assert_eq!(e.what, EventType::Nothing, "already at max");
    }

    #[test]
    fn the_thumb_tracks_the_value() {
        let mut s = focused(0, 10, 11);
        s.set_value(10);
        let mut term = crate::test_util::test_terminal(11, 1);
        s.draw(&mut term);
        assert_eq!(term.read_cell(10, 0).unwrap().ch, THUMB);
        assert_eq!(term.read_cell(0, 0).unwrap().ch, TRACK);
    }

    #[test]
    fn degenerate_ranges_do_not_panic() {
        let s = Slider::new(Rect::new(0, 0, 10, 1), 9, 2);
        assert_eq!(s.range(), (2, 9), "reversed bounds are swapped");

        for (min, max, width) in [(5, 5, 10), (0, 100, 1), (i64::MIN, i64::MAX, 20)] {
            let mut s = focused(min, max, width);
            key(&mut s, KB_END);
            key(&mut s, KB_LEFT);
            click(&mut s, width - 1);
            let mut term = crate::test_util::test_terminal(width as u16, 1);
            s.draw(&mut term);
            let thumbs = (0..width).filter(|&x| term.read_cell(x, 0).unwrap().ch == THUMB).count();
            assert_eq!(thumbs, 1, "one thumb inside the view for {min}..={max} in {width}");
        }
    }

    #[test]
    fn dragging_past_either_end_clamps() {
        // 11 cells over 0..=10: one value per cell.
        let mut s = focused(0, 10, 11);

        // Start a drag at x=5 (value 5).
        mouse_down(&mut s, 5, 0);
        assert_eq!(s.value(), 5);

        // Drag far to the right, past the track's edge, clamping to max.
        mouse_move(&mut s, 40, 3, MB_LEFT_BUTTON);
        assert_eq!(s.value(), 10, "drag to x=40 (far right) clamps to max");

        // Drag far to the left, past the track's edge, clamping to min.
        mouse_move(&mut s, -20, 0, MB_LEFT_BUTTON);
        assert_eq!(s.value(), 0, "drag to x=-20 (far left) clamps to min");

        // Release the button; end the drag.
        mouse_move(&mut s, 5, 0, 0);

        // A subsequent move at the same position without the button changes nothing.
        mouse_move(&mut s, 5, 0, 0);
        assert_eq!(s.value(), 0, "after release, moves without button do nothing");
    }
}
