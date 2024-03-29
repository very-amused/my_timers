use std::{fs::File, error::Error};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
	// Config for connecting to the MariaDB/MySQL database
	pub db: crate::db::Config,
	// Config for log outputs
	pub log: crate::logging::Config
}

pub fn parse(path: &str) -> Result<Config, Box<dyn Error>> {
	let config_file = File::open(path)?;
	let config: Config = serde_json::from_reader(config_file)?;
	Ok(config)
}
