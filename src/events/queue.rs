use tokio::sync::{mpsc, oneshot};
use chrono::{DateTime, Local};
use std::{cmp, error::Error};
use sqlx::{AnyPool, Executor};
use tracing::{instrument, event, Level, span, Instrument};
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
			return Self{tx: None, rx: None};
		}

		// Create channel with the capacity to hold 5 minutes worth of worst-case event backlog
		let (tx, rx) = mpsc::channel::<EventTask>(5 * cmp::max(n_events, 1));

		Self {
			tx: Some(tx),
			rx: Some(rx)
		}
	}
}

impl EventTask<'_> {
	/// Equivalent to Event::run for an EventTask pulled from a queue.
	/// Used with non-concurrent drivers.
	#[instrument(skip_all, fields(event = %self.event, interval = %self.event.interval), err)]
	pub async fn run(&self, pool: AnyPool) -> Result<(), Box<dyn Error>> {
		// Start a transaction to run the event on
		let queue_latency = (Local::now() - self.queued_at).to_std()?;
		let queue_latency = format!("{:#?}", queue_latency);
		event!(Level::INFO, queue_latency, "Running event");
		let mut tx = pool.begin().await?;

		// Run the event body

		Ok(())
	}
}
