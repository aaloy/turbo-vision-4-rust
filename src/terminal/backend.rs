// (C) 2025 - Enzo Lombardi

//! Backend trait for terminal I/O abstraction.
//!
//! This module defines the [`Backend`] trait that abstracts low-level terminal
//! operations, allowing turbo-vision to work with different terminal transports
//! such as crossterm (local terminal) or a remote channel, such as the
//! `tv-extensions` crate's SSH backend.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │                  Terminal                       │
//! │  (high-level: buffers, clipping, events)       │
//! └─────────────────────┬───────────────────────────┘
//!                       │
//!                       ▼
//! ┌─────────────────────────────────────────────────┐
//! │              Backend Trait                      │
//! │  (low-level: raw I/O, cursor, init/cleanup)    │
//! └────────┬────────────────────────────┬──────────┘
//!          │                            │
//!          ▼                            ▼
//! ┌─────────────────┐          ┌───────────────────────┐
//! │ CrosstermBackend│          │    A remote backend    │
//! │ (local terminal)│          │ (e.g. tv-extensions'  │
//! │                 │          │      SSH backend)      │
//! └─────────────────┘          └───────────────────────┘
//! ```

use std::io;
use std::time::Duration;

use crate::core::event::Event;

/// Terminal capabilities that a backend may or may not support.
///
/// This allows turbo-vision to adapt its behavior based on what the
/// connected terminal can handle.
#[derive(Debug, Clone, Copy)]
pub struct Capabilities {
    /// Whether the terminal supports mouse input.
    pub mouse: bool,
    /// Whether the terminal supports 256-color mode.
    pub colors_256: bool,
    /// Whether the terminal supports true color (24-bit RGB).
    pub true_color: bool,
    /// Whether bracketed paste mode is supported.
    pub bracketed_paste: bool,
    /// Whether focus events are supported.
    pub focus_events: bool,
    /// Whether the kitty keyboard protocol is supported.
    pub kitty_keyboard: bool,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            mouse: true,
            colors_256: true,
            true_color: false,
            bracketed_paste: false,
            focus_events: false,
            kitty_keyboard: false,
        }
    }
}

/// The core abstraction for terminal I/O operations.
///
/// This trait defines the interface that all terminal backends must implement.
/// It covers initialization, cleanup, event polling, cursor control, and
/// raw output operations.
///
/// # Implementation Notes
///
/// Backends are responsible for:
/// - Managing terminal mode (raw mode, alternate screen)
/// - Providing terminal dimensions
/// - Polling for and delivering input events
/// - Writing raw output data (ANSI escape sequences)
/// - Cursor visibility and positioning
///
/// The [`Terminal`](super::Terminal) struct handles higher-level concerns:
/// - Double-buffered rendering
/// - Differential updates
/// - Clipping regions
/// - Event queuing
pub trait Backend: Send {
    /// Downcasting hook so `Terminal` can reach backend-specific settings
    /// (e.g. the crossterm ESC-timeout) without widening the trait.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    /// Initialize the backend.
    ///
    /// This should set up the terminal for TUI operation:
    /// - Enter raw mode (no line buffering, no echo)
    /// - Enter alternate screen buffer
    /// - Hide cursor
    /// - Enable mouse capture (if supported)
    /// - Disable line wrapping
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails.
    fn init(&mut self) -> io::Result<()>;

    /// Clean up and restore the backend to its original state.
    ///
    /// This should reverse all changes made by [`init`](Self::init):
    /// - Show cursor
    /// - Disable mouse capture
    /// - Leave alternate screen
    /// - Disable raw mode
    /// - Re-enable line wrapping
    ///
    /// # Errors
    ///
    /// Returns an error if cleanup fails. Note that cleanup failures
    /// are often non-fatal and the terminal may still be usable.
    fn cleanup(&mut self) -> io::Result<()>;

    /// Get the current terminal dimensions.
    ///
    /// Returns `(width, height)` in character cells.
    ///
    /// # Errors
    ///
    /// Returns an error if the dimensions cannot be queried.
    fn size(&self) -> io::Result<(u16, u16)>;

    /// Poll for an input event with a timeout.
    ///
    /// Returns `Ok(Some(event))` if an event is available, `Ok(None)` if
    /// the timeout expires with no event, or an error if polling fails.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for an event.
    ///
    /// # Errors
    ///
    /// Returns an error if event polling fails (e.g., channel disconnected).
    fn poll_event(&mut self, timeout: Duration) -> io::Result<Option<Event>>;

