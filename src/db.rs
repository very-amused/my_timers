use mysql;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
	user: String,
	#[serde(default)]
	password: String,

	// Connection protocol to use, SOCKET or TLS are valid
	#[serde(default = "Config::default_protocol")]
	protocol: String,

	#[serde(default = "Config::default_address")]
	address: String,

	database: String,

	#[serde(default = "Config::default_tls")]
	tls: bool
}

impl Config {
	pub fn mysql_opts(&self) -> mysql::Opts {
		// Configure user, password, db
		let mut builder = mysql::OptsBuilder::new()
			.user(Some(&self.user))
			.pass(Some(&self.password))
			.db_name(Some(&self.database));
		// Configure connection addr
		builder = if self.protocol == "SOCKET" {
			builder.socket(Some(&self.address))
		} else {
			builder.ip_or_hostname(Some(&self.address))
		};
		// Configure TLS
		if self.tls {
			builder = builder.ssl_opts(mysql::SslOpts::default());
		}
		builder.into()
	}

	// Print-friendly identifier containing protocol, address, database, and tls options
	pub fn pretty_name(&self) -> String {
		// Format here is inspired by mysql DSNs
		if self.protocol == "SOCKET" {
			format!("unix/{}", self.database)
		} else {
			format!("{}({})/{}?tls={}", self.protocol, self.address, self.database, self.tls)
		}
	}

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
