//! Progress, known or unknown. Set the value with `set_value` (or `advance`)
//! against the `max` given to `new`; it can show the percentage or a
//! caption, in smooth, block or ASCII style. For work of unknown length,
//! `set_mode(ProgressMode::Marquee)` sweeps a block that `tick()` moves.
//! The button runs a job in a modal dialog, advanced from its tick closure.

use crate::panel::Panel;
use std::time::{Duration, Instant};
use turbo_vision::app::{Application, ModalTick};
use turbo_vision::core::command::{CM_CANCEL, CM_OK, CM_USER, CommandId};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::GroupLike;
use turbo_vision::views::button::Button;
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::msgbox::message_box_ok;
use turbo_vision::views::progress_bar::{ProgressBar, ProgressStyle};

const RUN: CommandId = CM_USER + 100;

pub fn build(panel: &mut Panel) {
    let mut smooth = ProgressBar::new(Rect::new(0, 0, 46, 1), 100);
    smooth.set_value(35);
    smooth.show_percent();
    panel.add(smooth);

    let mut blocks = ProgressBar::new(Rect::new(0, 1, 46, 2), 100);
    blocks.set_style(ProgressStyle::Blocks);
    blocks.set_value(70);
    panel.add(blocks);

    let mut ascii = ProgressBar::new(Rect::new(0, 2, 46, 3), 4);
    ascii.set_style(ProgressStyle::Ascii);
    ascii.set_value(3);
    ascii.set_caption("3 of 4 files");
    panel.add(ascii);

    panel.add(Button::new(
        Rect::new(0, 4, 16, 6),
        "~R~un a job",
        RUN,
        true,
    ));
}

pub fn handle(app: &mut Application, command: CommandId) -> bool {
    if command != RUN {
        return false;
    }
    let mut dialog = Dialog::new(Rect::new(15, 8, 65, 15), "Copying");
    let mut bar = ProgressBar::new(Rect::new(2, 1, 46, 2), 50);
    bar.show_percent();
    let bar = dialog.add_typed(bar);
    dialog.add(Button::new(
        Rect::new(18, 3, 30, 5),
        "Cancel",
        CM_CANCEL,
        true,
    ));

    // The tick closure runs on every pass of the modal loop.
    let mut last = Instant::now();
    let result = app.execute_modal(&mut dialog, |_, dialog| {
        if last.elapsed() < Duration::from_millis(40) {
            return ModalTick::Continue;
        }
        last = Instant::now();
        let Some(bar) = dialog.get_mut(bar) else {
            return ModalTick::End(CM_CANCEL);
        };
        bar.advance(1);
        if bar.value() >= bar.max() {
            ModalTick::End(CM_OK)
        } else {
            ModalTick::Continue
        }
    });
    let text = if result == CM_OK {
        "The job finished."
    } else {
        "The job was cancelled."
    };
    message_box_ok(app, text);
    true
}
