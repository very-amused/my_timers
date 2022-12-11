use std::{env, error::Error};

mod config;
mod db;

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
	let pool = mysql::Pool::new(opts)?;
	pool.get_conn()?;
	println!("Connected to database {}", config.db.pretty_name());

	Ok(())
}
