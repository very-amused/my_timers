use tokio::sync::mpsc;
use chrono::{DateTime, Local};
use std::cmp;
use super::Event;

/// A deferred event added to the global event queue.
/// Used with sqlite to prevent write lock contention
pub struct EventTask<'e> {
	pub event: &'e Event,
	pub queued_at: DateTime<Local>
}

pub struct EventQueue<'e> {
	pub tx: Option<mpsc::Sender<EventTask<'e>>>,
	pub rx: Option<mpsc::Receiver<EventTask<'e>>>
}

impl EventQueue<'_> {
	pub fn new(driver: &str, n_events: usize) -> Self {
		if driver != "sqlite" {
			return EventQueue{tx: None, rx: None};
		}

		// Create channel with the capacity to hold 5 minutes worth of worst-case event backlog
		let (tx, rx) = mpsc::channel::<EventTask>(5 * cmp::max(n_events, 1));

		EventQueue {
			tx: Some(tx),
			rx: Some(rx)
		}
	}
}
