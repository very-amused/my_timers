use std::{fs::File, error::Error};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
	// Name of the my_timers process (i.e CSplan, CSplan_Staging)
	pub name: String,
	// Config for connecting to the MariaDB/MySQL database
	pub db: crate::db::Config
}

impl Config {

}

pub fn parse(path: &str) -> Result<Config, Box<dyn Error>> {
	let config_file = File::open(path)?;
	let config: Config = serde_json::from_reader(config_file)?;
	Ok(config)
}
