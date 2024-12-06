use std::{fs::File, error::Error};
use serde::Deserialize;
use crate::db::error::DBConfigError;

#[derive(Deserialize)]
pub struct Config {
	// Config for connecting to the MariaDB/MySQL database
	pub db: crate::db::Config,
	// Config for log outputs
	pub log: crate::logging::Config
}

impl Config {
	fn validate(&mut self) -> Result<(), DBConfigError> {
		self.db.set_default_address();
		self.db.validate()
	}
}

pub fn parse(path: &str) -> Result<Config, Box<dyn Error>> {
	let config_file = File::open(path)?;
	let mut config: Config = serde_json::from_reader(config_file)?;
	config.validate()?;
	Ok(config)
}
