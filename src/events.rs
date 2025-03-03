use std::{error::Error, fs::File, io::{BufReader, BufRead}, collections::{VecDeque, HashSet}, fmt::Display, pin::Pin};

use sqlx::{AnyPool, Executor};
use tracing::{instrument, event, Level, span, Instrument};
use tokio::sync::mpsc;
use chrono::Local;
use lazy_static::lazy_static;

use crate::cron::{self, error::CronParseError};

mod queue;
pub use queue::{EventTask, EventQueue};

#[derive(Debug)]
pub enum EventParseError {
	CronParseError(CronParseError),
	SyntaxError(String),
	SQLError(sqlx::Error)
}

impl Display for EventParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::CronParseError(e) => e.fmt(f),
			Self::SyntaxError(e) => write!(f, "Invalid event syntax: {}", e),
			Self::SQLError(e) => e.fmt(f)
		}
	}
}

impl Error for EventParseError {}

/// A database event
#[derive(Debug)]
pub struct Event {
	pub label: String,
	pub interval: cron::CronInterval,
	body: Vec<String> // Each stmt in an event body is validated as an SQL stmt during initial parsing
}

impl Event {
	async fn parse(evt_parts: &mut VecDeque<String>, pool: AnyPool) -> Result<Pin<Box<Event>>, EventParseError> {
		if evt_parts.len() != 3 {
			return Err(EventParseError::SyntaxError(format!("{} unexpected number of event tokens (expected {}, received {})",
				evt_parts.get(1).unwrap_or(&"".into()), 3, evt_parts.len())));
		}
		// Parse label and interval
		let mut evt = Event {
			label: evt_parts.pop_front().unwrap().trim().to_string(),
			interval: evt_parts.pop_front().unwrap().trim().parse()
				.map_err(EventParseError::CronParseError)?,
			body: Vec::new()
		};

		// Parse SQL body
		let body = evt_parts.pop_front().unwrap();
		let mut stmts: VecDeque<String> = body.split(";")
			.map(|s| s.trim().replace("\t", "")) // Remove tabs
			.filter(|s| s.len() > 0).collect();

		while let Some(stmt) = stmts.pop_front() {
			// Validate SQL stmt
			pool.prepare(&stmt).await
				.map_err(EventParseError::SQLError)?;
			// Push to event body
			evt.body.push(stmt);
		}

		evt_parts.clear(); // Ensure the event parsing queue is empty
		Ok(Box::pin(evt))
	}

	/// Run an event's SQL body on a transaction,
	/// only committing the results if all statements succeed
	#[instrument(skip_all, fields(event = %self, interval = %self.interval), err)]
	pub async fn run<'e>(&'e self, pool: AnyPool, queue_tx: Option<mpsc::Sender<EventTask<'e>>>) -> Result<(), Box<dyn Error + 'e>> {
		// Queue the event instead of immediately running if needed (non-concurrent drivers such as sqlite)
		if let Some(tx) = queue_tx {
			event!(Level::INFO, "Queueing event");
			tx.send(EventTask{
				event: self,
				queued_at: Local::now()
			}).await?;
			return Ok(());
		}

		// Start a transaction to run the event on
		event!(Level::INFO, "Running event");
		let mut tx = pool.begin().await?;

		// Run the event body
		let mut i: usize = 0;
		for stmt in &self.body {
			let span = span!(Level::DEBUG, "Exec", stmt = i,  action = Self::action(stmt));
			async {
				let result = sqlx::query(stmt)
					.execute(&mut *tx)
					.await?;
				event!(Level::DEBUG, "{} Rows affected", result.rows_affected());
				Ok::<(), sqlx::Error>(())
			}.instrument(span).await?;
			i += 1;
		}

		tx.commit().await?;
		event!(Level::INFO, "Done");
		Ok(())
	}

