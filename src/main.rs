use std::{env, error::Error};
use tracing_subscriber::prelude::*;

mod config;
mod db;
mod logging;

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
	let file_guarded = match config.log.file {
		Some(f) => f.layer(),
		None => None
	};
	let (file_log, _guard) = if let Some(f) = file_guarded { 
		(Some(f.0), Some(f.1))
	} else { 
		(None, None)
	};
	let sub = tracing_subscriber::FmtSubscriber::new()
		.with(file_log);

	Ok(())
}
