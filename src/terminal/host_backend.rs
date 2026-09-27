// (C) 2025 - Enzo Lombardi

//! A backend for an embedder that owns the screen: events are pushed in
//! through [`HostInput`], and output goes nowhere — the embedder reads the
//! finished cells from [`Terminal::buffer`](super::Terminal::buffer).
//!
//! `poll_event` never waits. A host-driven application is stepped by its
//! embedder, one [`Application::pump`](crate::app::Application::pump) at a
//! time, so blocking for input would stall the host instead.

use std::collections::VecDeque;
use std::io;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use super::backend::{Backend, Capabilities};
use crate::core::event::Event;

#[derive(Debug, Default)]
struct Shared {
    queue: VecDeque<Event>,
    size: (u16, u16),
}

/// Locks the shared state. The critical sections only push, pop or assign,
/// so a panic elsewhere can't leave it half-updated: poisoning is ignored.
fn lock(shared: &Mutex<Shared>) -> MutexGuard<'_, Shared> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The embedder's end: push events, change the size.
#[derive(Debug, Clone)]
pub struct HostInput(Arc<Mutex<Shared>>);

impl HostInput {
    /// Queues one event for the next pump.
    pub fn push(&self, event: Event) {
        lock(&self.0).queue.push_back(event);
    }

    /// Sets the screen size the next pump lays out for.
    pub fn set_size(&self, w: u16, h: u16) {
        lock(&self.0).size = (w, h);
    }

    /// Events still queued.
    #[must_use]
    pub fn len(&self) -> usize {
        lock(&self.0).queue.len()
    }

    /// True when nothing is queued.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// The application's end.
#[derive(Debug)]
pub struct HostBackend(Arc<Mutex<Shared>>);

impl HostBackend {
    /// A backend of `w` by `h` cells, and the handle that feeds it.
    #[must_use]
    pub fn new(w: u16, h: u16) -> (Self, HostInput) {
        let shared = Arc::new(Mutex::new(Shared {
            queue: VecDeque::new(),
            size: (w, h),
        }));
        (Self(Arc::clone(&shared)), HostInput(shared))
    }
}

impl Backend for HostBackend {
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
        Ok(lock(&self.0).size)
    }
    fn poll_event(&mut self, _timeout: Duration) -> io::Result<Option<Event>> {
        Ok(lock(&self.0).queue.pop_front())
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
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            mouse: false,
            colors_256: true,
            true_color: true,
            ..Capabilities::default()
        }
    }
}
