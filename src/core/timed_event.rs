// (C) 2025 - Enzo Lombardi

//! Events posted to arrive after a delay.
//!
//! A view cannot reach the event loop from `handle_event`, yet sometimes it
//! needs to act later on its own: a button animated by a key press sends its
//! command once the press has been seen. [`post_after`] queues such an event;
//! [`Terminal::poll_event`](crate::terminal::Terminal::poll_event) returns it
//! once it is due, so it reaches every event loop, the application's and a
//! hand-written one alike, as if it had just been typed. Matches the role of
//! magiblot's `TTimerQueue`, which `TButton` uses for its press animation.
//!
//! The queue is per thread, like the global command set: an application and
//! its views live on one thread.

use super::event::Event;
use std::cell::RefCell;
use std::time::{Duration, Instant};

thread_local! {
    /// Pending events in posting order, each with the instant it is due.
    static QUEUE: RefCell<Vec<(Instant, Event)>> = const { RefCell::new(Vec::new()) };
}

/// Queue `event` to be delivered once `delay` has passed.
///
/// Events due at the same instant arrive in the order they were posted.
pub fn post_after(event: Event, delay: Duration) {
    post_at(event, Instant::now() + delay);
}

/// Queue `event` to be delivered once `due` has come.
pub fn post_at(event: Event, due: Instant) {
    QUEUE.with(|q| q.borrow_mut().push((due, event)));
}

/// Remove and return the earliest event due at `now`, if any.
pub fn take_due(now: Instant) -> Option<Event> {
    QUEUE.with(|q| {
        let mut q = q.borrow_mut();
        let index = (0..q.len())
            .filter(|&i| q[i].0 <= now)
            .min_by_key(|&i| q[i].0)?;
        Some(q.remove(index).1)
    })
}

/// When the earliest pending event is due, or `None` when nothing is queued.
pub fn next_due() -> Option<Instant> {
    QUEUE.with(|q| q.borrow().iter().map(|(due, _)| *due).min())
}

/// Drop every pending event.
pub fn clear() {
    QUEUE.with(|q| q.borrow_mut().clear());
}

/// Remove and return the earliest pending event without waiting for it to
/// come due: what an event loop would deliver next, for tests.
#[cfg(test)]
pub(crate) fn take_next() -> Option<Event> {
    take_due(Instant::now() + Duration::from_secs(3600))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_event_arrives_only_once_due() {
        clear();
        post_after(Event::command(1), Duration::from_secs(60));
        assert!(take_due(Instant::now()).is_none());
        let event = take_due(Instant::now() + Duration::from_secs(61)).expect("due");
        assert_eq!(event.command, 1);
        assert!(next_due().is_none(), "delivered once");
    }

    #[test]
    fn events_arrive_in_due_order_then_posting_order() {
        clear();
        post_after(Event::command(1), Duration::from_millis(20));
        post_after(Event::command(2), Duration::ZERO);
        post_after(Event::command(3), Duration::from_millis(20));
        let later = Instant::now() + Duration::from_secs(1);
        let order: Vec<_> = std::iter::from_fn(|| take_due(later))
            .map(|e| e.command)
            .collect();
        assert_eq!(order, [2, 1, 3]);
    }
}
