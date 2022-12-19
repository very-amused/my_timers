use std::{env, error::Error};
use tracing::{event, Level, span};

mod config;
mod db;
mod logging;
mod time_format;
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
		let span = span!(Level::DEBUG, "Checking database connection");
		let _guard = span.enter();
		pool.get_conn().await?;
		event!(Level::DEBUG, target=config.db.database, "Connected to database {}", config.db.pretty_name());
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
	let events = {
		let span = span!(Level::DEBUG, "Parsing events");
		let _guard = span.enter();
		let events = events::parse(&config_path, pool.clone()).await?;
		event!(Level::DEBUG, "Done");
		events
	};
	event!(Level::TRACE, "{:#?}", events);


	Ok(())
}
