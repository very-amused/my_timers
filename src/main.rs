use std::{env, error::Error, time::Duration};
use chrono::{Timelike, Local};
use tokio::{time::{self, sleep}, task::JoinSet};
use tracing::{event, Level, span, Instrument};

mod config;
mod db;
mod logging;
mod events;
mod cron;

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

	// Read events from config
	let config_path = {
		const ENV: &str = "MY_TIMERS_EVENTS";
		const DEFAULT: &str = "events.conf";
		env::var(ENV).or_else(|err| {
			eprintln!("{} is not set, using {}", ENV, DEFAULT);
			Err(err)
		}).unwrap_or(DEFAULT.into())
	};
	let events = events::parse(&config_path, pool.clone()).await?;

	// Initialize task joinset
	let mut set = JoinSet::<()>::new();
	// Timer thread (runs matching events every minute at xx:00)
	event!(Level::DEBUG, "Starting event loop in {:?}", to_next_minute());
	sleep(to_next_minute()).await;
	event!(parent: None, Level::INFO, "Ready!");
	let mut interval = time::interval(Duration::from_secs(60));
	// Start minute interval ticker
	loop {
		interval.tick().await;
		// Iterate through each event, run the ones that match
		let now = Local::now();
		for evt in &events {
			if evt.interval.match_time(&now) {
				let pool = pool.clone();
				let evt = evt.clone();
				set.spawn(async move {
					// Error logging is handled in the event's tracing span
					evt.run(pool).await.ok();
				});	
			}
		}
	}

	set.shutdown().await;

	Ok(())
}