    /// Write raw data to the terminal.
    ///
    /// This writes bytes directly to the terminal output without any
    /// processing. Used for ANSI escape sequences and character output.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw bytes to write.
    ///
    /// # Errors
    ///
    /// Returns an error if the write fails.
    fn write_raw(&mut self, data: &[u8]) -> io::Result<()>;

    /// Flush any buffered output to the terminal.
    ///
    /// # Errors
    ///
    /// Returns an error if flushing fails.
    fn flush(&mut self) -> io::Result<()>;

    /// Move the cursor to a position and show it.
    ///
    /// # Arguments
    ///
    /// * `x` - Column (0-indexed).
    /// * `y` - Row (0-indexed).
    ///
    /// # Errors
    ///
    /// Returns an error if cursor control fails.
    fn show_cursor(&mut self, x: u16, y: u16) -> io::Result<()>;

    /// Hide the cursor.
    ///
    /// # Errors
    ///
    /// Returns an error if cursor control fails.
    fn hide_cursor(&mut self) -> io::Result<()>;

    /// Query terminal capabilities.
    ///
    /// Returns the capabilities that this backend supports.
    fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }

    /// Suspend the terminal for shell escape (Ctrl+Z handling).
    ///
    /// This restores the terminal to normal mode while keeping the
    /// backend alive. Call [`resume`](Self::resume) to return to TUI mode.
    ///
    /// # Errors
    ///
    /// Returns an error if suspension fails.
    fn suspend(&mut self) -> io::Result<()> {
        self.cleanup()
    }

    /// Resume the terminal after suspension.
    ///
    /// Re-initializes the terminal for TUI operation after a
    /// [`suspend`](Self::suspend) call.
    ///
    /// # Errors
    ///
    /// Returns an error if resumption fails.
    fn resume(&mut self) -> io::Result<()> {
        self.init()
    }

    /// Query the terminal cell aspect ratio for shadow rendering.
    ///
    /// Returns `(horizontal, vertical)` multipliers for shadow proportions.
    /// Typical terminal fonts are ~2:1 ratio (cells are taller than wide).
    ///
    /// Default returns `(2, 1)` which works for most terminals.
    fn cell_aspect_ratio(&self) -> (i16, i16) {
        (2, 1)
    }

    /// Emit a terminal bell (beep) sound.
    ///
    /// # Errors
    ///
    /// Returns an error if the bell cannot be emitted.
    fn bell(&mut self) -> io::Result<()> {
        self.write_raw(b"\x07")?;
        self.flush()
    }

    /// Whether an embedder steps the application over this backend instead
    /// of the application running its own event loop.
    ///
    /// A host-driven backend belongs to a host that owns the screen and the
    /// input, such as a WASM guest's frame: the host calls into the
    /// application once per frame and can only deliver the next event after
    /// that call returns. [`Application`](crate::app::Application) reads this
    /// once, when it is built, and on a host-driven backend refuses every
    /// loop that would block waiting for input (modal views, the History and
    /// dropdown popups) with `CM_CANCEL`. `poll_event` on such a backend
    /// should never wait either.
    ///
    /// Defaults to `false`, which is right for every backend that owns a
    /// terminal.
    fn is_host_driven(&self) -> bool {
        false
    }

    /// Clear the entire screen.
    ///
    /// # Errors
    ///
    /// Returns an error if the clear fails.
    fn clear_screen(&mut self) -> io::Result<()> {
        // Reset colors first to prevent color bleed
        self.write_raw(b"\x1b[0m")?;
        self.write_raw(b"\x1b[2J")?;
        self.write_raw(b"\x1b[H")?;
        self.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only the required methods: every provided one keeps its default.
    struct Plain;

    impl Backend for Plain {
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
        fn init(&mut self) -> io::Result<()> {
            Ok(())
        }
        fn cleanup(&mut self) -> io::Result<()> {
            Ok(())
        }
        fn size(&self) -> io::Result<(u16, u16)> {
            Ok((80, 25))
        }
        fn poll_event(&mut self, _timeout: Duration) -> io::Result<Option<Event>> {
            Ok(None)
        }
        fn write_raw(&mut self, _data: &[u8]) -> io::Result<()> {
            Ok(())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
        fn show_cursor(&mut self, _x: u16, _y: u16) -> io::Result<()> {
            Ok(())
        }
        fn hide_cursor(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn a_backend_is_not_host_driven_unless_it_says_so() {
        assert!(!Plain.is_host_driven());
    }
}
