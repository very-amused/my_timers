use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
	// Name of the my_timers process (i.e CSplan, CSplan_Staging)
	name: String,
	// Config for connecting to the MariaDB/MySQL database
	db: DBConfig
}

impl Config {

}

#[derive(Serialize, Deserialize)]
pub struct DBConfig {
	user: String,
	#[serde(default)]
	password: String,

	// Connection protocol to use, SOCKET or TLS are valid
	#[serde(default = "DBConfig::default_protocol")]
	protocol: String,

	#[serde(default = "DBConfig::default_address")]
	address: String,

	database: String,

	#[serde(default = "DBConfig::default_tls")]
	tls: bool
}

impl DBConfig {
	fn default_protocol() -> String {
		"SOCKET".into()
	}
	fn default_address() -> String {
		"/var/run/mysqld/mysqld.sock".into()
	}
	fn default_tls() -> bool {
		false
	}
}

pub fn parse(path: &str) -> Config {
	todo!()
}