	/// Parse an action summary of an SQL statement
	/// i.e "UPDATE my_table, another_table", "DELETE FROM my_table", "INSERT INTO my_table"
	fn action(stmt: &str) -> String {
		// Tokens that end parsing (self-exclusive) of the "action" portion of various kinds of queries
		lazy_static! {
			static ref END_INSERT: HashSet<&'static str> = HashSet::from(["PARTITION", "SELECT", "VALUES", "VALUE"]);
			static ref END_UPDATE: HashSet<&'static str> = HashSet::from(["PARTITION", "FOR", "SET"]);
			static ref END_DELETE: HashSet<&'static str> = HashSet::from(["PARTITION", "FOR", "WHERE", "ORDER", "LIMIT", "RETURNING", "BEFORE"]);
		}
		let tokens: Vec<&str> = stmt.trim().split(" ").collect();
		let mut end = 2; // Exclusive end token index
		match tokens[0].to_uppercase().as_str() {
			"INSERT" => while end < tokens.len() && !(
				END_INSERT.contains(tokens[end].to_ascii_uppercase().as_str()) ||
				tokens[end].starts_with('('))
			{
				end += 1;
			},
			"UPDATE" => while end < tokens.len() && !END_UPDATE.contains(tokens[end].to_ascii_uppercase().as_str()) {
				end += 1;
			},
			"DELETE" => while end < tokens.len() && !END_DELETE.contains(tokens[end].to_ascii_uppercase().as_str()) {
				end += 1;
			}
			_ => {}
		}
		tokens[0..end].join(" ")
	}
}

impl Display for Event {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.label)
	}
}

#[instrument(name = "Parsing events", level = "debug", skip(pool), err)]
pub async fn parse(path: &str, pool: AnyPool) -> Result<Vec<Pin<Box<Event>>>, Box<dyn Error>> {
	event!(Level::DEBUG, "Parsing events");
	// Open file reader
	let file = File::open(path)?;
	let reader = BufReader::new(file);

	let mut events: Vec<Pin<Box<Event>>> = Vec::new();

	// Iterate over lines
	let mut evt_parts: VecDeque<String> = VecDeque::with_capacity(3);
	for l in reader.lines().map(|l| -> String {
		l.unwrap_or("".to_string())
	}) {
		// Discard comments
		const COMMENT: &str = "#";
		let l = l.splitn(2, COMMENT).nth(0).unwrap();

		match evt_parts.len() {
			0 => { // Label (maybe interval)
				let mut l_parts: VecDeque<&str> = l.splitn(2, ':').collect();
				if let Some(label) = l_parts.pop_front() {
					evt_parts.push_back(label.into());
				}
				if let Some(interval) = l_parts.pop_front() {
					evt_parts.push_back(interval.into());
				}
			},
			1 => { // Continue multiline label (maybe interval)
				let mut l_parts: VecDeque<&str> = l.splitn(2, ':').collect();
				if let Some(label) = l_parts.pop_front() {
					evt_parts[0].push(' ');
					evt_parts[0].push_str(label);
				}
				if let Some(interval) = l_parts.pop_front() {
					evt_parts.push_back(interval.into())
				}
			},
			2 => { // Continue/end multiline interval
				// An indented line marks the beginning of an event body
				if l.starts_with('\t') || l.starts_with("  ") {
					evt_parts.push_back(l.into());
				} else {
					evt_parts[1].push_str(&l);
				}
			},
			3 => { // Body
				if l.starts_with('\t') || l.starts_with("  ") {
					evt_parts[2].push(' ');
					evt_parts[2].push_str(&l);
				} else {
					// Parse event
					events.push(Event::parse(&mut evt_parts, pool.clone()).await?);
				}
			},
			_ => panic!("event parsing dequeue exceeded max size of 3")
		};
	}
	// If there is no terminating newline, the last event still needs to be pushed
	if evt_parts.len() == 3 {
		events.push(Event::parse(&mut evt_parts, pool.clone()).await?);
	}

	{
		// Displayable events
		let mut d_events: Vec<String> = Vec::with_capacity(events.len());
		for evt in &events {
			d_events.push(format!("{}", evt));
		}
		event!(Level::TRACE, "Loaded events:\n\t{}", d_events.join("\n\t"));
	}
	event!(Level::DEBUG, "Done");
	Ok(events)
}
