use std::{fs::File, error::Error};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
	// Config for connecting to the MariaDB/MySQL database
	pub db: crate::db::Config,
	// Log config options
	pub log: crate::logging::Config
}

pub fn parse(path: &str) -> Result<Config, Box<dyn Error>> {
	let config_file = File::open(path)?;
	let mut config: Config = serde_json::from_reader(config_file)?;
	config.log.init_format_opts();
	Ok(config)
}
