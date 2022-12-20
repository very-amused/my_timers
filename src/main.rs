use std::{env, error::Error, time::Duration};
use chrono::{Timelike, Local};
use tokio::{time::{self, sleep}, task::JoinSet, signal};
use tracing::{event, Level, span, Instrument, instrument};

mod config;
mod db;
mod logging;
mod events;
mod cron;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	// Parse config
	let config_path = {
		const ENV: &str = "MY_TIMERS_CONFIG";
		const DEFAULT: &str = "config.json";
		env::var(ENV).or_else(|err| {
			eprintln!("{} is not set, using {}", ENV, DEFAULT);
			Err(err)
		}).unwrap_or(DEFAULT.into())
	};
	let config = config::parse(&config_path).or_else(|err| {
		eprintln!("Failed to parse config: {}", err);
		Err(err)
	})?;


	// Initialize logging destinations
	let _guard = config.log.init();
	event!(Level::INFO, "my_timers started");

	// Connect to DB
	let opts = config.db.mysql_opts();
	let pool = mysql_async::Pool::new(opts);
	{
		let span = span!(Level::DEBUG, "Connecting to DB");
		async {
			event!(Level::DEBUG, "Connecting to database {}", config.db.pretty_name());
			pool.get_conn().await?;
			event!(Level::DEBUG, "Connected");
			Ok::<(), mysql_async::Error>(())
		}.instrument(span).await?;
	}

	// Listen for ctrl+c/SIGINT to safely shutdown.
	// tokio::select! must be used to catch sigints for all future awaits
	// on the main threads
	let ctrl_c = signal::ctrl_c();
	tokio::pin!(ctrl_c);

	// Read events from config
	let config_path = {
		const ENV: &str = "MY_TIMERS_EVENTS";
		const DEFAULT: &str = "events.conf";
		env::var(ENV).or_else(|err| {
			eprintln!("{} is not set, using {}", ENV, DEFAULT);
			Err(err)
		}).unwrap_or(DEFAULT.into())
	};
	let events = tokio::select! {
		evts = events::parse(&config_path, pool.clone()) => evts?,
		Ok(_) = &mut ctrl_c => return shutdown(None, pool).await
	};


	// Initialize task joinset
	let mut event_threads = JoinSet::<()>::new();
	// Timer thread (runs matching events every minute at xx:00)
	event!(Level::DEBUG, "Starting event loop in {:?}", to_next_minute());
	tokio::select! {
		_ = sleep(to_next_minute()) => {},
		Ok(_) = &mut ctrl_c => return shutdown(Some(event_threads), pool).await
	}
	event!(parent: None, Level::INFO, "Ready!");
	// Start minute interval ticker
	let mut interval = time::interval(Duration::from_secs(60));

	// Event loop
	loop {
		// Wait for the next minute, breaking the loop if ctrl+c is pressed
		tokio::select! {
			_ = interval.tick() => {},
			Ok(_) = &mut ctrl_c => return shutdown(Some(event_threads), pool).await
		}
		// Iterate through each event, run the ones that match
		let now = Local::now();
		for evt in &events {
			if evt.interval.match_time(&now) {
				let pool = pool.clone();
				let evt = evt.clone();
				event_threads.spawn(async move {
					// Error logging is handled in the event's tracing span
					evt.run(pool).await.ok();
				});	
			}
		}
	}
}

fn to_next_minute() -> Duration {
	let mut now = chrono::Utc::now();
	// Ceil to nearest ms + 1
	// TODO: explain timing of event loop start
	now = now.with_nanosecond(1_000_000 * (now.nanosecond() / 1_000_000) + 2_000_000).unwrap();
	let mut next = now.clone();
	next = if next.minute() == 59 {
		next.with_hour(next.hour()+1).unwrap()
		.with_minute(0).unwrap()
	} else {
		next.with_minute(next.minute()+1).unwrap()
	};
	next = next.with_second(0).unwrap()
		.with_nanosecond(0).unwrap();
	(next - now).to_std().unwrap()
}

#[instrument(name = "Shutting down", skip_all, err)]
async fn shutdown(event_threads: Option<JoinSet<()>>, pool: mysql_async::Pool) -> Result<(), Box<dyn Error>> {
	event!(Level::INFO, "Shutting down");
	if let Some(mut threads) = event_threads {
		event!(Level::DEBUG, "Stopping event threads");
		threads.shutdown().await;
	}
	event!(Level::DEBUG, "Disconnecting from database");
	pool.disconnect().await?;
	event!(Level::INFO, "Done");
	Ok(())
}
