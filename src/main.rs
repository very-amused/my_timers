use std::{env, error::Error};
use tracing::{event, Level};

mod config;
mod db;
mod logging;
mod time_format;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	// Parse config
	let config_path = {
		const ENV: &str = "MY_TIMERS_CONFIG";
		const DEFAULT: &str = "config.json";
		env::var(ENV).or_else(|err| {
			eprintln!("Failed to get config path from {}: {}", ENV, err);
			eprintln!("Falling back to {}", DEFAULT);
			Err(err)
		}).unwrap_or(DEFAULT.into())
	};
	let config = config::parse(&config_path).or_else(|err| {
		eprintln!("Failed to parse config: {}", err);
		Err(err)
	})?;

	// Connect to DB
	let opts = config.db.mysql_opts();
	let pool = mysql::Pool::new(opts).or_else(|err| {
		eprintln!("Failed to connect to DB: {}", err);
		Err(err)
	})?;
	pool.get_conn()?;
	println!("Connected to database {}", config.db.pretty_name());

	// Prepare logging destinations (subscribers)
	let _guard = config.log.init();

	// Test log message
	event!(Level::INFO, "my_timers daemon has started");

	Ok(())
}
