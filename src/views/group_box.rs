// (C) 2025 - Enzo Lombardi

//! `GroupBox` view - a titled frame drawn around related controls.
//!
//! A `GroupBox` only draws: a single-line box with its title set into the top
//! edge, in the dialog's text colours. The controls it surrounds are not its
//! children; they are siblings added after it, so they draw on top of it and
//! keep their own focus order, and a dialog still finds them with
//! [`GroupLike::get`](super::GroupLike::get). [`Form::group`](super::form::Form::group)
//! creates one around the rows that follow.
//!
//! ```text
//! ┌─ Address ─────────────────┐
//! │ Street  ________________  │
//! │ City    ________________  │
//! └───────────────────────────┘
//! ```

use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::draw::DrawBuffer;
use crate::core::event::Event;
use crate::core::geometry::Rect;
use crate::core::palette::STATIC_TEXT_NORMAL;
use crate::terminal::Terminal;

pub struct GroupBox {
    core: ViewCore,
    title: String,
}

impl GroupBox {
    /// A box filling `bounds`, with `title` in its top edge (empty for none).
    /// A `~` in the title is not a hot key: it is left out when drawn.
    pub fn new(bounds: Rect, title: &str) -> Self {
        Self {
            core: ViewCore {
                bounds,
                palette_chain: None,
                ..ViewCore::default()
            },
            title: title.chars().filter(|&c| c != '~').collect(),
        }
    }

    /// The title shown in the top edge.
    pub fn title(&self) -> &str {
        &self.title
    }
}

impl View for GroupBox {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = usize::try_from(self.core.bounds.width_clamped()).unwrap_or(0);
        let height = self.core.bounds.height_clamped();
        if width < 2 || height < 2 {
            return;
        }
        let attr = self.map_color(STATIC_TEXT_NORMAL);

        // Top edge: ┌─ Title ───┐, the title cut short if the box is narrow.
        let mut top = DrawBuffer::new(width);
        top.move_char(0, '─', attr, width);
        top.put_char(0, '┌', attr);
        top.put_char(width - 1, '┐', attr);
        if !self.title.is_empty() && width > 6 {
            let room = width - 6;
            let title: String = self.title.chars().take(room).collect();
            top.move_str(2, &format!(" {title} "), attr);
        }
        write_line_to_terminal(terminal, 0, 0, &top);

        // Sides, with the inside cleared to the background.
        let mut side = DrawBuffer::new(width);
        side.move_char(0, ' ', attr, width);
        side.put_char(0, '│', attr);
        side.put_char(width - 1, '│', attr);
        for y in 1..height - 1 {
            write_line_to_terminal(terminal, 0, y, &side);
        }

        let mut bottom = DrawBuffer::new(width);
        bottom.move_char(0, '─', attr, width);
        bottom.put_char(0, '└', attr);
        bottom.put_char(width - 1, '┘', attr);
        write_line_to_terminal(terminal, 0, height - 1, &bottom);
    }

    fn handle_event(&mut self, _event: &mut Event) {
        // A box only draws: clicks on it go nowhere, and it never takes focus.
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_STATIC_TEXT))
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

    fn row(t: &Terminal, y: i16, width: i16) -> String {
        (0..width)
            .map(|x| t.read_cell(x, y).map_or('?', |c| c.ch))
            .collect()
    }

    #[test]
    fn it_draws_a_titled_box() {
        let mut gb = GroupBox::new(Rect::new(0, 0, 16, 3), "~A~ddress");
        let mut t = crate::test_util::test_terminal(20, 5);
        gb.draw(&mut t);
        assert_eq!(row(&t, 0, 16), "┌─ Address ────┐");
        assert_eq!(row(&t, 1, 16), "│              │");
        assert_eq!(row(&t, 2, 16), "└──────────────┘");
    }

    #[test]
    fn a_long_title_is_cut_to_fit() {
        let mut gb = GroupBox::new(Rect::new(0, 0, 10, 2), "Shipping address");
        let mut t = crate::test_util::test_terminal(12, 3);
        gb.draw(&mut t);
        assert_eq!(row(&t, 0, 10), "┌─ Ship ─┐");
    }

    #[test]
    fn it_never_takes_the_focus() {
        assert!(!GroupBox::new(Rect::new(0, 0, 10, 3), "x").can_focus());
    }
}
