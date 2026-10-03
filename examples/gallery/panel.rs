// (C) 2025 - Enzo Lombardi
//! Where a demo builds its views.
//!
//! A demo's views go straight into the gallery's dialog, so they take the
//! focus and answer keys like any dialog's controls. A nested `Group` would
//! not: a plain group never takes the focus (open question O5 in
//! docs/DESIGN-SYSTEM-PLAN.md). `Panel` keeps the demo's coordinates simple:
//! `(0, 0)` is the corner of the "Try it" box, as for a group's children.

use turbo_vision::core::geometry::{Point, Rect};
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::{GroupLike, View, ViewId};

/// The area of the gallery's dialog that a demo builds in.
pub struct Panel<'a> {
    dialog: &'a mut Dialog,
    origin: Point,
}

impl<'a> Panel<'a> {
    /// The area of `dialog` whose top-left corner is `origin`.
    pub fn new(dialog: &'a mut Dialog, origin: Point) -> Self {
        Self { dialog, origin }
    }

    /// Add `view`, placed relative to the panel's corner, and return its id
    /// (to link a label to it, for instance).
    pub fn add<V: View + 'static>(&mut self, mut view: V) -> ViewId {
        let b = view.bounds();
        let (x, y) = (self.origin.x, self.origin.y);
        view.set_bounds(Rect::new(b.a.x + x, b.a.y + y, b.b.x + x, b.b.y + y));
        self.dialog.add(view)
    }
}
